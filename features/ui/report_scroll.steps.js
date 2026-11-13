const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { After, Given, When, Then } = require("@cucumber/cucumber");

// The shared selenium driver is owned by parse_display.steps.js (BeforeAll/
// AfterAll); steps use it via the World (this.driver). The account and
// Categories steps come from parse_display.steps.js and categories.steps.js,
// "I open the Balances report" from header_matches_balances_report.steps.js,
// the slow searches from search_typing.steps.js.

const repoRoot = path.resolve(__dirname, "..", "..");

const TALL_REPORT_STYLE = "e2e-tall-report";

// Long enough for the scroll event to reach the handler, not for a slowed
// search to finish.
const SCROLL_EVENT_MS = 200;

/** Move `.content` with a script and remember where it was put. The browser
 *  fires a real scroll event, the same one a wheel tick would. */
async function scrollPage(world, expression) {
  world.scrolledTo = await world.driver.executeScript(`
    const el = document.querySelector(".content");
    el.scrollTop = ${expression};
    return el.scrollTop;`);
  await world.driver.sleep(SCROLL_EVENT_MS);
}

After(async function () {
  await this.driver?.executeScript(`document.getElementById("${TALL_REPORT_STYLE}")?.remove();`);
});

Given(
  "the transactions file {string} holds {int} purchases on the account {string}",
  function (relativePath, count, account) {
    const day = (i) => new Date(Date.UTC(2024, 0, 1 + i)).toISOString().slice(0, 10);
    const entries = Array.from({ length: count }, (_, i) => {
      const n = String(i + 1).padStart(4, "0");
      return `${day(i)}T09:00:00Z * "Shop ${n}" "Card purchase ${n}" ; txn:busy-${n}
    ${account}   -1.00 AUD
    expenses:busy   1.00 AUD
`;
    });
    const file = path.join(repoRoot, relativePath);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, entries.join("\n"));
  },
);

// The report's real length depends on the data (a vault with many warnings
// runs long); a stylesheet outside #app survives the renders.
When("the report is long enough to scroll", async function () {
  await this.driver.executeScript(`
    const style = document.createElement("style");
    style.id = "${TALL_REPORT_STYLE}";
    style.textContent = ".reportView { min-height: 4000px; }";
    document.head.appendChild(style);`);
});

When("I scroll the page to the bottom", async function () {
  await scrollPage(this, "el.scrollHeight");
});

When("I scroll the page up by {int} pixels", async function (pixels) {
  await scrollPage(this, `el.scrollTop - ${pixels}`);
});

When("I scroll the page to {int} pixels", async function (pixels) {
  await scrollPage(this, String(pixels));
});

Then("the page stays where I scrolled it", async function () {
  const now = await this.driver.executeScript(`return document.querySelector(".content").scrollTop;`);
  assert.ok(Math.abs(now - this.scrolledTo) <= 2, `the page moved from ${this.scrolledTo} to ${now}`);
});

Then("the transaction list still holds {int} rows", async function (rows) {
  assert.equal(await this.driver.executeScript(`return window.__diag().searchFiltered;`), rows);
});

Then("the transaction table shows {int} rows", async function (rows) {
  let shown;
  await this.driver.wait(
    async () => {
      shown = await this.driver.executeScript(`return document.querySelectorAll('[data-testid="txn-row"]').length;`);
      return shown === rows;
    },
    15_000,
    () => `expected ${rows} rows in the table, saw ${shown}`,
  );
});
