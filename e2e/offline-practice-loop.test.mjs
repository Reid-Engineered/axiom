import assert from 'node:assert/strict';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { setTimeout as delay } from 'node:timers/promises';
import { fileURLToPath } from 'node:url';
import { By, Key, until } from 'selenium-webdriver';
import { startSession, stopProcess, createWorkspaceFromFirstLaunch } from './native-helpers.mjs';
import { assertOfflineIsolation, markOfflineComplete, runIsolated } from './offline-isolation.mjs';
import { shellVolumeAnswer } from './shell-volume.mjs';

test(
  'native Practice completes with external networking disabled',
  { timeout: process.env.AXIOM_E2E_PARENT_NETNS ? 120_000 : 150_000 },
  async (context) => {
    if (!process.env.AXIOM_E2E_PARENT_NETNS) {
      await runIsolated(fileURLToPath(import.meta.url), { signal: context.signal });
      return;
    }
    await assertOfflineIsolation(process.env.AXIOM_E2E_PARENT_NETNS);
    const isolatedDataHome = await mkdtemp(path.join(tmpdir(), 'axiom-e2e-offline-'));
    const processes = [];
    let driver;
    try {
      ({ driver } = await startSession(isolatedDataHome, processes));
      await createWorkspaceFromFirstLaunch(driver, 'Offline Practice');
      const click = async (xpath) => {
        const element = await driver.wait(until.elementLocated(By.xpath(xpath)), 10_000);
        await element.click();
      };
      await click(
        "//aside//nav[@aria-label='Primary']//button[contains(normalize-space(.), 'Offline Practice')]",
      );
      await driver.wait(until.elementLocated(By.css('[data-route="workspaceOverview"]')), 10_000);
      await click(
        "//button[contains(normalize-space(.), 'The Method of Cylindrical Shells (Vertical Axis of Revolution)')]",
      );
      await driver.wait(until.elementLocated(By.css('[data-route="conceptView"]')), 10_000);
      await click("//button[normalize-space(.)='Practice this']");
      await driver.wait(until.elementLocated(By.css('[data-route="studySession"]')), 10_000);
      const pane = By.css('section[aria-labelledby="problem-heading"]');
      const answerLocator = By.xpath("//label[span[normalize-space(.)='Answer']]/input");
      const input = await driver.wait(until.elementLocated(answerLocator), 10_000);
      const prompt = await driver
        .findElement(By.css('section[aria-labelledby="problem-heading"] > p:nth-of-type(2)'))
        .getText();
      const answer = shellVolumeAnswer(prompt);
      assert.equal(
        await driver.findElement(By.id('problem-heading')).getAttribute('textContent'),
        'Problem 1',
      );
      // The default CI window clips the answer below the scrollable working area.
      await driver.executeScript('arguments[0].scrollIntoView({ block: "center" });', input);
      await input.click();
      await driver.actions({ async: true }).sendKeys('0').perform();
      await click("//button[normalize-space(.)='Check']");
      await driver.wait(
        until.elementLocated(
          By.xpath("//p[@role='status' and normalize-space(.)='That does not match yet.']"),
        ),
        10_000,
      );
      await driver.wait(
        async () => (await driver.findElement(pane).findElements(By.css('ul li'))).length === 1,
        10_000,
        'Wrong answer did not reveal the first persisted hint.',
      );
      assert.ok((await driver.findElement(pane).findElement(By.css('ul li')).getText()).trim());
      await click("//button[normalize-space(.)='Hint']");
      await driver.wait(
        async () => (await driver.findElement(pane).findElements(By.css('ul li'))).length === 2,
        10_000,
        'Explicit Hint did not reveal the next hint.',
      );
      const hints = await driver.findElement(pane).findElements(By.css('ul li'));
      assert.notEqual(await hints[0].getText(), await hints[1].getText());
      await driver.executeScript('arguments[0].scrollIntoView({ block: "center" });', input);
      await input.click();
      await driver
        .actions({ async: true })
        .keyDown(Key.CONTROL)
        .sendKeys('a')
        .keyUp(Key.CONTROL)
        .perform();
      await driver.actions({ async: true }).sendKeys(Key.BACK_SPACE).perform();
      await driver.wait(async () => (await input.getAttribute('value')) === '', 2_000);
      await driver.actions({ async: true }).sendKeys(answer).perform();
      assert.equal(await input.getAttribute('value'), answer);
      await click("//button[normalize-space(.)='Check']");
      await driver.wait(
        until.elementLocated(By.xpath("//p[@role='status' and normalize-space(.)='Correct.']")),
        10_000,
      );
      await click("//button[normalize-space(.)='Next problem']");
      await driver.wait(
        async () =>
          (await driver.findElement(By.id('problem-heading')).getAttribute('textContent')) ===
          'Problem 2',
        10_000,
      );
      await driver.wait(
        async () => (await driver.findElement(answerLocator).getAttribute('value')) === '',
        10_000,
      );
      await driver.wait(
        until.elementLocated(By.xpath("//button[normalize-space(.)='Check']")),
        10_000,
      );
      shellVolumeAnswer(
        await driver
          .findElement(By.css('section[aria-labelledby="problem-heading"] > p:nth-of-type(2)'))
          .getText(),
      );
      assert.equal((await driver.findElement(pane).findElements(By.css('ul li'))).length, 0);
      assert.equal(
        (await driver.findElement(pane).findElements(By.css('[role="status"]'))).length,
        0,
      );
      await assertOfflineIsolation(process.env.AXIOM_E2E_PARENT_NETNS);
      await markOfflineComplete();
    } finally {
      if (driver) await driver.quit().catch(() => undefined);
      await Promise.allSettled(processes.map(stopProcess));
      await delay(250);
      await rm(isolatedDataHome, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
    }
  },
);
