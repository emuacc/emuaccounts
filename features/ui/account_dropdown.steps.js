const assert = require("node:assert/strict");
const { When, Then } = require("@cucumber/cucumber");
const { By, Key, until } = require("selenium-webdriver");

// The shared selenium driver is owned by parse_display.steps.js (BeforeAll/
// AfterAll); steps use it via the World (this.driver). "the app is running",
// "I parse the transactions file ..." and "I select the account ..." come from
// parse_display.steps.js, the row and "Edit Rule" steps from edit_rule.steps.js.

const DROPDOWN = ".accountInput-dropdown";
const AMOUNT_ACCOUNT = "#ruleAmountAccount";

// The scroll a user's next wheel tick would fire: an orphaned dropdown repositions on it.
const DROPDOWN_LEFT_BEHIND = `
  window.dispatchEvent(new Event("scroll"));
  const el = document.querySelector(arguments[0]);
  if (!el) return null;
  const r = el.getBoundingClientRect();
  return { text: el.textContent, top: r.top, left: r.left };`;

function located(driver, css) {
  return driver.wait(until.elementLocated(By.css(css)), 5_000);
}

// The editor seeds the field from the row, so replace what is there.
async function typeIntoAmountAccount(driver, text) {
  const input = await located(driver, AMOUNT_ACCOUNT);
  await input.clear();
  await input.click();
  await input.sendKeys(text);
  return input;
}

When("I type {string} into the rule's Amount Account field", async function (text) {
  await typeIntoAmountAccount(this.driver, text);
});

When("I choose the suggestion for {string} in the rule's Amount Account field", async function (text) {
  const input = await typeIntoAmountAccount(this.driver, text);
  await located(this.driver, DROPDOWN);
  await input.sendKeys(Key.ENTER);
  await this.driver.wait(
    async () => (await this.driver.findElements(By.css(DROPDOWN))).length === 0,
    5_000,
    "Choosing a suggestion should close the dropdown",
  );
});

When("I click into the rule's Amount Account field", async function () {
  await (await located(this.driver, AMOUNT_ACCOUNT)).click();
});

When("I click into the rule's Comment field", async function () {
  await (await located(this.driver, "#ruleComment")).click();
});

When("I cancel the rule editor", async function () {
  await (await located(this.driver, "#ruleCancel")).click();
});

When("I press Alt+Left", async function () {
  await this.driver.actions().keyDown(Key.ALT).sendKeys(Key.ARROW_LEFT).keyUp(Key.ALT).perform();
});

Then("the account dropdown is open", async function () {
  await located(this.driver, DROPDOWN);
});

Then("the rule editor is closed", async function () {
  await this.driver.wait(
    async () => (await this.driver.findElements(By.css(".ruleEditorBar"))).length === 0,
    5_000,
    "The rule editor should have closed",
  );
});

Then("no account dropdown is left on the page", async function () {
  let leftBehind;
  await this.driver
    .wait(async () => {
      leftBehind = await this.driver.executeScript(DROPDOWN_LEFT_BEHIND, DROPDOWN);
      return leftBehind === null;
    }, 3_000)
    .catch(() => assert.fail(`An account dropdown was left on the page: ${JSON.stringify(leftBehind)}`));
});
