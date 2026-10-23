const { Then } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");
const assert = require("node:assert/strict");

// Session setup is owned by parse_display.steps.js BeforeAll/AfterAll and
// "I open the global settings dialog" by global_settings.steps.js. These
// steps consume the driver via the cucumber-js World (this.driver).
//
// Presence-only, never Save: the e2e app runs against the real vault config,
// so persisting a term override would mutate config.json.

Then("I should see the terminology list", async function () {
  await this.driver.wait(
    until.elementLocated(By.css('[data-testid="terminology-list"]')),
    10_000,
  );
});

Then(
  "the term {string} should offer the defaults {string} and {string}",
  async function (key, singular, plural) {
    await this.driver.wait(
      until.elementLocated(By.css(`[data-testid="term-singular-${key}"]`)),
      10_000,
    );
    // Read attributes in-page: tauri-wd cannot marshal element handles back
    // through selenium's attribute atoms (see parse_display.steps.js).
    const placeholders = await this.driver.executeScript(
      `const q = (id) => document.querySelector('[data-testid="' + id + '"]')?.getAttribute("placeholder") ?? null;
       return [q("term-singular-" + arguments[0]), q("term-plural-" + arguments[0])];`,
      key,
    );
    assert.deepEqual(placeholders, [singular, plural]);
  },
);
