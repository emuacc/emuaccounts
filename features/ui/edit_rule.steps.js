const assert = require("node:assert/strict");
const { When, Then } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");

// Driver setup is owned by parse_display.steps.js BeforeAll/AfterAll.
// Steps consume it via cucumber-js World (this.driver), matching the
// convention used by hide_transaction.steps.js / group_trade_link.steps.js.

// Locate the payee's row and (optionally) click its date cell — an inert
// area with no cell-clickable handler — inside ONE script: background
// renders stale element handles, and tauri-wd can't marshal them into
// executeScript anyway. `click: false` only reports whether the row exists.
const FIND_PAYEE_ROW_SCRIPT = `
  const rows = Array.from(document.querySelectorAll('[data-testid="txn-row"]'));
  const row = rows.find(r => {
    const cell = r.querySelector('[data-testid="txn-payee"]');
    return cell && ((cell.textContent || '').includes(arguments[0])
      || (cell.getAttribute('data-source-payee') || '').includes(arguments[0]));
  });
  if (!row) return false;
  if (arguments[1]) row.querySelector('td:first-child').click();
  return true;`;

When(
  "I expand the transaction row for payee {string}",
  async function (payee) {
    await this.driver.wait(
      async () => this.driver.executeScript(FIND_PAYEE_ROW_SCRIPT, payee, false),
      10_000,
      `No transaction row found for payee "${payee}"`,
    );
    await this.driver.executeScript(FIND_PAYEE_ROW_SCRIPT, payee, true);
    await this.driver.wait(
      until.elementLocated(By.css(".txDetail__ruleBtn")),
      5_000,
    );
  },
);

// Detail-row buttons and the editor each mounts (rule editor for
// sidebarView "rule-editor", party editor for "label-editor").
const DETAIL_BUTTONS = {
  "Edit Rule": { button: ".txDetail__ruleBtn", editor: ".ruleEditorBar" },
  "Edit Party": { button: ".txDetail__partyBtn", editor: '[data-testid="party-editor"]' },
};

When(
  'I click {string} in the row detail',
  async function (label) {
    const target = DETAIL_BUTTONS[label];
    assert.ok(target, `Unknown detail button "${label}" — add it to DETAIL_BUTTONS`);
    const btn = await this.driver.findElement(By.css(target.button));
    await btn.click();
    await this.driver.wait(
      until.elementLocated(By.css(target.editor)),
      5_000,
    );
  },
);

Then(
  "the rule editor pattern should contain {string}",
  async function (expected) {
    // Live property via script, not selenium's getAttribute: its atom calls
    // Element.getAttributeNode (gone from current WKWebView), and
    // getDomAttribute("value") would read the stale attribute, not the input's
    // current value. Re-query inside the script and poll — the editor
    // re-renders right after mounting, so a passed element handle can go
    // stale and read as null.
    let value = null;
    await this.driver.wait(async () => {
      value = await this.driver.executeScript(
        "const el = document.querySelector('#ruleSearchInput'); return el ? el.value : null;",
      );
      return typeof value === "string" && value.includes(expected);
    }, 5_000, () => `Expected rule pattern to contain "${expected}" but got "${value}"`);
  },
);

// Exact pill-text match. A loose `text.includes(expected)` match silently
// accepted both `payee:Kraken` and `payee:*Kraken*` as identical, which let
// a wildcard-stripping bug ship: the editor displayed `payee:Orca` for a
// stored `*Orca*` substring rule, hiding why it matched "Francium lyfOrca".
// Polls: a freshly-typed pill forms asynchronously, and the editor's seeded
// pills would otherwise satisfy a bare "any pill exists" wait too early.
Then(
  "the rule editor should show the pill {string}",
  async function (expected) {
    let seen = [];
    await this.driver.wait(async () => {
      seen = await this.driver.executeScript(
        `return Array.from(document.querySelectorAll('#ruleSearch .smartSearch__pillLabel'))
           .map(p => (p.textContent || '').trim());`,
      );
      return Array.isArray(seen) && seen.includes(expected);
    }, 5_000, () => `Expected pill "${expected}". Pills seen: ${JSON.stringify(seen)}`);
  },
);

