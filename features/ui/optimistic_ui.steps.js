const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { After, Given, When, Then } = require("@cucumber/cucumber");

// Driver setup is owned by parse_display.steps.js BeforeAll/AfterAll; the
// World carries it (this.driver) plus the harness vault (this.sourcesDir).
//
// Each edit is made AND read back inside one executeScript: the click
// handlers render synchronously while the rebuild's IPC is still in flight,
// so what the script reads is what the table showed before the rebuild
// finished — no timing guesswork. `pending` (the mutation queue's depth)
// records that the rebuild really was still running.

const repoRoot = path.resolve(__dirname, "..", "..");

// Find a row by its raw payee (data-source-payee survives a rename).
const ROW_HELPERS = `
  const rowFor = (payee) => Array.from(document.querySelectorAll('[data-testid="txn-row"]')).find((r) => {
    const cell = r.querySelector('[data-testid="txn-payee"]');
    return cell && (cell.getAttribute('data-source-payee') === payee
      || (cell.querySelector('.copyable')?.textContent || '').trim() === payee);
  });
  const snapshot = (payee) => {
    const row = rowFor(payee);
    if (!row) return { present: false, pending: window.__diag().pendingMutations };
    const category = row.querySelector('td.category');
    return {
      present: true,
      category: category ? category.childNodes[0]?.textContent?.trim() ?? '' : '',
      party: (row.querySelector('[data-testid="txn-payee"] .copyable')?.textContent || '').trim(),
      uncategorised: row.classList.contains('txRow--uncategorised'),
      pending: window.__diag().pendingMutations,
    };
  };`;

const RECATEGORISE_SCRIPT = `${ROW_HELPERS}
  const [payee, value] = arguments;
  const row = rowFor(payee);
  if (!row) return { error: 'no row for ' + payee };
  const cell = row.querySelector('td[data-rule-type="category"]');
  cell.click();
  const input = cell.querySelector('.cell-edit-input');
  if (!input) return { error: 'category cell did not open an editor' };
  input.value = value;
  input.dispatchEvent(new Event('input', { bubbles: true }));
  input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
  return snapshot(payee);`;

const RENAME_SCRIPT = `${ROW_HELPERS}
  const [payee, value] = arguments;
  const row = rowFor(payee);
  if (!row) return { error: 'no row for ' + payee };
  const cell = row.querySelector('td[data-rule-type="payee"]');
  cell.click();
  const input = cell.querySelector('.cell-edit-input');
  if (!input) return { error: 'payee cell did not open an editor' };
  input.value = value;
  input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
  return snapshot(payee);`;

const ROW_BUTTON_SCRIPT = `${ROW_HELPERS}
  const [payee, selector] = arguments;
  const row = rowFor(payee);
  if (!row) return { error: 'no row for ' + payee };
  const btn = row.querySelector(selector);
  if (!btn) return { error: 'no ' + selector + ' on the row for ' + payee };
  btn.click();
  return snapshot(payee);`;

const SNAPSHOT_SCRIPT = `${ROW_HELPERS} return snapshot(arguments[0]);`;

async function act(world, script, ...args) {
  const result = await world.driver.executeScript(script, ...args);
  assert.ok(result && !result.error, result?.error ?? "edit script returned nothing");
  world.beforeRebuild = result;
}

function assertBeforeRebuild(world) {
  assert.ok(world.beforeRebuild, "no edit was made");
  assert.ok(
    world.beforeRebuild.pending > 0,
    "the rebuild had already finished when the row was read — the read proves nothing",
  );
  return world.beforeRebuild;
}

/** Wait for the mutation queue to drain, then for the landing render. */
async function afterRebuild(world, payee) {
  await world.driver.wait(
    async () => world.driver.executeScript("return window.__diag().pendingMutations === 0"),
    30_000,
    "the rebuild never finished",
  );
  await world.driver.sleep(300);
  return world.driver.executeScript(SNAPSHOT_SCRIPT, payee);
}

