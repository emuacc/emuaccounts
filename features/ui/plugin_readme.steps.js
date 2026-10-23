const fs = require("fs");
const path = require("path");
const { After, Given, When, Then } = require("@cucumber/cucumber");
const { By, until } = require("selenium-webdriver");

// The app resolves a vault's plugins folder as <parent of sources>/plugins
// (main.rs resolve_plugins_dir); the harness vault's sources dir comes from
// the World (parse_display.steps.js Before hook).
function pluginsDir(world) {
  return path.join(path.dirname(world.sourcesDir), "plugins");
}

// Plugin folders this feature staged, removed after each scenario so they
// never leak into another run's discovery.
let staged = [];

Given("a plugin {string} with a README is installed in the vault", function (name) {
  const dir = path.join(pluginsDir(this), name);
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(
    path.join(dir, "plugin.toml"),
    `[plugin]\nname = "${name}"\nversion = "0.0.1"\ndescription = "e2e fixture"\nscript = "sync.py"\n`,
  );
  fs.writeFileSync(path.join(dir, "sync.py"), 'print("{}")\n');
  fs.writeFileSync(
    path.join(dir, "README.md"),
    `# ${name}\n\n## Setting up ${name}\n\n1. Build the query.\n2. Paste the [token](https://docs.example.invalid/token).\n`,
  );
  staged.push(dir);
});

After(function () {
  for (const dir of staged) {
    fs.rmSync(dir, { recursive: true, force: true });
  }
  staged = [];
});

When("I open the setup panel for plugin {string}", async function (name) {
  const btn = await this.driver.wait(
    until.elementLocated(By.css(`[data-testid="plugin-setup-${name}"]`)),
    10_000,
  );
  await btn.click();
});

// tauri-wd cannot marshal element handles reliably across calls, so every
// assertion re-queries the DOM inside executeScript.
async function panelText(driver, name) {
  return driver.executeScript(
    `const el = document.querySelector('[data-testid="plugin-readme-${name}"]');
     return el ? el.textContent : null;`,
  );
}

Then("the setup panel for plugin {string} shows {string}", async function (name, text) {
  await this.driver.wait(async () => {
    const content = await panelText(this.driver, name);
    return typeof content === "string" && content.includes(text);
  }, 10_000, `setup panel for ${name} never showed ${JSON.stringify(text)}`);
});

Then("the setup panel for plugin {string} opens its links outside the app", async function (name) {
  const targets = await this.driver.executeScript(
    `return Array.from(document.querySelectorAll('[data-testid="plugin-readme-${name}"] a'))
       .map((a) => a.getAttribute('target'));`,
  );
  if (!Array.isArray(targets) || targets.length === 0) {
    throw new Error(`setup panel for ${name} rendered no links`);
  }
  for (const target of targets) {
    if (target !== "_blank") throw new Error(`README link keeps target=${target}; the app would navigate away`);
  }
});
