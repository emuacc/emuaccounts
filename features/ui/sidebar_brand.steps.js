const { Then } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");
const assert = require("node:assert/strict");

// Session setup is owned by parse_display.steps.js BeforeAll/AfterAll; this
// step consumes the driver via the cucumber-js World (this.driver).

Then("the sidebar brand tagline should read {string}", async function (expected) {
  await this.driver.wait(
    until.elementLocated(By.css('[data-testid="brand-tagline"]')),
    10_000,
  );
  // Read the text in-page: tauri-wd cannot marshal element handles back
  // through selenium's atoms (see parse_display.steps.js).
  const tagline = await this.driver.executeScript(
    `return document.querySelector('[data-testid="brand-tagline"]').textContent.trim();`,
  );
  assert.equal(tagline, expected);
});