Given("the {string} fixture folders have no rules or labels", async function (fixture) {
  // Rules and labels written by an earlier scenario live beside the staged
  // fixture (the row's folder) and in its institution folder (the parent).
  const fixturePath = path.join(repoRoot, "src-tauri/features/fixtures", `${fixture}.transactions`);
  const anchor = /^\s+(assets:[\w:.-]+)\s/m.exec(fs.readFileSync(fixturePath, "utf8"));
  assert.ok(anchor, `fixture has no assets posting: ${fixturePath}`);
  const folder = path.join(this.sourcesDir, "e2e", ...anchor[1].replace(/^assets:/, "").split(":"));
  for (const dir of [folder, path.dirname(folder)]) {
    for (const file of ["_rules.json", "_labels.json"]) fs.rmSync(path.join(dir, file), { force: true });
  }
});

When("I re-categorise the transaction row for payee {string} to {string}", async function (payee, value) {
  await this.driver.executeScript(`
    const body = document.querySelector('.txTable tbody');
    window.__minTxnRows = document.querySelectorAll('[data-testid="txn-row"]').length;
    window.__rowsObserver?.disconnect();
    window.__rowsObserver = new MutationObserver(() => {
      const n = document.querySelectorAll('[data-testid="txn-row"]').length;
      window.__minTxnRows = Math.min(window.__minTxnRows, n);
    });
    if (body) window.__rowsObserver.observe(document.querySelector('#app') || body, { childList: true, subtree: true });`);
  await act(this, RECATEGORISE_SCRIPT, payee, value);
});

When("I rename the party on the transaction row for payee {string} to {string}", async function (payee, value) {
  await act(this, RENAME_SCRIPT, payee, value);
});

When("I hide the transaction row for payee {string}", async function (payee) {
  await act(this, ROW_BUTTON_SCRIPT, payee, '[data-testid="txn-delete"]');
});

When("I unhide the transaction row for payee {string}", async function (payee) {
  await this.driver.wait(
    async () => (await this.driver.executeScript(SNAPSHOT_SCRIPT, payee)).present,
    10_000,
    `no hidden row for payee "${payee}"`,
  );
  await act(this, ROW_BUTTON_SCRIPT, payee, '[data-testid="txn-unhide"]');
});

// The marker is a property on the <tr> node, not an attribute: morphdom
// syncs attributes to the new markup, but a property survives only if the
// very same node is kept — which is what "updated in place" means.
When("I tag the transaction row for payee {string} with marker {string}", async function (payee, marker) {
  const tagged = await this.driver.executeScript(`${ROW_HELPERS}
    const row = rowFor(arguments[0]);
    if (row) row.__e2eMarker = arguments[1];
    return !!row;`, payee, marker);
  assert.ok(tagged, `no row for payee "${payee}"`);
});

When("I open the hidden transactions with Show Ignored", async function () {
  await this.driver.executeScript(`
    const btn = document.querySelector('#showHiddenBtn');
    if (btn && !btn.classList.contains('actionBtn--active')) btn.click();`);
  await this.driver.wait(
    async () => this.driver.executeScript(
      "return document.querySelector('#showHiddenBtn')?.classList.contains('actionBtn--active') && !document.querySelector('#showHiddenBtn').disabled",
    ),
    15_000,
    "Show Ignored never turned on",
  );
  // Categories pane → the ignore root → ignore:hidden. Drill one level per
  // render (a drill click re-renders the list).
  await this.driver.executeScript(`document.querySelector('.sidebarNav__item[data-view="categories"]')?.click();`);
  await this.driver.wait(async () => this.driver.executeScript(`
    const leaf = document.querySelector('[data-testid="account-item"][data-account="ignore:hidden"]');
    if (leaf) { leaf.click(); return true; }
    const back = document.querySelector('[data-testid="drilldown-back"]');
    const group = document.querySelector('[data-testid="drilldown-item"][data-group="ignore"]');
    if (group) group.click();
    else if (back) back.click();
    return false;`), 15_000, "never reached the ignore:hidden category");
  await this.driver.wait(
    async () => this.driver.executeScript(
      "return (document.querySelector('[data-testid=\"selected-account\"]')?.textContent || '').includes('hidden')",
    ),
    10_000,
    "ignore:hidden never became the selection",
  );
});

