import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { access, mkdtemp, rm } from 'node:fs/promises';
import { createServer } from 'node:net';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { Builder, By, Key, until } from 'selenium-webdriver';

const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const applicationPath =
  process.env.AXIOM_E2E_APP ?? path.join(repositoryRoot, 'src-tauri', 'target', 'release', 'axiom');
const tauriDriverPath = process.env.TAURI_DRIVER_BIN ?? 'tauri-driver';

async function availablePort() {
  return new Promise((resolve, reject) => {
    const server = createServer();
    server.once('error', reject);
    server.listen(0, '127.0.0.1', () => {
      const address = server.address();
      if (!address || typeof address === 'string') {
        server.close();
        reject(new Error('Could not reserve a WebDriver port.'));
        return;
      }
      server.close((error) => (error ? reject(error) : resolve(address.port)));
    });
  });
}

async function waitForDriver(driverUrl, driverProcess, output) {
  for (let attempt = 0; attempt < 50; attempt += 1) {
    if (driverProcess.exitCode !== null) {
      throw new Error(`tauri-driver exited before becoming ready.\n${output.join('')}`);
    }
    try {
      const response = await fetch(`${driverUrl}/status`);
      if (response.ok) return;
    } catch {
      // The driver needs a short startup window before its status endpoint is available.
    }
    await new Promise((resolve) => setTimeout(resolve, 200));
  }
  throw new Error(`tauri-driver did not become ready.\n${output.join('')}`);
}

async function stopProcess(child) {
  if (child.exitCode !== null) return;
  child.kill('SIGTERM');
  await Promise.race([
    new Promise((resolve) => child.once('exit', resolve)),
    new Promise((resolve) => setTimeout(resolve, 3_000)),
  ]);
  if (child.exitCode === null) child.kill('SIGKILL');
}

test(
  'first launch creates a workspace through SQLite and reaches home',
  { timeout: 60_000 },
  async () => {
    await access(applicationPath);
    const [driverPort, nativePort] = await Promise.all([availablePort(), availablePort()]);
    const driverUrl = `http://127.0.0.1:${driverPort}`;
    const isolatedDataHome = await mkdtemp(path.join(tmpdir(), 'axiom-e2e-'));
    const driverOutput = [];
    const driverProcess = spawn(
      tauriDriverPath,
      ['--port', String(driverPort), '--native-port', String(nativePort)],
      {
        cwd: repositoryRoot,
        env: { ...process.env, XDG_DATA_HOME: isolatedDataHome },
        stdio: ['ignore', 'pipe', 'pipe'],
      },
    );
    driverProcess.stdout.on('data', (chunk) => driverOutput.push(chunk.toString()));
    driverProcess.stderr.on('data', (chunk) => driverOutput.push(chunk.toString()));
    driverProcess.on('error', (error) => driverOutput.push(`${error.message}\n`));

    let driver;
    try {
      await waitForDriver(driverUrl, driverProcess, driverOutput);
      driver = await new Builder()
        .usingServer(driverUrl)
        .withCapabilities({
          browserName: 'wry',
          'tauri:options': { application: applicationPath },
        })
        .build();

      const firstLaunch = await driver.wait(
        until.elementLocated(By.css('[data-route="firstLaunch"]')),
        10_000,
      );
      assert.equal(await firstLaunch.isDisplayed(), true);

      // A fresh install (isolated XDG_DATA_HOME, no prior SQLite data) must show the
      // sidebar for visual continuity, but with zero workspaces — the intentional empty
      // state, never the populated tree.
      const sidebarNav = await driver.wait(
        until.elementLocated(By.css('aside nav[aria-label="Primary"]')),
        10_000,
      );
      assert.equal(await sidebarNav.isDisplayed(), true);
      const emptyStateMessages = await sidebarNav.findElements(
        By.xpath(".//p[normalize-space(.)='No workspaces yet.']"),
      );
      assert.ok(emptyStateMessages.length > 0, 'expected the intentional empty sidebar state');
      const populatedTreeAffordance = await sidebarNav.findElements(
        By.xpath(".//button[normalize-space(.)='+ New Workspace']"),
      );
      assert.equal(
        populatedTreeAffordance.length,
        0,
        'the populated-tree affordance must not render before any workspace exists',
      );

      await driver.findElement(By.xpath("//button[normalize-space(.)='Continue']")).click();

      await driver.wait(until.elementLocated(By.css('[data-route="createWorkspace"]')), 10_000);
      const workspaceSubject = await driver.findElement(
        By.xpath("//label[span[normalize-space(.)='Subject']]/input"),
      );
      await workspaceSubject.click();
      // WebKitGTK 2.52 rejects tauri-driver's legacy Element Send Keys payload; Actions is W3C-safe.
      await driver
        .actions({ async: true })
        .keyDown(Key.CONTROL)
        .sendKeys('a')
        .keyUp(Key.CONTROL)
        .sendKeys('Axiom E2E Subject')
        .perform();
      assert.equal(await workspaceSubject.getAttribute('value'), 'Axiom E2E Subject');
      await driver.findElement(By.xpath("//button[normalize-space(.)='Create Workspace']")).click();

      await driver.wait(until.elementLocated(By.css('[data-route="home"]')), 10_000);
      const heading = await driver.findElement(By.xpath("//h1[normalize-space(.)='Workspaces']"));
      assert.equal(await heading.isDisplayed(), true);
      const createdWorkspaceEntries = await driver.findElements(
        By.xpath("//button[contains(normalize-space(.), 'Axiom E2E Subject')]"),
      );
      assert.ok(createdWorkspaceEntries.length > 0);

      // The sidebar persists across the whole flow — confirm it specifically picked up
      // the newly created workspace (not just that it appears somewhere on the page) and
      // no longer shows the first-launch empty state.
      await driver.wait(
        until.elementLocated(
          By.xpath(
            "//aside//nav[@aria-label='Primary']//button[contains(normalize-space(.), 'Axiom E2E Subject')]",
          ),
        ),
        10_000,
        'sidebar did not pick up the newly created workspace',
      );
      const homeSidebarNav = await driver.findElement(
        By.css('aside nav[aria-label="Primary"]'),
      );
      const emptyStateAfterCreate = await homeSidebarNav.findElements(
        By.xpath(".//p[normalize-space(.)='No workspaces yet.']"),
      );
      assert.equal(emptyStateAfterCreate.length, 0);
    } finally {
      if (driver) await driver.quit().catch(() => undefined);
      await stopProcess(driverProcess);
      await rm(isolatedDataHome, {
        recursive: true,
        force: true,
        maxRetries: 5,
        retryDelay: 100,
      });
    }
  },
);
