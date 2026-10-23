const assert = require("node:assert/strict");
const os = require("node:os");
const path = require("node:path");
const fs = require("node:fs");
const net = require("node:net");
const { spawn, spawnSync } = require("node:child_process");

const { Before, BeforeAll, AfterAll, Given, When, Then, setDefaultTimeout } = require("@cucumber/cucumber");
const { Builder, By, Capabilities, until } = require("selenium-webdriver");

setDefaultTimeout(120_000);

const repoRoot = path.resolve(__dirname, "../..");
const tauriDriverPath = path.resolve(os.homedir(), ".cargo", "bin", "tauri-driver");
const tauriWdPath = path.resolve(os.homedir(), ".cargo", "bin", "tauri-wd");
// macOS/tauri-wd builds go into this isolated target dir (matching
// scripts/run_debug_mcp.sh) so the `webdriver` feature's extra crates don't
// churn the main src-tauri/target shared with `cargo test` / `tauri dev`.
const mcpTargetDir = path.join(os.homedir(), ".cache", "emuacc-target-mcp");

let driverServer; // spawned tauri-driver (Linux/Windows) or tauri-wd (macOS)
let driver;
let generatedDir;
let sourcesDir;

// WebDriver backend that drives the suite:
//   - "tauri-driver": Tauri's WebDriver intermediary (Linux/Windows), DOM-level.
//   - "tauri-wd":     WKWebView WebDriver for macOS (DOM-level). Default on macOS
//                     because tauri-driver has no macOS backend.
//   - "appium-mac2":  native AX automation, owned by appium_macos.steps.js
//                     (@appium scenarios only) — this file no-ops for it.
// Override with the E2E_BACKEND env var; otherwise it is chosen by platform.
function selectedBackend() {
  const explicit = process.env.E2E_BACKEND;
  if (explicit === "appium-mac2" || explicit === "tauri-wd" || explicit === "tauri-driver") {
    return explicit;
  }
  return process.platform === "darwin" ? "tauri-wd" : "tauri-driver";
}

function mkTempDir(prefix) {
  return fs.mkdtempSync(path.join(os.tmpdir(), `${prefix}-`));
}

function resolveAppBinaryPath(backend) {
  // tauri-wd loads the binary built (with the `webdriver` feature) into the
  // isolated MCP target dir; tauri-driver uses the regular debug target dir.
  const targetDir =
    backend === "tauri-wd"
      ? path.join(mcpTargetDir, "debug")
      : path.join(repoRoot, "src-tauri", "target", "debug");

  const platformExt = process.platform === "win32" ? ".exe" : "";
  for (const name of ["emuacc-covid", "emuacc_covid", "tauri-app", "app"]) {
    const candidate = path.join(targetDir, `${name}${platformExt}`);
    if (fs.existsSync(candidate)) return candidate;
  }

  // macOS may output an .app even with --no-bundle, depending on Tauri version/config.
  const bundleDir = path.join(targetDir, "bundle", "macos");
  if (fs.existsSync(bundleDir)) {
    for (const app of fs.readdirSync(bundleDir).filter((f) => f.endsWith(".app"))) {
      const binName = path.basename(app, ".app");
      const candidate = path.join(bundleDir, app, "Contents", "MacOS", binName);
      if (fs.existsSync(candidate)) return candidate;
    }
  }

  const hint =
    backend === "tauri-wd"
      ? 'Build with: npm run build && CARGO_TARGET_DIR=~/.cache/emuacc-target-mcp cargo build --manifest-path src-tauri/Cargo.toml --bin emuacc-covid --features "webdriver tauri/custom-protocol"'
      : "Run: npx tauri build --debug --no-bundle";
  throw new Error(`Unable to locate built Tauri binary under ${targetDir}. ${hint}`);
}

function waitForPort(host, port, timeoutMs) {
  const start = Date.now();
  return new Promise((resolve, reject) => {
    const tick = () => {
      const socket = net.createConnection({ host, port });
      socket.once("connect", () => {
        socket.end();
        resolve();
      });
      socket.once("error", () => {
        socket.destroy();
        if (Date.now() - start > timeoutMs) reject(new Error(`Timed out waiting for ${host}:${port}`));
        else setTimeout(tick, 200);
      });
    };
    tick();
  });
}

