const { When, Then } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");

// The shared selenium driver is owned by parse_display.steps.js (BeforeAll/
// AfterAll); steps use it via the World (this.driver). "the app is running",
// "I parse the transactions file ..." and "the sidebar should include the
// account ..." come from parse_display.steps.js, the Categories steps from
// categories.steps.js.

/** Wait for the mutation queue to drain: the rebuild and the refresh after it
 *  are done, so the next step (or scenario) sees the settled vault. */
async function waitForMutations(driver) {
  await driver.wait(
    async () => (await driver.executeScript("return window.__diag().pendingMutations")) === 0,
    30_000,
  );
}

When("I recategorise the first transaction row as {string}", async function (account) {
  const cell = await this.driver.wait(
    until.elementLocated(By.css('.cell-clickable[data-rule-type="category"]')),
    10_000,
  );
  await cell.click();
  await this.driver.wait(until.elementLocated(By.css("[data-cell-editing] input")), 10_000);
  // Set the value and fire `input`, then Enter, inside the page: sendKeys into
  // the prefilled editor appends, and clear() blurs it, which reverts the edit.
  // Nothing in the vault matches the new name, so the dropdown's only (and
  // highlighted) item is "+ Add", and Enter saves it as a rule.
  await this.driver.executeScript(
    `const el = document.querySelector("[data-cell-editing] input");
     el.value = arguments[0];
     el.dispatchEvent(new Event("input", { bubbles: true }));
     el.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));`,
    account,
  );
  await waitForMutations(this.driver);
});

When("I add a manual transaction posting to the new category {string}", async function (account) {
  await this.driver.findElement(By.css("#addNew")).click();
  await this.driver.wait(until.elementLocated(By.css("#manualPayee")), 10_000);
  await this.driver.findElement(By.css("#manualPayee")).sendKeys("New category UI");
  await this.driver.findElement(By.css("#manualNarration")).sendKeys("Gift");
  await this.driver.findElement(By.css("#manualAmount")).sendKeys("4.20");
  // One "other account" row whose blank amount auto-balances it.
  await this.driver.findElement(By.css("#manualContraAccount-0")).sendKeys(account);
  // sendKeys returns before the keys have all landed; don't save a half-typed name.
  await this.driver.wait(
    () => this.driver.executeScript(
      `return document.querySelector("#manualContraAccount-0")?.value === arguments[0];`,
      account,
    ),
    5_000,
  );
  await this.driver.findElement(By.css("#manualSave")).click();
  await waitForMutations(this.driver);
});

Then("the account dropdowns should offer {string}", async function (account) {
  // The manual transaction modal's account field: back on the Accounts pane,
  // open it and type the account into the dropdown's filter.
  await this.driver.findElement(By.css('.sidebarNav__item[data-view="accounts"]')).click();
  const addNew = await this.driver.wait(until.elementLocated(By.css("#addNew")), 10_000);
  await addNew.click();
  const field = await this.driver.wait(until.elementLocated(By.css("#manualContraAccount-0")), 10_000);
  await field.sendKeys(account);
  await this.driver.wait(
    until.elementLocated(By.css(`.accountInput-dropdown__item[data-kind="existing"][data-value="${account}"]`)),
    10_000,
  );
  await this.driver.findElement(By.css("#manualCancel")).click();
});
