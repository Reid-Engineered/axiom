import assert from 'node:assert/strict';
import { mkdtemp, readFile, readlink, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { assertOfflineIsolation, runIsolated } from './offline-isolation.mjs';
import { shellVolumeAnswer } from './shell-volume.mjs';

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
    try {
      const fixture = path.join(directory, 'stalled.test.mjs');
      await writeFile(
        fixture,
        "import test from 'node:test'; test('stalled', async () => new Promise(() => setInterval(() => {}, 1000)));",
      );
      await assert.rejects(runIsolated(fixture, { signal: AbortSignal.timeout(2_000) }), {
        name: 'TimeoutError',
      });
    } finally {
      await rm(directory, { recursive: true, force: true });
    }
  },
);
