import assert from 'node:assert/strict';
import { spawn } from 'node:child_process';
import { access } from 'node:fs/promises';
import { createServer } from 'node:net';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { Builder, By, Key, until } from 'selenium-webdriver';

export const repositoryRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
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

export async function stopProcess(child) {
  if (child.exitCode !== null) return;
  child.kill('SIGTERM');
  await Promise.race([
    new Promise((resolve) => child.once('exit', resolve)),
    new Promise((resolve) => setTimeout(resolve, 3_000)),
  ]);
  if (child.exitCode === null) child.kill('SIGKILL');
}

export async function startSession(isolatedDataHome, driverProcesses) {
  await access(applicationPath);
  const [driverPort, nativePort] = await Promise.all([availablePort(), availablePort()]);
  const driverUrl = `http://127.0.0.1:${driverPort}`;
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
  driverProcesses.push(driverProcess);
  driverProcess.stdout.on('data', (chunk) => driverOutput.push(chunk.toString()));
  driverProcess.stderr.on('data', (chunk) => driverOutput.push(chunk.toString()));
  driverProcess.on('error', (error) => driverOutput.push(`${error.message}\n`));

  await waitForDriver(driverUrl, driverProcess, driverOutput);
  const driver = await new Builder()
    .usingServer(driverUrl)
    .withCapabilities({
      browserName: 'wry',
      'tauri:options': { application: applicationPath },
    })
    .build();
  return { driver, driverProcess };
}

export async function createWorkspaceFromFirstLaunch(driver, subject) {
  const firstLaunch = await driver.wait(
    until.elementLocated(By.css('[data-route="firstLaunch"]')),
    10_000,
  );
  assert.equal(await firstLaunch.isDisplayed(), true);
  await driver.findElement(By.xpath("//button[normalize-space(.)='Continue']")).click();

  await driver.wait(until.elementLocated(By.css('[data-route="createWorkspace"]')), 10_000);
  const workspaceSubject = await driver.findElement(
    By.xpath("//label[span[normalize-space(.)='Subject']]/input"),
  );
  await workspaceSubject.click();
  // Keep selection, clearing, and typing in separate Actions requests so React processes
  // the controlled input's cleared state before WebKit dispatches the replacement text.
  await driver
    .actions({ async: true })
    .keyDown(Key.CONTROL)
    .sendKeys('a')
    .keyUp(Key.CONTROL)
    .perform();
  await driver.actions({ async: true }).sendKeys(Key.BACK_SPACE).perform();
  await driver.wait(
    async () => (await workspaceSubject.getAttribute('value')) === '',
    2_000,
    'subject field did not clear',
  );
  await driver.actions({ async: true }).sendKeys(subject).perform();
  assert.equal(await workspaceSubject.getAttribute('value'), subject);
  await driver.findElement(By.xpath("//button[normalize-space(.)='Create Workspace']")).click();

  await driver.wait(until.elementLocated(By.css('[data-route="home"]')), 10_000);
}