function buildApp(backend) {
  if (backend === "tauri-wd") {
    // 1) Frontend with the E2E DOM hooks (#e2e-parse-path, [data-testid=app-ready]),
    //    gated behind VITE_E2E. tauri/custom-protocol embeds this dist/ at link time.
    const fe = spawnSync("npm", ["run", "build"], {
      cwd: repoRoot,
      stdio: "inherit",
      shell: true,
      env: { ...process.env, VITE_E2E: "1" },
    });
    if (fe.status !== 0) throw new Error("Frontend build failed (npm run build).");

    // 2) App binary with tauri-plugin-webdriver-automation (the `webdriver`
    //    feature) so tauri-wd can attach, built into the isolated target dir.
    const app = spawnSync(
      "cargo",
      [
        "build",
        "--manifest-path",
        "src-tauri/Cargo.toml",
        "--bin",
        "emuacc-covid",
        // Comma-separated (not space) so it survives as ONE shell word under
        // spawnSync's shell:true, which would otherwise split it into a stray arg.
        "--features",
        "webdriver,tauri/custom-protocol",
      ],
      {
        cwd: repoRoot,
        stdio: "inherit",
        shell: true,
        env: { ...process.env, CARGO_TARGET_DIR: mcpTargetDir },
      },
    );
    if (app.status !== 0) throw new Error("App build failed (cargo build --features webdriver).");
    return;
  }

  spawnSync("npx", ["tauri", "build", "--debug", "--bundles", "none"], {
    cwd: repoRoot,
    stdio: "inherit",
    shell: true,
    env: { ...process.env, VITE_E2E: "1" },
  });
}

function startDriverServer(backend) {
  if (backend === "tauri-wd") {
    if (!fs.existsSync(tauriWdPath)) {
      throw new Error(
        `tauri-wd is not installed at ${tauriWdPath}. Install with: cargo install tauri-webdriver-automation`,
      );
    }
    // tauri-wd owns port 4444 exclusively; drop any stale server (e.g. one left
    // running for MCP debugging) so this suite drives the binary we just built.
    spawnSync("pkill", ["-f", "tauri-wd"]);
    // tauri-wd ignores capabilities' `tauri:options.env`; the launched app
    // inherits THIS process's env, so the vault overrides go here. BOTH dirs
    // must be overridden: with only EMUACC_GENERATED_DIR set, sources resolve
    // to the configured real vault (env > config > fallback) and .setup()
    // cold-transforms the whole vault into the empty temp dir — a debug-build
    // pipeline run that blocks the window past every wait in this suite.
    driverServer = spawn(tauriWdPath, ["--port", "4444"], {
      stdio: [null, process.stdout, process.stderr],
      env: { ...process.env, EMUACC_GENERATED_DIR: generatedDir, EMUACC_SOURCES_DIR: sourcesDir },
    });
  } else {
    if (!fs.existsSync(tauriDriverPath)) {
      throw new Error(
        `tauri-driver is not installed at ${tauriDriverPath}. Install with: cargo install tauri-driver --locked`,
      );
    }
    driverServer = spawn(tauriDriverPath, [], {
      stdio: [null, process.stdout, process.stderr],
      env: { ...process.env, EMUACC_GENERATED_DIR: generatedDir, EMUACC_SOURCES_DIR: sourcesDir },
    });
  }
}

// tauri-wd surfaces the WKWebView window handle ~1-2s after the session starts;
// it must be selected before the DOM is queryable. (tauri-driver auto-focuses.)
async function selectWebviewWindow(timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  let handles = [];
  while (Date.now() < deadline) {
    try {
      handles = await driver.getAllWindowHandles();
    } catch {
      handles = [];
    }
    if (handles.length) break;
    await driver.sleep(200);
  }
  if (!handles.length) throw new Error("tauri-wd: app window did not appear");
  await driver.switchTo().window(handles[0]);
}

async function startWebdriverSession() {
  const backend = selectedBackend();

  buildApp(backend);

  generatedDir = mkTempDir("emuacc-covid-e2e-generated");
  sourcesDir = mkTempDir("emuacc-covid-e2e-sources");
  // A vault with no base currency is gated behind the first-run setup screen
  // (base_currency_setup.feature covers that). Every other scenario expects
  // the main page, so seed the vault config the way a set-up vault has it.
  fs.writeFileSync(path.join(generatedDir, "config.json"), JSON.stringify({ base_currency: "AUD" }, null, 2));
  startDriverServer(backend);
  await waitForPort("127.0.0.1", 4444, 30_000);

  const capabilities = new Capabilities();
  capabilities.set("tauri:options", { application: resolveAppBinaryPath(backend) });
  capabilities.setBrowserName("wry");

  driver = await new Builder()
    .withCapabilities(capabilities)
    .usingServer("http://127.0.0.1:4444/")
    .build();

  if (backend === "tauri-wd") await selectWebviewWindow(20_000);

  await driver.wait(until.elementLocated(By.css('[data-testid="app-ready"]')), 30_000);
}

