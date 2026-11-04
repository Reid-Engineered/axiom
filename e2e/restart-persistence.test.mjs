import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { setTimeout as delay } from 'node:timers/promises';
import test from 'node:test';
import { By, until } from 'selenium-webdriver';
import { startSession, stopProcess, createWorkspaceFromFirstLaunch } from './native-helpers.mjs';

test('workspace data survives an application restart', { timeout: 120_000 }, async () => {
  const isolatedDataHome = await mkdtemp(path.join(tmpdir(), 'axiom-e2e-restart-'));
  const driverProcesses = [];
  const firstWorkspace = 'Axiom Restart Session One';
  const secondWorkspace = 'Axiom Restart Session Two';
  let activeDriver;

  try {
    const firstSession = await startSession(isolatedDataHome, driverProcesses);
    activeDriver = firstSession.driver;
    await createWorkspaceFromFirstLaunch(activeDriver, firstWorkspace);
    await activeDriver.quit();
    activeDriver = undefined;
    await stopProcess(firstSession.driverProcess);

    const secondSession = await startSession(isolatedDataHome, driverProcesses);
    activeDriver = secondSession.driver;
    await createWorkspaceFromFirstLaunch(activeDriver, secondWorkspace);

    for (const workspaceName of [firstWorkspace, secondWorkspace]) {
      const workspaceEntry = await activeDriver.wait(
        until.elementLocated(
          By.xpath(`//button[contains(normalize-space(.), '${workspaceName}')]`),
        ),
        10_000,
        `Expected Home to list ${workspaceName}.`,
      );
      assert.equal(await workspaceEntry.isDisplayed(), true);
    }
  } finally {
    if (activeDriver) await activeDriver.quit().catch(() => undefined);
    await Promise.allSettled(driverProcesses.map((driverProcess) => stopProcess(driverProcess)));
    await delay(250);
    await rm(isolatedDataHome, {
      recursive: true,
      force: true,
      maxRetries: 5,
      retryDelay: 100,
    });
  }
});
