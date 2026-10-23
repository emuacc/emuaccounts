const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { Given, Then, When } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");

// The shared selenium driver and the harness vault's sources dir reach these
// steps through the cucumber-js World (this.driver, this.sourcesDir), both
// owned by parse_display.steps.js. "the app is running", "I parse the
// transactions file …" and "I select the account …" are reused from there.

/** Stage a `_prices/<name>` file in the harness vault. `convert_to_base_currency`
 *  loads the price graph from the sources dir on every call, so no rebuild is
 *  needed — the file only has to exist before the account is selected. */
Given("the price file {string} contains:", function (name, content) {
  const dir = path.join(this.sourcesDir, "_prices");
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(path.join(dir, name), `${content.trim()}\n`);
});

// Everything runs inside executeScript — tauri-wd can't marshal element
// handles, and re-renders would stale them.
const VALUE_CELL_SCRIPT = `
  const notes = arguments[0];
  const row = Array.from(document.querySelectorAll('[data-testid="txn-row"]'))
    .find((r) => (r.querySelector('[data-testid="txn-notes"]')?.textContent || '').trim() === notes);
  const cell = row && row.querySelector('td.value-clickable');
  if (!cell) return null;
  if (arguments[1]) cell.click();
  return {
    text: (cell.textContent || '').replace(/\\s+/g, ' ').trim(),
    commodity: cell.getAttribute('data-value-commodity'),
    datetime: cell.getAttribute('data-value-datetime'),
    basis: cell.getAttribute('data-value-basis'),
  };`;

const MODAL_SCRIPT = `
  const modal = document.querySelector('[data-testid="set-price-modal"]');
  if (!modal) return null;
  return {
    commodity: modal.querySelector('#setPriceCommodity')?.value ?? null,
    date: modal.querySelector('#setPriceDate')?.value ?? null,
    basis: (modal.querySelector('[data-testid="set-price-basis"]')?.textContent || '').replace(/\\s+/g, ' ').trim(),
  };`;

/** The value cell of the row whose notes equal `notes`, waiting until
 *  `ready(cell)` holds (the value column loads after the rows render). On
 *  timeout the last read is returned so the assertion reports what was there. */
async function valueCellWhen(driver, notes, ready) {
  await driver.wait(until.elementLocated(By.css('[data-testid="txn-row"]')), 10_000);
  let cell = null;
  try {
    await driver.wait(async () => {
      cell = await driver.executeScript(VALUE_CELL_SCRIPT, notes, false);
      return cell !== null && ready(cell);
    }, 10_000);
  } catch {
    // fall through to the caller's assertion on the last read
  }
  assert.ok(cell, `no value cell on the transaction row with notes "${notes}"`);
  return cell;
}

Then("the transaction row with notes {string} should show the value {string}", async function (notes, value) {
  const cell = await valueCellWhen(this.driver, notes, (c) => c.text === value);
  assert.equal(cell.text, value);
});

Then("the value of the transaction row with notes {string} should be based on {string}", async function (notes, basis) {
  const cell = await valueCellWhen(this.driver, notes, (c) => c.basis === basis);
  assert.equal(cell.basis, basis);
});

When("I click the value cell of the transaction row with notes {string}", async function (notes) {
  // Wait for the value to load first so the modal opens on the final basis.
  await valueCellWhen(this.driver, notes, (c) => c.text !== "—");
  await this.driver.executeScript(VALUE_CELL_SCRIPT, notes, true);
  await this.driver.wait(until.elementLocated(By.css('[data-testid="set-price-modal"]')), 10_000);
});

Then("the set-price modal should show the commodity {string} dated {string}", async function (commodity, date) {
  const modal = await this.driver.executeScript(MODAL_SCRIPT);
  assert.ok(modal, "the set-price modal is not open");
  assert.equal(modal.commodity, commodity);
  assert.equal(modal.date, date);
});

Then("the set-price modal should state the basis {string}", async function (basis) {
  const modal = await this.driver.executeScript(MODAL_SCRIPT);
  assert.ok(modal, "the set-price modal is not open");
  assert.equal(modal.basis, `Basis: ${basis}`);
});

When("I cancel the set-price modal", async function () {
  await this.driver.executeScript(`document.querySelector('#setPriceCancel')?.click();`);
  await this.driver.wait(async () => {
    const open = await this.driver.executeScript(`return !!document.querySelector('[data-testid="set-price-modal"]');`);
    return !open;
  }, 10_000, "the set-price modal did not close");
});