async function stopWebdriverSession() {
  try {
    if (driver) await driver.quit();
  } finally {
    if (driverServer) driverServer.kill();
    if (generatedDir) {
      try {
        fs.rmSync(generatedDir, { recursive: true, force: true });
      } catch {}
    }
    if (sourcesDir) {
      try {
        fs.rmSync(sourcesDir, { recursive: true, force: true });
      } catch {}
    }
  }
}

BeforeAll(async () => {
  if (selectedBackend() === "appium-mac2") return; // appium_macos.steps.js owns that path
  await startWebdriverSession();
});

AfterAll(async () => {
  if (selectedBackend() === "appium-mac2") return;
  await stopWebdriverSession();
});

// Bridge the driver onto the cucumber World so step files can use `this.driver`
// (the convention in categories/edit_rule/hide_transaction/…) alongside the
// module-level `driver` used by this file's own steps.
Before(function () {
  if (selectedBackend() !== "appium-mac2") {
    this.driver = driver;
    this.generatedDir = generatedDir;
    this.sourcesDir = sourcesDir;
  }
});

Given("the app is running", async () => {
  const el = await driver.findElement(By.css('[data-testid="app-ready"]'));
  // getDomAttribute, not getAttribute: selenium's getAttribute atom calls
  // Element.getAttributeNode, which current WKWebView no longer implements.
  assert.equal(await el.getDomAttribute("data-testid"), "app-ready");
});

/** The fixture's first posting under one of `roots` (e.g. "assets"): the
 *  account the staged folder declares and the scenario selects. */
function fixtureAnchor(fixturePath, roots) {
  const content = fs.readFileSync(fixturePath, "utf8");
  const anchor = new RegExp(`^\\s+((?:${roots.join("|")}):[\\w:.-]+)\\s+(-?[\\d,.]+)\\s+([A-Za-z][\\w.:-]*)`, "m").exec(content);
  assert.ok(anchor, `fixture has no ${roots.join("/")} posting to anchor on: ${fixturePath}`);
  return { account: anchor[1], commodity: anchor[3] };
}

When('I parse the transactions file {string}', async (relativePath) => {
  const fixturePath = path.resolve(repoRoot, relativePath);
  assert.ok(fs.existsSync(fixturePath), `fixture file missing: ${fixturePath}`);

  // Stage the fixture the way the vault stores data: a manual.transactions +
  // account declaration in the fixture's FIRST asset account's folder (the
  // account the scenario selects), in the legacy shorthand layout under a
  // single "e2e" owner so every fixture shares one account set. The folder
  // matters: query_search folder-prunes by account, so transactions only
  // surface when their ledger lives in the selected account's own folder.
  const { account, commodity } = fixtureAnchor(fixturePath, ["assets"]);
  const folder = path.join(sourcesDir, "e2e", ...account.replace(/^assets:/, "").split(":"));
  await stageFixtureAndRebuild(fixturePath, folder, account, commodity);
});

When('I stage the transactions file {string} in the source folder {string}', async (relativePath, folderRelative) => {
  // Explicit-layout staging: the folder is given relative to sources (e.g.
  // "e2e/liabilities/credit-card/amex"), so the pipeline derives the account
  // from the folder's root marker rather than from the shorthand layout.
  const fixturePath = path.resolve(repoRoot, relativePath);
  assert.ok(fs.existsSync(fixturePath), `fixture file missing: ${fixturePath}`);
  const { account, commodity } = fixtureAnchor(fixturePath, ["assets", "liabilities"]);
  const folder = path.join(sourcesDir, ...folderRelative.split("/"));
  await stageFixtureAndRebuild(fixturePath, folder, account, commodity);
});

/** Stage `fixturePath` as `<folder>/manual.transactions` (plus the account's
 *  declaration) in the harness vault, rebuild through the in-app e2e hook,
 *  then re-run vault startup so the "e2e" set is discovered and selected. */
