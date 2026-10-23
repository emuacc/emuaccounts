const assert = require("node:assert/strict");
const { When, Then } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");

// Session setup is owned by parse_display.steps.js BeforeAll/AfterAll; these
// steps consume the driver via the cucumber-js World (this.driver). Everything
// that touches the DOM runs inside executeScript re-queries — tauri-wd can't
// marshal element handles and re-renders would stale them.

/** "$79,150 AUD" / "-$1,234" → -1234 (whole base-currency units). */
function parseWholeAmount(text) {
  const m = String(text).replace(/,/g, "").match(/-?\d+(?:\.\d+)?/);
  assert.ok(m, `no amount in "${text}"`);
  const negative = /^\s*-/.test(text) || /\(\s*\$?\d/.test(text);
  const value = Math.round(Number(m[0]));
  return negative && value > 0 ? -value : value;
}

const HEADER_SELECTOR = ".accountHeader__balance--folder";
const REPORT_TOTAL_SELECTOR = ".reportSummary__row--total span:last-child";

When("I open the root folder view", async function () {
  await this.driver.executeScript(
    `document.querySelector('[data-view="accounts"]')?.click();
     document.querySelector('[data-testid="sidebar-folder-root"]')?.click();`,
  );
  await this.driver.wait(until.elementLocated(By.css(HEADER_SELECTOR)), 15_000);
});

When("I note the folder header total", async function () {
  // The header renders once the backend tree totals arrive; poll until the
  // folder balance carries a number.
  let text = "";
  await this.driver.wait(async () => {
    text = await this.driver.executeScript(
      `const el = document.querySelector(${JSON.stringify(HEADER_SELECTOR)});
       return el ? el.textContent.trim() : "";`,
    );
    return /\d/.test(text);
  }, 20_000, () => `folder header never showed a total (saw "${text}")`);
  this.folderHeaderTotal = parseWholeAmount(text);
});

When("I open the Balances report", async function () {
  await this.driver.executeScript(`document.querySelector('[data-view="reports"]')?.click();`);
  await this.driver.wait(until.elementLocated(By.css('[data-report="balances"]')), 15_000);
  await this.driver.executeScript(`document.querySelector('[data-report="balances"]')?.click();`);
  await this.driver.wait(until.elementLocated(By.css(REPORT_TOTAL_SELECTOR)), 30_000);
});

Then("the Balances report total equals the noted folder header total", async function () {
  assert.notEqual(this.folderHeaderTotal, undefined, "folder header total was never noted");
  let text = "";
  await this.driver.wait(async () => {
    text = await this.driver.executeScript(
      `const el = document.querySelector(${JSON.stringify(REPORT_TOTAL_SELECTOR)});
       return el ? el.textContent.trim() : "";`,
    );
    return /\d/.test(text);
  }, 20_000, () => `Balances report total never rendered (saw "${text}")`);
  const reportTotal = parseWholeAmount(text);
  assert.equal(
    reportTotal,
    this.folderHeaderTotal,
    `Balances report total ${reportTotal} != folder header total ${this.folderHeaderTotal}`,
  );
});