When("I turn off Show Ignored in the Accounts view", async function () {
  await this.driver.executeScript(`document.querySelector('.sidebarNav__item[data-view="accounts"]')?.click();`);
  await this.driver.executeScript(`
    const btn = document.querySelector('#showHiddenBtn');
    if (btn && btn.classList.contains('actionBtn--active')) btn.click();`);
  await this.driver.wait(
    async () => this.driver.executeScript(
      "const b = document.querySelector('#showHiddenBtn'); return !!b && !b.classList.contains('actionBtn--active') && !b.disabled",
    ),
    15_000,
    "Show Ignored never turned off",
  );
});

// Show Ignored changes no rules, so there is no rebuild to wait for: the
// toggle's re-query is in flight while __diag().showIgnoredInFlight > 0. The
// click and the read-back share one executeScript, so what it reads is the
// frame the click rendered. Element.animate is wrapped to record which rows
// faded (playRowFlashes animates every cell of a row it flashes).
const TOGGLE_SCRIPT = `
  const want = arguments[0];
  const btn = document.querySelector('#showHiddenBtn');
  if (!btn) return { error: 'no Show Ignored button' };
  if (btn.classList.contains('actionBtn--active') === want) return { error: 'Show Ignored was already ' + (want ? 'on' : 'off') };
  if (!window.__e2eAnimateWrapped) {
    const animate = Element.prototype.animate;
    Element.prototype.animate = function (...args) {
      const row = this.closest && this.closest('tr[data-expand-key]');
      if (row && window.__fadedRows) window.__fadedRows.push(row.getAttribute('data-expand-key'));
      return animate.apply(this, args);
    };
    window.__e2eAnimateWrapped = true;
  }
  window.__fadedRows = [];
  btn.click();
  const payees = Array.from(document.querySelectorAll('[data-testid="txn-row"]')).map((r) => {
    const cell = r.querySelector('[data-testid="txn-payee"]');
    return cell?.getAttribute('data-source-payee') || (cell?.querySelector('.copyable')?.textContent || '').trim();
  });
  return { payees, inFlight: window.__diag().showIgnoredInFlight };`;

async function toggleShowIgnored(world, on) {
  const result = await world.driver.executeScript(TOGGLE_SCRIPT, on);
  assert.ok(result && !result.error, result?.error ?? "toggle script returned nothing");
  world.afterClick = result;
}

/** Wait for the toggle's re-query to land and render. */
async function afterShowIgnored(world, payee) {
  await world.driver.wait(
    async () => world.driver.executeScript("return window.__diag().showIgnoredInFlight === 0"),
    15_000,
    "the Show Ignored re-query never landed",
  );
  await world.driver.sleep(100);
  return world.driver.executeScript(SNAPSHOT_SCRIPT, payee);
}

async function rowFaded(world, payee) {
  return world.driver.executeScript(`${ROW_HELPERS}
    const key = rowFor(arguments[0])?.getAttribute('data-expand-key');
    return !!key && (window.__fadedRows || []).includes(key);`, payee);
}

// A set config lists the accounts Show Ignored reveals (`hidden_accounts`);
// the harness vault's has none, so nothing would be left out. Put back after
// the scenario: the harness vault outlives it.
Given("the data folder leaves out the {string} accounts unless Show Ignored is on", function (prefix) {
  const configPath = path.join(this.generatedDir, "e2e", "config.json");
  const original = fs.existsSync(configPath) ? fs.readFileSync(configPath, "utf8") : null;
  fs.mkdirSync(path.dirname(configPath), { recursive: true });
  fs.writeFileSync(configPath, JSON.stringify({ ...JSON.parse(original ?? "{}"), hidden_accounts: [prefix] }, null, 2));
  this.restoreSetConfig = () => {
    if (original === null) fs.rmSync(configPath, { force: true });
    else fs.writeFileSync(configPath, original);
  };
});

After(function () {
  this.restoreSetConfig?.();
});