async function stageFixtureAndRebuild(fixturePath, folder, account, commodity) {
  // A previous scenario may have ended inside the rule/party editor view
  // (which replaces the main pane — no status line, no txn table) or with a
  // modal open (whose overlay intercepts clicks). Dismiss modals and
  // navigate back to the accounts view first; both are no-ops when clean.
  await driver.executeScript(
    `document.querySelectorAll('.modalOverlay').forEach((o) => o.click());
     document.querySelector('.sidebarNav__item[data-view="accounts"]')?.click();`,
  );
  await driver.wait(until.elementLocated(By.css('[data-testid="parse-status"]')), 10_000);

  // The in-app e2e hook rebuilds the pipeline over the harness vault — it no
  // longer parses a typed path (that flow died when #e2e-parse-run switched
  // to rebuildPipeline).
  fs.mkdirSync(folder, { recursive: true });
  fs.copyFileSync(fixturePath, path.join(folder, "manual.transactions"));
  const accountsPath = path.join(folder, "accounts.transactions");
  const declaration = `account ${account} ${commodity}\n`;
  const existing = fs.existsSync(accountsPath) ? fs.readFileSync(accountsPath, "utf8") : "";
  if (!existing.includes(declaration)) fs.appendFileSync(accountsPath, declaration);

  const run = await driver.findElement(By.css("#e2e-parse-run"));
  await run.click();

  await driver.wait(
    until.elementTextContains(
      driver.findElement(By.css('[data-testid="parse-status"]')),
      "Parsed",
    ),
    30_000,
  );

  // Re-run vault startup so a freshly-created account set is discovered and
  // auto-selected — the session began on an empty vault with no sets. Never
  // location.reload() here (it poisons the webdriver plugin); the restart
  // hook re-runs startVault the way a relaunch would.
  const restart = await driver.findElement(By.css("#e2e-restart-vault"));
  await restart.click();
  await driver.wait(until.elementLocated(By.css('[data-testid="app-ready"]')), 30_000);

  // Stale scenarios (add_account) can create and switch to other account
  // sets; every staged fixture lives in the "e2e" set — switch back when a
  // set picker is present and pointing elsewhere.
  await driver.executeScript(
    `const sel = document.querySelector('#accountSetSelect');
     if (sel && sel.value !== 'e2e' && Array.from(sel.options).some((o) => o.value === 'e2e')) {
       sel.value = 'e2e';
       sel.dispatchEvent(new Event('change', { bubbles: true }));
     }`,
  );
}

Then('the sidebar should include the account {string}', async (account) => {
  const selector = `[data-testid="account-item"][data-account="${account}"]`;
  await driver.wait(until.elementLocated(By.css(selector)), 10_000);
});

// Account groups are the drill-down's group rows (data-group = the group's
// full account path, e.g. "assets:exchange" or "liabilities").
Then('the sidebar should include the account group {string}', async (group) => {
  const selector = `[data-testid="drilldown-item"][data-group="${group}"]`;
  await driver.wait(until.elementLocated(By.css(selector)), 10_000);
});

Then('the sidebar should not include the account group {string}', async (group) => {
  const selector = `[data-testid="drilldown-item"][data-group="${group}"]`;
  const elements = await driver.findElements(By.css(selector));
  assert.equal(elements.length, 0, `Expected account group "${group}" to not be in the sidebar`);
});

Then('the account {string} should show an amount of {string}', async (account, amount) => {
  const selector = `[data-testid="account-item"][data-account="${account}"]`;
  const item = await driver.findElement(By.css(selector));
  const amt = await item.findElement(By.css(".account__amt"));
  assert.equal(await amt.getText(), amount);
});

