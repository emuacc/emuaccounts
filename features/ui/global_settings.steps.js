const { When, Then } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");

// Session setup is owned by parse_display.steps.js BeforeAll/AfterAll. These
// steps consume the driver via the cucumber-js World (this.driver), matching
// update_prices_startup.steps.js / hide_transaction.steps.js.
//
// They assert presence only and never Save: the e2e vault is a temp dir, but
// the app-wide settings (prefixes, Show all holdings, AI, terms) live in the
// real app-data config.json, which Save rewrites.

When("I open the global settings dialog", async function () {
  await this.driver.wait(until.elementLocated(By.css("#globalSettingsBtn")), 10_000);
  // A previous scenario may have left the dialog open; its overlay would
  // swallow the gear click.
  await this.driver.executeScript(
    `if (!document.querySelector('#globalSettingsSave')) document.querySelector('#globalSettingsBtn')?.click();`,
  );
});

Then("I should see the included-accounts prefix input", async function () {
  await this.driver.wait(
    until.elementLocated(By.css('[data-testid="primary-prefix-input"]')),
    10_000,
  );
});

Then("I should see the included-accounts list", async function () {
  await this.driver.wait(
    until.elementLocated(By.css('[data-testid="primary-prefix-list"]')),
    10_000,
  );
});

Then("I should see the Show all holdings checkbox", async function () {
  await this.driver.wait(
    until.elementLocated(By.css('[data-testid="show-all-holdings"]')),
    10_000,
  );
});

Then("the clearing accounts list should show {string}", async function (account) {
  await this.driver.wait(
    until.elementLocated(
      By.css(`[data-testid="clearing-account-list"] [data-clearing-remove="${account}"]`),
    ),
    10_000,
  );
});

Then("I should see the clearing account input", async function () {
  await this.driver.wait(
    until.elementLocated(By.css('[data-testid="clearing-account-input"]')),
    10_000,
  );
});
