const { When, Then } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");

// The driver and fixture staging live in parse_display.steps.js; these steps
// use the cucumber World bridge (this.driver). Clicks and reads go through
// executeScript re-queries — tauri-wd can't marshal element handles and a
// re-render would stale them.

When("I open the Accounts root folder", async function () {
  // Level 0 of the Accounts pane: the entity's own folders. Unlike the
  // header_matches_balances_report step this does not wait for a priced
  // folder total, so it works for fixtures without base-currency prices.
  await this.driver.executeScript(
    `document.querySelector('.sidebarNav__item[data-view="accounts"]')?.click();
     document.querySelector('[data-testid="sidebar-folder-root"]')?.click();`,
  );
  await this.driver.wait(async () => {
    return this.driver.executeScript(
      `return document.querySelector('[data-testid="drilldown-item"]') !== null
          && document.querySelector('[data-testid="drilldown-back"]') === null;`,
    );
  }, 10_000, "Accounts pane never showed its level-0 folders");
});

When("I open the Categories roots", async function () {
  // The Categories drill path persists across scenarios (and vault restarts),
  // so a previous scenario may have left the pane at a leaf level where no
  // group row renders. Switch to the pane and pop back to its roots.
  await this.driver.executeScript(
    `document.querySelector('.sidebarNav__item[data-view="categories"]')?.click();`,
  );
  for (let i = 0; i < 10; i++) {
    const popped = await this.driver.executeScript(
      `const back = document.querySelector('[data-testid="drilldown-back"]');
       if (back) { back.click(); return true; }
       return false;`,
    );
    if (!popped) break;
    await this.driver.sleep(200);
  }
  await this.driver.wait(until.elementLocated(By.css('[data-testid="drilldown-item"]')), 10_000);
});

When("I drill into the sidebar folder {string}", async function (group) {
  const selector = `[data-testid="drilldown-item"][data-group="${group}"]`;
  await this.driver.wait(until.elementLocated(By.css(selector)), 10_000);
  await this.driver.executeScript(`document.querySelector(${JSON.stringify(selector)})?.click();`);
});

When("I go back one sidebar level", async function () {
  const selector = '[data-testid="drilldown-back"]';
  await this.driver.wait(until.elementLocated(By.css(selector)), 10_000);
  await this.driver.executeScript(`document.querySelector(${JSON.stringify(selector)})?.click();`);
});

Then("the folder header should show {string}", async function (expected) {
  let text = "";
  await this.driver.wait(async () => {
    text = await this.driver.executeScript(
      `const el = document.querySelector('[data-testid="selected-account"]');
       return el ? el.textContent.trim() : "";`,
    );
    return text === expected;
  }, 10_000, () => `folder header shows "${text}", expected "${expected}"`);
});
