const assert = require("node:assert/strict");
const { When, Then } = require("@cucumber/cucumber");
const { By } = require("selenium-webdriver");

// Driver setup is owned by parse_display.steps.js BeforeAll/AfterAll.

When(
  "I tag the first transaction row with marker {string}",
  async function (marker) {
    // Add a custom data attribute via the DevTools-equivalent script
    // execution. If morphdom preserves the row's identity across the
    // next render, this attribute will still be present afterward.
    await this.driver.executeScript(`
      const row = document.querySelector('[data-testid="txn-row"]');
      if (!row) throw new Error("no txn row to tag");
      row.setAttribute("data-architectural-marker", arguments[0]);
    `, marker);
  },
);

When(
  "I trigger a render by clearing then restoring the search input",
  async function () {
    // Tickle a no-op state change by toggling the search input's value
    // — enough to cause buildMainHtml to re-run and morphdom to walk
    // the existing DOM. The eventual DOM should be byte-identical to
    // what was there before, so morphdom should make zero structural
    // changes to the txn rows.
    await this.driver.executeScript(`
      const search = document.querySelector('#accountSearchInput');
      if (!search) return;
      const orig = search.value;
      search.value = orig + ' ';
      search.dispatchEvent(new Event('input', { bubbles: true }));
      search.value = orig;
      search.dispatchEvent(new Event('input', { bubbles: true }));
    `);
    // Give the renderer a beat to settle.
    await this.driver.sleep(250);
  },
);

Then(
  "the first transaction row still has marker {string}",
  async function (expected) {
    const actual = await this.driver.executeScript(`
      const row = document.querySelector('[data-testid="txn-row"]');
      return row ? row.getAttribute("data-architectural-marker") : null;
    `);
    assert.equal(
      actual,
      expected,
      `Expected the row's marker to survive the render — morphdom should ` +
      `have preserved this DOM node's identity. Got ${JSON.stringify(actual)}.`,
    );
  },
);

When(
  "I start an inline edit on the first payee cell and type {string}",
  async function (text) {
    const cell = await this.driver.findElement(
      By.css('td.cell-clickable[data-rule-type="payee"]'),
    );
    await cell.click();
    // startCellEdit injects the input synchronously, but give the
    // WebView a beat before typing into it.
    const deadline = Date.now() + 3000;
    let input = null;
    while (Date.now() < deadline) {
      const found = await this.driver.findElements(By.css(".cell-edit-input"));
      if (found.length > 0) { input = found[0]; break; }
      await this.driver.sleep(100);
    }
    if (!input) throw new Error("inline cell editor never appeared after clicking the payee cell");
    await input.sendKeys(text);
  },
);

Then(
  "the inline cell editor still contains {string}",
  async function (expected) {
    const actual = await this.driver.executeScript(`
      const input = document.querySelector('.cell-edit-input');
      return input ? input.value : null;
    `);
    assert.equal(
      actual,
      expected,
      `Expected the in-progress inline edit to survive the background ` +
      `render (data-cell-editing morphdom guard). Got ${JSON.stringify(actual)}.`,
    );
  },
);