// Click the Nth visible txn row by position. Clicking the date cell —
// an inert region of the row — avoids the `.cell-clickable` and
// `.commodity-clickable` handlers that would otherwise swallow the click.
When(
  "I click the second transaction row",
  async function () {
    // Wait for the rows, then click in one script — element handles go
    // stale under background renders and can't cross into executeScript.
    await this.driver.wait(
      async () => (await this.driver.executeScript(
        `return document.querySelectorAll('[data-testid="txn-row"]').length;`,
      )) >= 2,
      10_000,
      "expected at least 2 txn rows",
    );
    await this.driver.executeScript(
      `const rows = document.querySelectorAll('[data-testid="txn-row"]');
       rows[1].querySelector('td:first-child').click();`,
    );
    // Wait for either: a detail row to appear, or the render to settle. The
    // detail render is async (toggleTxnExpand fetches account config first),
    // so a fixed sleep is unreliable.
    await this.driver.wait(async () => {
      const details = await this.driver.findElements(By.css(".txRow__detail"));
      return details.length > 0;
    }, 5_000, "no detail row appeared after clicking row 2");
  },
);

Then(
  "exactly one transaction detail row should be visible",
  async function () {
    const details = await this.driver.findElements(By.css(".txRow__detail"));
    assert.equal(
      details.length,
      1,
      `expected exactly 1 detail row, got ${details.length} — ` +
      `clicking one row expanded its twin via shared data-expand-key`,
    );
  },
);

Then(
  "the detail row should follow the second transaction row",
  async function () {
    const result = await this.driver.executeScript(`
      const rows = document.querySelectorAll('[data-testid="txn-row"]');
      if (rows.length < 2) return { ok: false, reason: 'fewer than 2 rows', count: rows.length };
      const target = rows[1];
      const next = target.nextElementSibling;
      if (!next || !next.classList.contains('txRow__detail')) {
        return {
          ok: false,
          reason: 'next sibling is not a detail row',
          nextTag: next ? next.tagName : null,
          nextClass: next ? next.className : null,
        };
      }
      const expandKey = target.getAttribute('data-expand-key');
      const detailFor = next.getAttribute('data-txn-detail-for');
      if (expandKey !== detailFor) {
        return { ok: false, reason: 'detail key mismatch', expandKey, detailFor };
      }
      return { ok: true };
    `);
    assert.ok(
      result.ok,
      `Detail row should immediately follow the clicked row: ${JSON.stringify(result)}`,
    );
  },
);

Then(
  "the detail row should contain an {string} button",
  async function (label) {
    const target = DETAIL_BUTTONS[label];
    assert.ok(target, `Unknown detail button "${label}" — add it to DETAIL_BUTTONS`);
    const buttons = await this.driver.findElements(By.css(`.txRow__detail ${target.button}`));
    assert.ok(buttons.length >= 1, `no "${label}" button found in detail row`);
    const text = (await buttons[0].getText()).trim();
    assert.equal(text, label);
  },
);

// The smart-search input parses pills when the typed text ends with a space,
// so callers include the trailing space in the string they type.
When(
  "I type {string} into the rule editor pattern",
  async function (text) {
    const input = await this.driver.findElement(By.css("#ruleSearchInput"));
    await input.click();
    // The cursor lands at the end of the seeded pattern — separate the typed
    // text with a space or the keyword fuses with the last seeded token
    // ("Sell BTCcommodity:...") and never tokenizes into a pill.
    const current = await this.driver.executeScript(
      "const el = document.querySelector('#ruleSearchInput'); return el ? el.value : '';",
    );
    await input.sendKeys((current && !current.endsWith(" ") ? " " : "") + text);
    await this.driver.wait(async () => {
      const pills = await this.driver.findElements(
        By.css("#ruleSearch .smartSearch__pillLabel"),
      );
      return pills.length > 0;
    }, 5_000, `no pill formed after typing "${text}"`);
  },
);
