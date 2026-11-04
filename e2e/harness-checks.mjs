import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import { mkdtemp, readFile, readlink, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { setTimeout as delay } from 'node:timers/promises';
import { assertOfflineIsolation, runIsolated } from './offline-isolation.mjs';
import { shellVolumeAnswer } from './shell-volume.mjs';
import { stopProcess } from './native-helpers.mjs';

test(
  'driver cleanup stops descendants even after the driver exits',
  {
    skip: process.platform !== 'linux',
    timeout: 10_000,
  },
  async () => {
    const child = spawn(
      process.execPath,
      [
        '-e',
        `
    const { spawn } = require('node:child_process');
    const orphan = spawn(process.execPath, ['-e', 'setInterval(() => {}, 1000)'], { stdio: 'inherit' });
    console.log(orphan.pid);
    orphan.unref();
  `,
      ],
      { detached: true, stdio: ['ignore', 'pipe', 'pipe'] },
    );
    let output = '';
    child.stdout.on('data', (chunk) => {
      output += chunk;
    });
    try {
      await once(child, 'exit');
      const pid = Number(output.trim());
      assert.ok(pid > 0, 'Driver fixture did not start a descendant.');
      await stopProcess(child);
      assert.ok(child.stdout.destroyed && child.stderr.destroyed);
      let stopped = false;
      for (let attempt = 0; attempt < 20 && !stopped; attempt += 1) {
        const stat = await readFile(`/proc/${pid}/stat`, 'utf8').catch((error) => {
          if (error.code === 'ENOENT') return '';
          throw error;
        });
        stopped = !stat || /^\d+ \(.*\) Z /.test(stat);
        if (!stopped) await delay(100);
      }
      assert.ok(stopped, 'Driver descendant remained running after cleanup.');
    } finally {
      await stopProcess(child);
    }
  },
);

test('volume oracle reads supported prompts and rejects unrelated/sample prose', () => {
  const prompt = (c, b) =>
    `Define R as the region bounded above by the graph of f(x) = ${c}x - x^2 and below by the\nx-axis over the interval [0, ${b}]. Find the volume of the solid of revolution formed by\nrevolving R around the y-axis.`;
  assert.equal(shellVolumeAnswer(prompt(2, 1)), '5*pi/6');
  assert.equal(shellVolumeAnswer(prompt(6, 6)), '1296*pi/6');
  assert.throws(() => shellVolumeAnswer('region y = x² − 1 on [1, 3]'));
  assert.throws(() => shellVolumeAnswer(prompt(2, 3)));
});

test('isolation guard rejects a forged child marker in the original namespace', async () => {
  if (process.platform !== 'linux') {
    await assert.rejects(assertOfflineIsolation('forged'), /requires Linux/);
    return;
  }
  await assert.rejects(
    assertOfflineIsolation(await readlink('/proc/self/ns/net')),
    /still in the parent network namespace/,
  );
});

test(
  'isolated re-execution actually runs the nested Node test',
  {
    skip: process.platform !== 'linux' ? 'Namespace regression check requires Linux.' : false,
    timeout: 10_000,
  },
  async () => {
    const directory = await mkdtemp(path.join(tmpdir(), 'axiom-isolation-check-'));
    try {
      const marker = path.join(directory, 'completed');
      const fixture = path.join(directory, 'nested.test.mjs');
      const isolationModule = new URL('./offline-isolation.mjs', import.meta.url).href;
      await writeFile(
        fixture,
        `
      import test from 'node:test';
      import { writeFile } from 'node:fs/promises';
      import { assertOfflineIsolation, markOfflineComplete } from ${JSON.stringify(isolationModule)};
      test('nested isolation proof', async () => {
        await assertOfflineIsolation(process.env.AXIOM_E2E_PARENT_NETNS);
        await writeFile(${JSON.stringify(marker)}, 'completed');
        await markOfflineComplete();
      });
    `,
      );
      await runIsolated(fixture);
      assert.equal(await readFile(marker, 'utf8'), 'completed');
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  'isolated wrapper rejects a successful child without completion evidence',
  {
    skip: process.platform !== 'linux',
    timeout: 10_000,
  },
  async () => {
    const directory = await mkdtemp(path.join(tmpdir(), 'axiom-empty-child-'));
    try {
      const fixture = path.join(directory, 'empty.test.mjs');
      await writeFile(fixture, 'export {};');
      await assert.rejects(runIsolated(fixture), /without completing the learner flow/);
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);

test(
  'isolated wrapper stops its process group when aborted',
  {
    skip: process.platform !== 'linux',
    timeout: 10_000,
  },
  async () => {
    const directory = await mkdtemp(path.join(tmpdir(), 'axiom-aborted-child-'));
    const controller = new AbortController();
    let running;
    try {
      const fixture = path.join(directory, 'stalled.test.mjs');
      const pidFile = path.join(directory, 'grandchild.pid');
      await writeFile(
        fixture,
        `import test from 'node:test';
        import { spawn } from 'node:child_process';
        import { writeFile } from 'node:fs/promises';
        test('stalled', async () => {
          const grandchild = spawn('sleep', ['60']);
          await writeFile(${JSON.stringify(pidFile)}, String(grandchild.pid));
          await new Promise(() => {});
        });`,
      );
      running = runIsolated(fixture, { signal: controller.signal });
      void running.catch(() => undefined);
      let pid;
      for (let attempt = 0; attempt < 50 && !pid; attempt += 1) {
        pid = Number(
          await readFile(pidFile, 'utf8').catch((error) => {
            if (error.code === 'ENOENT') return '';
            throw error;
          }),
        );
        if (!pid) await delay(100);
      }
      assert.ok(pid > 0, 'Stalled fixture did not start its grandchild.');
      controller.abort(new Error('Requested abort after grandchild started'));
      await assert.rejects(running, /Requested abort after grandchild started/);
      let stopped = false;
      for (let attempt = 0; attempt < 20 && !stopped; attempt += 1) {
        const stat = await readFile(`/proc/${pid}/stat`, 'utf8').catch((error) => {
          if (error.code === 'ENOENT') return '';
          throw error;
        });
        // A killed orphan can remain a zombie until the host's init reaps it.
        stopped = !stat || /^\d+ \(.*\) Z /.test(stat);
        if (!stopped) await delay(100);
      }
      assert.ok(stopped, 'Grandchild remained running after the wrapper aborted.');
    } finally {
      controller.abort();
      await running?.catch(() => undefined);
      await rm(directory, { recursive: true, force: true });
    }
  },
);
