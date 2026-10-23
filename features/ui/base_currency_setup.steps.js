const assert = require("node:assert/strict");
const fs = require("node:fs");
const path = require("node:path");
const { Given, When, Then } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");

// Session setup is owned by parse_display.steps.js BeforeAll/AfterAll; it seeds
// generated/config.json with base_currency AUD and publishes the vault's
// generated dir on the World (this.generatedDir) so these steps can put the
// vault into (and back out of) the not-yet-set-up state.

function vaultConfigPath(world) {
  assert.ok(world.generatedDir, "generatedDir not published on the World");
  return path.join(world.generatedDir, "config.json");
}

function readVaultConfig(world) {
  const p = vaultConfigPath(world);
  return fs.existsSync(p) ? JSON.parse(fs.readFileSync(p, "utf8")) : {};
}

function writeVaultConfig(world, config) {
  fs.writeFileSync(vaultConfigPath(world), JSON.stringify(config, null, 2));
}

Given("the vault has no base currency", function () {
  const config = readVaultConfig(this);
  delete config.base_currency;
  writeVaultConfig(this, config);
});

Given("the vault base currency is {string}", function (currency) {
  writeVaultConfig(this, { ...readVaultConfig(this), base_currency: currency });
});

// The gate runs in the vault startup path. A page reload would exercise it,
// but reloading orphans the pending script inside tauri-plugin-webdriver-
// automation and poisons its lock for the rest of the session — so the
// VITE_E2E build exposes #e2e-restart-vault, which re-runs that same path.
When("the vault is reopened", async function () {
  const restart = await this.driver.findElement(By.css("#e2e-restart-vault"));
  await restart.click();
  await this.driver.wait(
    until.elementLocated(By.css('[data-testid="base-currency-setup"], [data-testid="app-ready"]')),
    30_000,
  );
});

Then("I should see the base currency setup screen", async function () {
  await this.driver.wait(until.elementLocated(By.css('[data-testid="base-currency-setup"]')), 10_000);
});

Then("the main page should not be shown", async function () {
  const sidebars = await this.driver.findElements(By.css(".sidebar"));
  assert.equal(sidebars.length, 0, "main page sidebar rendered behind the setup gate");
});

When("I choose the base currency {string}", async function (currency) {
  const input = await this.driver.wait(
    until.elementLocated(By.css('[data-testid="base-currency-input"]')),
    10_000,
  );
  await input.clear();
  await input.sendKeys(currency);
  const cont = await this.driver.findElement(By.css('[data-testid="base-currency-continue"]'));
  await cont.click();
});

Then("the main page should be shown", async function () {
  await this.driver.wait(until.elementLocated(By.css('[data-testid="app-ready"]')), 30_000);
  const gates = await this.driver.findElements(By.css('[data-testid="base-currency-setup"]'));
  assert.equal(gates.length, 0, "setup screen still visible");
});

Then("the vault config should have base currency {string}", function (currency) {
  assert.equal(readVaultConfig(this).base_currency, currency);
});