When('I select the account {string}', async (account) => {
  const selector = `[data-testid="account-item"][data-account="${account}"]`;
  // The sidebar auto-drills into single-chain folders (and persists the
  // drill), so another fixture's account may not be in the DOM at all —
  // reset to the folder root first.
  await driver.executeScript(
    `document.querySelector('[data-testid="sidebar-folder-root"]')?.click();`,
  );
  // At the root only GROUP nodes render; leaf account items appear only at
  // their drill level. Drill one matching group per RENDER until the leaf
  // item is in the DOM. The signature guard matters: drill clicks push a
  // path segment blindly, so clicking the same rendered list twice corrupts
  // the drill path — only click again once the list has changed.
  // (Everything runs inside executeScript — tauri-wd can't marshal element
  // handles, and re-renders would stale them.)
  const DRILL_SCRIPT = `
    if (document.querySelector(arguments[0])) return { done: true };
    const groups = Array.from(document.querySelectorAll('[data-testid="drilldown-item"]'));
    const sig = groups.map(g => g.getAttribute('data-group')).join('|');
    if (sig === arguments[2]) return { done: false, sig: null };
    const target = groups.find(g => {
      const group = g.getAttribute('data-group') || '';
      return arguments[1] === group || arguments[1].startsWith(group + ':');
    });
    if (!target) {
      // Drilled into the wrong subtree (persisted drill from an earlier
      // scenario) — escape to the folder root and let the next poll drill.
      const root = document.querySelector('[data-testid="sidebar-folder-root"]');
      if (root) { root.click(); return { done: false, sig }; }
      return { done: false, sig: null };
    }
    target.click();
    return { done: false, sig };`;
  const deadline = Date.now() + 20_000;
  let lastSig = null;
  let stalePolls = 0;
  let found = false;
  while (Date.now() < deadline) {
    const res = await driver.executeScript(DRILL_SCRIPT, selector, account, lastSig);
    if (res && res.done) { found = true; break; }
    if (res && res.sig) {
      lastSig = res.sig;
      stalePolls = 0;
    } else {
      // A click can land before the render re-attaches listeners and change
      // nothing; if the list stays identical for ~1.5s, allow a re-click
      // rather than dead-locking on the signature guard.
      stalePolls += 1;
      if (stalePolls > 10) { lastSig = null; stalePolls = 0; }
    }
    await driver.sleep(150);
  }
  if (!found) {
    const dump = await driver.executeScript(
      `return JSON.stringify({
         drill: Array.from(document.querySelectorAll('[data-testid="drilldown-item"]')).map(e => e.getAttribute('data-group')),
         items: Array.from(document.querySelectorAll('[data-testid="account-item"]')).map(e => e.getAttribute('data-account')),
         status: document.querySelector('[data-testid="parse-status"]')?.textContent?.trim(),
       });`,
    );
    assert.fail(`account item ${account} never rendered after drilling — DOM: ${dump}`);
  }
  const item = await driver.wait(until.elementLocated(By.css(selector)), 5_000);
  await item.click();

  // The header shows the pretty form ("exchange : kraken:btc") — the raw
  // folder root ("assets:" / "liabilities:") and inner whitespace never
  // appear, so normalize before comparing instead of using
  // elementTextContains on the raw account name.
  const expected = account.replace(/^(assets|liabilities):/, "");
  let lastSeen = "";
  await driver.wait(async () => {
    lastSeen = await driver.executeScript(
      `const el = document.querySelector('[data-testid="selected-account"]');
       return el ? el.textContent.replace(/\\s+/g, "") : "";`,
    );
    return lastSeen.includes(expected);
  }, 10_000, () => `selected-account header never showed "${expected}" (saw "${lastSeen}")`);
});

Then('I should see a transaction row for payee {string}', async (payee) => {
  const row = await driver.findElement(By.css('[data-testid="txn-row"]'));
  const cell = await row.findElement(By.css('[data-testid="txn-payee"]'));
  assert.equal(await cell.getText(), payee);
});

Then('I should see a notes value of {string}', async (notes) => {
  const row = await driver.findElement(By.css('[data-testid="txn-row"]'));
  const cell = await row.findElement(By.css('[data-testid="txn-notes"]'));
  assert.equal(await cell.getText(), notes);
});

Then('that transaction row should show a deposit of {string}', async (amount) => {
  const row = await driver.findElement(By.css('[data-testid="txn-row"]'));
  const deposit = await row.findElement(By.css('[data-testid="txn-deposit"]'));
  assert.equal(await deposit.getText(), amount);
});

When('I add a manual transaction with payee {string} and notes {string}', async (payee, notes) => {
  const addNew = await driver.findElement(By.css("#addNew"));
  await addNew.click();

  await driver.wait(until.elementLocated(By.css("#manualPayee")), 10_000);
  const payeeInput = await driver.findElement(By.css("#manualPayee"));
  const notesInput = await driver.findElement(By.css("#manualNarration"));

  await payeeInput.clear();
  await payeeInput.sendKeys(payee);
  await notesInput.clear();
  await notesInput.sendKeys(notes);

  // Value mode (default): an amount into the selected account...
  const amount = await driver.findElement(By.css("#manualAmount"));
  await amount.clear();
  await amount.sendKeys("3.50");
  // ...and one "other account" row whose blank amount auto-balances it.
  const contra = await driver.findElement(By.css("#manualContraAccount-0"));
  await contra.clear();
  await contra.sendKeys("expenses:coffee");

  const save = await driver.findElement(By.css("#manualSave"));
  await save.click();

  await driver.wait(until.elementTextContains(driver.findElement(By.css('[data-testid="parse-status"]')), "added manual"), 30_000);
});

