const assert = require("node:assert/strict");
const { When, Then } = require("@cucumber/cucumber");

// The shared selenium driver is owned by parse_display.steps.js; steps use it
// via the World (this.driver). "the app is running" and "I parse the
// transactions file ..." come from there.

/** Each report's menu key and the backend command that generates it for a year. */
const REPORTS = {
  "Capital Gains Tax": { key: "cgt", command: "generate_cgt_report_cmd" },
  Balances: { key: "balances", command: "generate_balances_report_cmd" },
};

function report(name) {
  const r = REPORTS[name];
  assert.ok(r, `no report called "${name}" in REPORTS`);
  return r;
}

// Tauri's IPC transport calls the global fetch for every command (see
// search_typing.steps.js), so wrapping fetch in the page holds a report's
// command back without touching the app. window.__e2eHeld counts, per held
// command, the calls sent and the answers that have come back.
const HOLD_BACK = `
  const [command, delayMs] = arguments;
  if (!window.__e2eHeld) {
    const realFetch = window.fetch;
    const held = {};
    window.__e2eHeld = held;
    window.fetch = function (input, init) {
      const url = String(input instanceof Request ? input.url : input);
      const hold = held[url.split("?")[0].split("/").pop()];
      if (!hold) return realFetch.call(window, input, init);
      hold.sent += 1;
      return new Promise((resolve) => setTimeout(resolve, hold.delayMs))
        .then(() => realFetch.call(window, input, init))
        .finally(() => { hold.answered += 1; });
    };
  }
  window.__e2eHeld[command] = { delayMs, sent: 0, answered: 0 };`;

/** What the report pane shows: the active menu item, the report's title and
 *  whether the loading screen is up. */
const READ_PAGE = `
  return {
    menu: document.querySelector(".reportMenu__item--active")?.textContent.trim() ?? null,
    title: document.querySelector(".reportHeader__title")?.textContent.trim() ?? null,
    generating: !!document.querySelector(".reportView__empty")?.textContent.includes("Generating report"),
  };`;

/** How long a page must stay as it is after an answer: long enough for a
 *  late answer to have reached the page if it was going to. */
const SETTLED_MS = 1_500;

function heldCount(driver, name, field) {
  return driver.executeScript(
    `return window.__e2eHeld?.[arguments[0]]?.[arguments[1]] ?? 0;`,
    report(name).command,
    field,
  );
}

async function waitForAnswer(driver, name) {
  await driver.wait(
    async () => (await heldCount(driver, name, "answered")) > 0,
    30_000,
    `the ${name} report never answered`,
  );
}

/** Check the page every 100ms for SETTLED_MS; fail on the first look that
 *  isn't `ok`. */
async function staysFor(driver, ok, describe) {
  const deadline = Date.now() + SETTLED_MS;
  while (Date.now() < deadline) {
    const page = await driver.executeScript(READ_PAGE);
    assert.ok(ok(page), `${describe}, but the page showed ${JSON.stringify(page)}`);
    await driver.sleep(100);
  }
}

function shows(name) {
  return (page) => page.menu === name && !page.generating && page.title?.includes(`${name} - `);
}

When("the {string} report takes {int} seconds to generate", async function (name, seconds) {
  await this.driver.executeScript(HOLD_BACK, report(name).command, seconds * 1000);
});

When(
  "I choose the {string} report while the {string} report is loading",
  async function (chosen, loading) {
    await this.driver.executeScript(`document.querySelector('[data-view="reports"]')?.click();`);
    await this.driver.wait(
      async () => (await heldCount(this.driver, loading, "sent")) > 0,
      15_000,
      `Reports never asked for the ${loading} report`,
    );
    const problem = await this.driver.executeScript(
      `const hold = window.__e2eHeld[arguments[0]];
       if (hold.answered > 0) return "the report it was loading has already answered";
       const item = document.querySelector('[data-report="' + arguments[1] + '"]');
       if (!item) return "no menu item for the chosen report";
       item.click();
       return null;`,
      report(loading).command,
      report(chosen).key,
    );
    assert.equal(problem, null);
  },
);

Then(
  "the page shows the {string} report once the {string} report has answered",
  async function (shown, answered) {
    await waitForAnswer(this.driver, answered);
    const ok = shows(shown);
    let page;
    await this.driver.wait(
      async () => ok((page = await this.driver.executeScript(READ_PAGE))),
      10_000,
      () => `the page never showed the ${shown} report; last saw ${JSON.stringify(page)}`,
    );
    await staysFor(this.driver, ok, `the page should have stayed on the ${shown} report`);
  },
);

Then(
  "the page still says the report is generating once the {string} report has answered",
  async function (answered) {
    await waitForAnswer(this.driver, answered);
    await staysFor(
      this.driver,
      (page) => page.generating,
      "the page should have stayed on its loading screen",
    );
  },
);
