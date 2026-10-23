const assert = require("node:assert/strict");
const { When, Then } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");

// Driver setup is owned by parse_display.steps.js BeforeAll/AfterAll.
// The shared step 'I click {string} in the row detail' (edit_rule.steps.js)
// dispatches "Edit Party" to .txDetail__partyBtn and waits for the editor.

Then(
  "the party editor pattern should contain {string}",
  async function (expected) {
    // Live property via script, re-queried inside it — tauri-wd cannot
    // marshal WebElement handles into executeScript arguments, and
    // selenium's getAttribute/getDomAttribute can't read input values here.
    let value = null;
    await this.driver.wait(async () => {
      value = await this.driver.executeScript(
        "const el = document.querySelector('#labelPattern'); return el ? el.value : null;",
      );
      return typeof value === "string" && value.includes(expected);
    }, 5_000, () => `Expected party pattern to contain "${expected}" but got "${value}"`);
  },
);

// Match-count text is "N match(es), M renamed by earlier labels" — wait on the
// leading integer so the 300ms-debounced preview recompute has time to land.
async function waitForMatchCount(driver, expected) {
  let last = "";
  await driver.wait(async () => {
    const el = await driver.findElement(By.css('[data-testid="party-match-count"]'));
    last = (await el.getText()).trim();
    return parseInt(last, 10) === expected;
  }, 5_000, () => `expected match count ${expected}, saw "${last}"`);
}

Then(
  "the party editor should report {int} matching transaction",
  async function (count) {
    await waitForMatchCount(this.driver, count);
  },
);

Then(
  "the party editor should report {int} matching transactions",
  async function (count) {
    await waitForMatchCount(this.driver, count);
  },
);

Then(
  "the party editor should show the existing labels section",
  async function () {
    const list = await this.driver.findElements(By.css('[data-testid="party-label-list"]'));
    assert.ok(list.length === 1, "party label list section not found");
  },
);

// Set the input's value property directly and fire `input` — sendKeys into a
// prefilled input would append, and clear() loses focus mid-edit in WKWebView.
// The element is re-queried inside the script: tauri-wd cannot marshal
// WebElement handles into executeScript arguments.
async function setPartyEditorInput(driver, inputId, value) {
  await driver.executeScript(
    `const el = document.querySelector('#' + arguments[0]);
     el.value = arguments[1];
     el.dispatchEvent(new Event("input", { bubbles: true }));`,
    inputId,
    value,
  );
}

When(
  "I replace the party editor pattern with {string}",
  async function (pattern) {
    await setPartyEditorInput(this.driver, "labelPattern", pattern);
  },
);

When(
  "I set the party name to {string}",
  async function (name) {
    await setPartyEditorInput(this.driver, "labelParty", name);
  },
);

When("I apply the party label", async function () {
  const btn = await this.driver.findElement(By.css("#labelSave"));
  await btn.click();
});

Then("the party editor should be closed", async function () {
  await this.driver.wait(async () => {
    const editors = await this.driver.findElements(By.css('[data-testid="party-editor"]'));
    return editors.length === 0;
  }, 10_000, "party editor did not close after apply");
});

// The rename lands after the save's pipeline rebuild refreshes the rows —
// wait for a Party cell whose display text is exactly the new name. Read
// via executeScript: post-save renders stale element handles, and
// selenium's getText atom is unreliable in current WKWebView.
Then("a transaction row should show the party {string}", async function (party) {
  let seen = [];
  await this.driver.wait(async () => {
    seen = await this.driver.executeScript(
      `return Array.from(document.querySelectorAll('[data-testid="txn-payee"]'))
         .map(c => (c.innerText || '').trim().split('\\n')[0]);`,
    );
    return Array.isArray(seen) && seen.includes(party);
  }, 30_000, () => `no Party cell showing "${party}" after the rebuild — saw ${JSON.stringify(seen)}`);
});