Given("I hid the transaction row for payee {string}", async function (payee) {
  await act(this, ROW_BUTTON_SCRIPT, payee, '[data-testid="txn-delete"]');
  const snap = await afterRebuild(this, payee);
  assert.equal(snap.present, false, `the row for payee "${payee}" was not hidden`);
});

When("I turn on Show Ignored", async function () {
  await toggleShowIgnored(this, true);
});

When("I turn off Show Ignored", async function () {
  await toggleShowIgnored(this, false);
});

Then("once its rows land the row for payee {string} fades in showing category {string}", async function (payee, category) {
  const snap = await afterShowIgnored(this, payee);
  assert.ok(snap.present && snap.category === category, `after the re-query the row showed ${JSON.stringify(snap)}`);
  assert.ok(await rowFaded(this, payee), `the row for payee "${payee}" appeared without fading in`);
});

Then("the row for payee {string} did not fade", async function (payee) {
  assert.equal(await rowFaded(this, payee), false, `the row for payee "${payee}" faded though it did not change`);
});

Then("the row for payee {string} was gone in the same frame as the click", function (payee) {
  const { payees, inFlight } = this.afterClick;
  assert.ok(inFlight > 0, "the re-query had already landed when the rows were read — the read proves nothing");
  assert.ok(!payees.includes(payee), `the click rendered ${JSON.stringify(payees)}`);
  assert.ok(payees.length > 0, "the click emptied the table");
});

Then("once its rows land there is no row for payee {string}", async function (payee) {
  const snap = await afterShowIgnored(this, payee);
  assert.equal(snap.present, false, `the row came back after the re-query: ${JSON.stringify(snap)}`);
});

Then("before the rebuild finished the row for payee {string} showed category {string}", function (_payee, category) {
  assert.equal(assertBeforeRebuild(this).category, category);
});

Then("before the rebuild finished the row for payee {string} was no longer marked uncategorised", function (_payee) {
  assert.equal(assertBeforeRebuild(this).uncategorised, false);
});

Then("before the rebuild finished the row for payee {string} showed party {string}", function (_payee, party) {
  assert.equal(assertBeforeRebuild(this).party, party);
});

Then("before the rebuild finished the row for payee {string} was gone", function (_payee) {
  assert.equal(assertBeforeRebuild(this).present, false);
});

Then("once the rebuild lands the row for payee {string} shows category {string}", async function (payee, category) {
  let snap;
  await this.driver.wait(async () => {
    snap = await afterRebuild(this, payee);
    return snap.present && snap.category === category;
  }, 30_000).catch(() => assert.fail(`after the rebuild the row showed ${JSON.stringify(snap)}`));
});

Then("once the rebuild lands the row for payee {string} shows party {string}", async function (payee, party) {
  const snap = await afterRebuild(this, payee);
  assert.equal(snap.party, party, `after the rebuild the row showed ${JSON.stringify(snap)}`);
});

Then("once the rebuild lands there is no row for payee {string}", async function (payee) {
  const snap = await afterRebuild(this, payee);
  assert.equal(snap.present, false, `the row came back after the rebuild: ${JSON.stringify(snap)}`);
});

Then("the transaction table never emptied while the rebuild landed", async function () {
  const min = await this.driver.executeScript("window.__rowsObserver?.disconnect(); return window.__minTxnRows;");
  assert.ok(min > 0, `the table dropped to ${min} rows while the rebuild landed`);
});

Then("the transaction row for payee {string} still has marker {string}", async function (payee, marker) {
  const actual = await this.driver.executeScript(`${ROW_HELPERS}
    return rowFor(arguments[0])?.__e2eMarker ?? null;`, payee);
  assert.equal(actual, marker, "the row was replaced, not updated in place");
});

Then("the transaction row for payee {string} is still expanded", async function (payee) {
  const expanded = await this.driver.executeScript(`${ROW_HELPERS}
    const row = rowFor(arguments[0]);
    const key = row?.getAttribute('data-expand-key');
    return !!key && !!document.querySelector('[data-txn-detail-for="' + CSS.escape(key) + '"]');`, payee);
  assert.ok(expanded, `the row for payee "${payee}" lost its expanded detail`);
});
