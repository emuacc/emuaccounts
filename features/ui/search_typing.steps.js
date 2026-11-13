const assert = require("node:assert/strict");
const { After, When, Then } = require("@cucumber/cucumber");

// The shared selenium driver is owned by parse_display.steps.js (BeforeAll/
// AfterAll); steps use it via the World (this.driver). "the app is running",
// "I parse the transactions file ..." and "I select the account ..." come
// from parse_display.steps.js.
//
// Everything here runs inside the page: tauri-wd's sendKeys appends to the
// value instead of typing at the caret, so it can't show a caret that jumped.
// Each key is inserted at the caret with setRangeText and announced with an
// `input` event, which is what the browser does for a typed character.

const SEARCH_INPUT = "#accountSearchInput";

/** How long the bar must see no search running before the searches count as
 *  finished: longer than the search debounce, so a search a late key
 *  scheduled has started by then. */
const SETTLED_MS = 1_000;

// Tauri's IPC transport calls the global fetch for every command, so wrapping
// fetch in the page slows the transaction searches (query_search, and
// query_global for the Categories pane) without touching the app. Tauri
// freezes __TAURI_INTERNALS__.invoke itself, so it can't be wrapped.
const SLOW_SEARCHES = `
  const delayMs = arguments[0];
  if (!window.__e2eRealFetch) window.__e2eRealFetch = window.fetch;
  const realFetch = window.__e2eRealFetch;
  const searches = { inFlight: 0 };
  window.__e2eSearches = searches;
  window.fetch = function (input, init) {
    const url = String(input instanceof Request ? input.url : input);
    if (!/\\/(query_search|query_global)(\\?|$)/.test(url)) return realFetch.call(window, input, init);
    searches.inFlight += 1;
    return new Promise((resolve) => setTimeout(resolve, delayMs))
      .then(() => realFetch.call(window, input, init))
      .finally(() => { searches.inFlight -= 1; });
  };`;

const RESTORE_SEARCHES = `
  if (window.__e2eRealFetch) {
    window.fetch = window.__e2eRealFetch;
    delete window.__e2eRealFetch;
  }
  delete window.__e2eSearches;`;

/** Clear the bar (pills and text) the way a user would: select-all + delete.
 *  arguments[1]: a missing bar is fine (the scenario left the accounts pane). */
const CLEAR_SEARCH = `
  const el = document.querySelector(arguments[0]);
  if (!el) return arguments[1] ? null : "no search bar";
  // Each removal rebuilds the pills, so click the first one until none is left.
  for (let i = 0; i < 20; i++) {
    const remove = document.querySelector("#accountSearch [data-pill-remove]");
    if (!remove) break;
    remove.click();
  }
  el.focus();
  el.value = "";
  el.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "deleteContentBackward" }));
  el.blur();
  return null;`;

/** Type one character at the caret of the focused element. A key goes where
 *  the focus is, so a search bar that lost focus fails here. */
const TYPE_KEY = `
  const el = document.querySelector(arguments[0]);
  if (!el) return "no search bar";
  if (document.activeElement !== el) return "the search bar lost focus (active: " + (document.activeElement?.id || document.activeElement?.tagName) + ")";
  el.setRangeText(arguments[1], el.selectionStart, el.selectionEnd, "end");
  el.dispatchEvent(new InputEvent("input", { bubbles: true, inputType: "insertText", data: arguments[1] }));
  return null;`;

const READ_BAR = `
  const el = document.querySelector(arguments[0]);
  return el && {
    value: el.value,
    start: el.selectionStart,
    end: el.selectionEnd,
    focused: document.activeElement === el,
  };`;

async function inPage(driver, script, ...args) {
  const problem = await driver.executeScript(script, ...args);
  assert.equal(problem, null, problem);
}

After(async function () {
  if (!this.slowSearches) return;
  await this.driver.executeScript(RESTORE_SEARCHES);
  await inPage(this.driver, CLEAR_SEARCH, SEARCH_INPUT, true);
});

When("the search bar is empty", async function () {
  await inPage(this.driver, CLEAR_SEARCH, SEARCH_INPUT, false);
  // Let the search the clear scheduled run before the scenario slows searches.
  await this.driver.sleep(SETTLED_MS);
});

When("every transaction search takes {int} ms", async function (delayMs) {
  this.slowSearches = true;
  await this.driver.executeScript(SLOW_SEARCHES, delayMs);
});

When("I type {string} into the search bar with {int} ms between keys", async function (text, gapMs) {
  await this.driver.executeScript(
    `const el = document.querySelector(arguments[0]);
     el.focus();
     el.setSelectionRange(el.value.length, el.value.length);`,
    SEARCH_INPUT,
  );
  for (const [i, ch] of [...text].entries()) {
    if (i > 0) await this.driver.sleep(gapMs);
    await inPage(this.driver, TYPE_KEY, SEARCH_INPUT, ch);
  }
});

When("I move the caret to {int} while a search is running", async function (pos) {
  await this.driver.wait(
    async () => (await this.driver.executeScript("return window.__e2eSearches.inFlight")) > 0,
    5_000,
    "no search was running to move the caret during",
  );
  await this.driver.executeScript(
    `document.querySelector(arguments[0]).setSelectionRange(arguments[1], arguments[1]);`,
    SEARCH_INPUT,
    pos,
  );
});

When("the searches finish", async function () {
  let quietSince = Date.now();
  await this.driver.wait(
    async () => {
      const inFlight = await this.driver.executeScript("return window.__e2eSearches.inFlight");
      if (inFlight > 0) quietSince = Date.now();
      return Date.now() - quietSince >= SETTLED_MS;
    },
    20_000,
    "the searches never finished",
  );
});

Then("the search bar shows {string} with the caret at {int}", async function (text, pos) {
  const bar = await this.driver.executeScript(READ_BAR, SEARCH_INPUT);
  assert.deepEqual(bar, { value: text, start: pos, end: pos, focused: true });
});

Then("the transaction table shows only the payee {string}", async function (payee) {
  let payees = [];
  await this.driver.wait(
    async () => {
      payees = await this.driver.executeScript(
        `return Array.from(document.querySelectorAll('[data-testid="txn-row"] [data-testid="txn-payee"]'))
           .map((c) => c.textContent.trim());`,
      );
      return payees.length > 0 && payees.every((p) => p === payee);
    },
    10_000,
    () => `expected only "${payee}" rows, saw ${JSON.stringify(payees)}`,
  );
});