When('I click the "Add New" button', async () => {
  const addNew = await driver.findElement(By.css("#addNew"));
  await addNew.click();
  await driver.wait(until.elementLocated(By.css('[data-testid="manual-txn-modal"]')), 10_000);
});

Then('the account field should show {string}', async (expected) => {
  const acct = await driver.findElement(By.css(".manualAccount"));
  const value = await acct.getText();
  assert.ok(value.includes(expected), `Expected account field to show "${expected}" but got "${value}"`);
});

// Add Account feature steps
When('I click the "Add Account" button', async () => {
  const addAccountBtn = await driver.findElement(By.css('[data-testid="add-account-btn"]'));
  await addAccountBtn.click();
  await driver.wait(until.elementLocated(By.css('[data-testid="add-account-modal"]')), 10_000);
});

When('I enter account name {string}', async (accountName) => {
  const input = await driver.findElement(By.css('#newAccountName'));
  await input.clear();
  await input.sendKeys(accountName);
});

Then('the entity select should show the current account set', async () => {
  const current = await driver.executeScript(
    `return { entity: document.querySelector('#newAccountEntity')?.value,
              set: localStorage.getItem('emuacc_selectedAccountSet') };`,
  );
  assert.ok(current.entity, 'entity select should be present');
  if (current.set) {
    assert.strictEqual(current.entity, current.set,
      `entity select should default to ${current.set}, got ${current.entity}`);
  }
});

When('I choose to create entity {string}', async (entityName) => {
  await driver.executeScript(
    `const sel = document.querySelector('#newAccountEntity');
     sel.value = '__new__';
     sel.dispatchEvent(new Event('change', { bubbles: true }));`,
  );
  await driver.wait(until.elementLocated(By.css('#newAccountEntityName')), 10_000);
  const input = await driver.findElement(By.css('#newAccountEntityName'));
  await input.clear();
  await input.sendKeys(entityName);
});

When('I select wallet chain {string}', async (chain) => {
  await driver.executeScript(
    `const sel = document.querySelector('#newWalletChain');
     sel.value = arguments[0];
     sel.dispatchEvent(new Event('change', { bubbles: true }));`,
    chain,
  );
  await driver.wait(async () => {
    const sel = await driver.findElement(By.css('#newWalletChain'));
    return (await sel.getAttribute('value')) === chain;
  }, 10_000);
});

When('I enter wallet address {string}', async (address) => {
  const input = await driver.findElement(By.css('#newWalletAddress'));
  await input.clear();
  await input.sendKeys(address);
});

When('I select account type {string}', async (accountType) => {
  await driver.executeScript(
    `const sel = document.querySelector('#newAccountType');
     sel.value = arguments[0];
     sel.dispatchEvent(new Event('change', { bubbles: true }));`,
    accountType,
  );
  // The change handler re-renders the modal; wait for the select to reflect it.
  await driver.wait(async () => {
    const sel = await driver.findElement(By.css('#newAccountType'));
    return (await sel.getAttribute('value')) === accountType;
  }, 10_000);
});

When('I submit the new account form', async () => {
  const submitBtn = await driver.findElement(By.css('#addAccountSubmit'));
  await submitBtn.click();
  // Wait for modal to close
  await driver.wait(async () => {
    const modals = await driver.findElements(By.css('[data-testid="add-account-modal"]'));
    return modals.length === 0;
  }, 10_000);
});

// An empty vault: nothing under sources/, so there is no account set to pick
// from. The base currency stays set, so the main page renders. Re-runs vault
// startup through the e2e hook (never location.reload(): it poisons the
// webdriver plugin's lock).
Given('the vault has no account sets', async () => {
  for (const entry of fs.readdirSync(sourcesDir)) {
    fs.rmSync(path.join(sourcesDir, entry), { recursive: true, force: true });
  }
  await driver.executeScript(`document.querySelectorAll('.modalOverlay').forEach((o) => o.click());`);
  const restart = await driver.findElement(By.css("#e2e-restart-vault"));
  await restart.click();
  await driver.wait(until.elementLocated(By.css('[data-testid="app-ready"]')), 30_000);
});

// Regression: with no account sets the entity select holds only "New entity…"
// and can never fire a change event, so the form has to start there — or the
// name input never renders and the very first account cannot be added.
Then('the new entity name field should be shown', async () => {
  await driver.wait(until.elementLocated(By.css('#newAccountEntityName')), 10_000);
});
