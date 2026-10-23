const assert = require("node:assert/strict");
const { Then } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");

// The shared selenium driver is owned by parse_display.steps.js (BeforeAll/
// AfterAll) and reaches these steps through the cucumber-js World
// (this.driver). "the app is running", "I parse the transactions file …" and
// "I select the account …" are reused from parse_display.steps.js.

// Everything runs inside executeScript — tauri-wd can't marshal element
// handles, and re-renders would stale them. The category label is the cell's
// first text node: the in-scope child account renders beneath it in its own
// div and is not part of the label.
const ROW_SCRIPT = `
  const notes = arguments[0];
  const row = Array.from(document.querySelectorAll('[data-testid="txn-row"]'))
    .find((r) => (r.querySelector('[data-testid="txn-notes"]')?.textContent || '').trim() === notes);
  if (!row) return null;
  const cell = row.querySelector('td.category');
  return {
    category: cell ? (cell.childNodes[0]?.textContent || '').trim() : null,
    editable: !!(cell && cell.classList.contains('cell-clickable') && cell.getAttribute('data-rule-type') === 'category'),
    fee: (row.querySelector('.fee-line')?.textContent || '').replace(/\\s+/g, ' ').trim() || null,
  };`;

/** The rendered row whose notes equal `notes`, waiting for the table to show it. */
async function rowWithNotes(driver, notes) {
  await driver.wait(until.elementLocated(By.css('[data-testid="txn-row"]')), 10_000);
  let row = null;
  await driver.wait(async () => {
    row = await driver.executeScript(ROW_SCRIPT, notes);
    return row !== null;
  }, 10_000, `no transaction row with notes "${notes}"`);
  return row;
}

Then("the transaction row with notes {string} should show the category {string}", async function (notes, category) {
  const row = await rowWithNotes(this.driver, notes);
  assert.equal(row.category, category);
});

Then("the transaction row with notes {string} should show the fee {string}", async function (notes, fee) {
  const row = await rowWithNotes(this.driver, notes);
  assert.equal(row.fee, fee);
});

Then("the category of the transaction row with notes {string} should not be editable", async function (notes) {
  const row = await rowWithNotes(this.driver, notes);
  assert.equal(row.editable, false, "category cell is a category-rule click target");
});

Then("the category of the transaction row with notes {string} should be editable", async function (notes) {
  const row = await rowWithNotes(this.driver, notes);
  assert.equal(row.editable, true, "category cell is not a category-rule click target");
});
