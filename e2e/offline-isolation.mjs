import assert from 'node:assert/strict';
import { execFileSync, spawn } from 'node:child_process';
import { readlink } from 'node:fs/promises';
import { connect, createServer } from 'node:net';

export async function runIsolated(testFile) {
  assert.equal(process.platform, 'linux', 'Offline acceptance requires Linux network namespaces.');
  const parentNamespace = await readlink('/proc/self/ns/net');
  const childEnv = { ...process.env, AXIOM_E2E_PARENT_NETNS: parentNamespace };
  // Node suppresses nested --test discovery when this worker-only marker is inherited.
  delete childEnv.NODE_TEST_CONTEXT;
  await new Promise((resolve, reject) => {
    const child = spawn(
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
      },
    );
    child.once('error', reject);
    child.once('exit', (code, signal) => {
      if (code === 0) resolve();
      else
        reject(
          new Error(
            `Offline acceptance failed (${signal ?? code}). Isolation is mandatory; check unshare/ip permissions and the inner test output.`,
          ),
        );
    });
  });
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
