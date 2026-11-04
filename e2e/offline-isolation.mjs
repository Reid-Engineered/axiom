import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { mkdtemp, readFile, readlink, rm, writeFile } from 'node:fs/promises';
import { connect, createServer } from 'node:net';
import { tmpdir } from 'node:os';
import path from 'node:path';

export async function runIsolated(testFile, { signal } = {}) {
  assert.equal(process.platform, 'linux', 'Offline acceptance requires Linux network namespaces.');
  const parentNamespace = await readlink('/proc/self/ns/net');
  const directory = await mkdtemp(path.join(tmpdir(), 'axiom-offline-completion-'));
  const completionFile = path.join(directory, 'completed');
  const childEnv = {
    ...process.env,
    AXIOM_E2E_PARENT_NETNS: parentNamespace,
    AXIOM_E2E_COMPLETION_FILE: completionFile,
  };
  // Node suppresses nested --test discovery when this worker-only marker is inherited.
  delete childEnv.NODE_TEST_CONTEXT;
  let child;
  let killTimer;
  const killGroup = (killSignal) => {
    if (!child?.pid) return;
    try {
      process.kill(-child.pid, killSignal);
    } catch (error) {
      if (error.code !== 'ESRCH') throw error;
    }
  };
  const abort = () => {
    killGroup('SIGTERM');
    killTimer = setTimeout(() => killGroup('SIGKILL'), 1_000);
    killTimer.unref();
  };
  try {
    signal?.throwIfAborted();
    await new Promise((resolve, reject) => {
      child = spawn(
        'unshare',
        [
          '--user',
          '--map-root-user',
          '--net',
          'sh',
          '-ec',
          'ip link set lo up; exec "$@"',
          'axiom-offline',
          process.execPath,
          '--test',
          testFile,
        ],
        {
          stdio: 'inherit',
          env: childEnv,
          detached: true,
        },
      );
      signal?.addEventListener('abort', abort, { once: true });
      child.once('error', reject);
      child.once('exit', (code, exitSignal) => {
        if (signal?.aborted) reject(signal.reason);
        else if (code === 0) resolve();
        else
          reject(
            new Error(
              `Offline acceptance failed (${exitSignal ?? code}). Isolation is mandatory; check unshare/ip permissions and the inner test output.`,
            ),
          );
      });
    });
    signal?.throwIfAborted();
    assert.equal(
      await readFile(completionFile, 'utf8').catch((error) => {
        if (error.code === 'ENOENT') return '';
        throw error;
      }),
      'completed',
      'Offline acceptance child exited without completing the learner flow.',
    );
  } finally {
    signal?.removeEventListener('abort', abort);
    clearTimeout(killTimer);
    killGroup('SIGKILL');
    await rm(directory, { recursive: true, force: true });
  }
}

export async function markOfflineComplete() {
  assert.ok(process.env.AXIOM_E2E_COMPLETION_FILE, 'Missing offline completion marker path.');
  await writeFile(process.env.AXIOM_E2E_COMPLETION_FILE, 'completed');
}

function connection(host, port) {
  return new Promise((resolve, reject) => {
    const socket = connect({ host, port });
    socket.setTimeout(2_000, () => socket.destroy(new Error(`Timed out connecting to ${host}`)));
    socket.once('connect', () => {
      socket.destroy();
      resolve();
    });
    socket.once('error', reject);
  });
}

export async function assertOfflineIsolation(parentNamespace) {
  assert.equal(process.platform, 'linux', 'Offline isolation guard requires Linux.');
  assert.ok(parentNamespace, 'Offline isolation guard requires the parent namespace identity.');
  assert.notEqual(
    await readlink('/proc/self/ns/net'),
    parentNamespace,
    'Offline isolation guard: still in the parent network namespace.',
  );
  const links = JSON.parse(execFileSync('ip', ['-j', 'link', 'show'], { encoding: 'utf8' }));
  assert.deepEqual(
    links.map((link) => link.ifname),
    ['lo'],
    'Offline isolation guard: external network interface present.',
  );
  for (const family of ['-4', '-6']) {
    const routes = JSON.parse(
      execFileSync('ip', ['-j', family, 'route', 'show', 'table', 'all'], { encoding: 'utf8' }),
    );
    assert.ok(
      routes.every((route) => route.dev === 'lo'),
      `Offline isolation guard: external ${family} route present.`,
    );
  }
  for (const host of ['1.1.1.1', '2606:4700:4700::1111']) {
    await assert.rejects(
      connection(host, 443),
      { code: 'ENETUNREACH' },
      `Offline isolation guard: ${host} must be unreachable (not merely time out).`,
    );
  }
  const server = createServer((socket) => socket.end());
  try {
    await new Promise((resolve, reject) => {
      server.once('error', reject);
      server.listen(0, '127.0.0.1', resolve);
    });
    await connection('127.0.0.1', server.address().port);
  } finally {
    await new Promise((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve())),
    );
  }
}
