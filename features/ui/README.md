# UI / E2E BDD (Cucumber)

UI/E2E `.feature` files executed with `cucumber-js`.

These are real WebDriver tests: a `selenium-webdriver` client (`By.css(...)`
DOM queries) drives the app's WebView. The only platform difference is which
WebDriver **server** serves `127.0.0.1:4444` — the feature files and steps are
identical everywhere.

## Requirements

- Node dependency `selenium-webdriver` installed (via `npm install`)

### Linux / Windows — `tauri-driver`

- Install: `cargo install tauri-driver --locked`
- Wraps the platform WebView driver (WebKitWebDriver / Edge WebDriver).
- On Linux, `WebKitWebDriver` has to be on `PATH`. Install Xvfb too (Arch:
  `xorg-server-xvfb`, Debian and Ubuntu: `xvfb`), or the app's window opens on
  your desktop and takes focus while the suite runs.

### macOS — `tauri-wd`

`tauri-driver` has no macOS backend (Apple ships no WKWebView WebDriver), so on
macOS the suite uses **`tauri-wd`** ([tauri-webdriver-automation]) — a W3C
WebDriver server that drives the WKWebView DOM directly. It's the same server
the `tauri-automation` MCP and `scripts/record_demos.mjs` already use.

- Install: `cargo install tauri-webdriver-automation` (provides `tauri-wd`)
- Grant **Accessibility + Automation** permission to your terminal
  (System Settings → Privacy & Security) — the WebDriver session can't drive the
  app window without it.
- Stop any running `emuaccounts` (e.g. `tauri dev`) first. `tauri-wd` owns
  port 4444; the suite `pkill`s stale `tauri-wd` servers to take the port.

[tauri-webdriver-automation]: https://github.com/danielraffel/tauri-webdriver

## Run

- `npm run e2e` — runs the suite, auto-selecting the backend for the current OS
  (`tauri-driver` on Linux/Windows, `tauri-wd` on macOS). Skips with a hint if
  the backend isn't installed.
- `npm run test:bdd:ui` — bare `cucumber-js`; same auto-selected backend, but it
  also runs the `@wip` and `@appium` scenarios.
- `npm run e2e:wd` — macOS DOM suite, explicitly forcing the `tauri-wd` backend
  (the non-skipping counterpart to `npm run e2e`).
- Force a backend with `E2E_BACKEND=tauri-wd|tauri-driver|appium-mac2`.

### Linux

`npm run e2e` needs no environment variables and no separate build. For one
feature, name it:

```
npx cucumber-js features/ui/parse_display.feature
```

Each run rebuilds the app first, so it tests the current tree with the E2E hooks.
If the build fails, the suite stops with the build's error and runs nothing.

The app runs on its own Xvfb display, so nothing shows on your desktop. The suite
starts and stops that display itself, on the first free number from `:99`, and it
prints a warning if Xvfb isn't installed. Then the window opens on your desktop
instead, which needs an X server (Xwayland on a Wayland desktop).

If you start Xvfb by hand, name the display (`Xvfb :99`). With `-displayfd` alone it
ignores lock files and can take `:0`, cutting a Wayland desktop off from Xwayland.

Either way the app runs under X11 at scale 1 (`GDK_BACKEND=x11`, `GDK_SCALE=1`, set
on the driver server). Under a fractional Wayland scale WebKitGTK reports a
`devicePixelRatio` like 1.18 and WebDriver clicks land on the wrong element.

### macOS — Appium mac2 (native-AX smoke, separate)

`npm run e2e:mac` runs the `@appium` scenarios via Appium's `mac2` driver. That
path automates the **native** macOS app through Accessibility (XCUITest), not
the DOM, so it only covers a coarse smoke check — it is **not** how the DOM
feature files above run. Requires Xcode + `./scripts/setup_appium_mac2.sh`.

## How it works

The driver lifecycle lives in `parse_display.steps.js` (`BeforeAll` / `Before` /
`AfterAll`), keyed on `selectedBackend()`. Every scenario gets a fresh app on an
empty vault with its base currency set, so a scenario sets up what it needs (usually
`I parse the transactions file ...`) and nothing it leaves behind reaches the next
one. Relaunching costs a few seconds a scenario. Run one suite at a time: the ports
(4444, and 4445 for the native driver) are fixed.

1. **Build.** `tauri-driver` → `tauri build --debug --no-bundle`.
   `tauri-wd` → `npm run build` then
   `cargo build --bin emuaccounts --features "webdriver tauri/custom-protocol"`
   into the isolated `~/.cache/emuacc-target-mcp` target dir. The `webdriver`
   feature embeds `tauri-plugin-webdriver-automation` so `tauri-wd` can attach;
   the isolated target dir keeps that feature's extra crates from churning the
   main `src-tauri/target`. Both builds set `VITE_E2E=1` so the frontend exposes
   the E2E hooks (`#e2e-parse-path`, `[data-testid="app-ready"]`).
2. **Server.** Spawn `tauri-driver` or `tauri-wd --port 4444`. `EMUACC_GENERATED_DIR`
   (a throwaway temp dir) is set on the **server process** — `tauri-wd` ignores
   the capabilities' `tauri:options.env`, so the launched app inherits it from
   the server instead.
3. **Session.** Identical capabilities everywhere: `tauri:options.application` =
   the built binary, `browserName: "wry"`, server `http://127.0.0.1:4444/`. On
   `tauri-wd` the WKWebView window handle appears ~1–2 s later, so the session
   selects it (`getAllWindowHandles` → `switchTo().window`) before querying.
4. **World bridge.** A `Before` hook publishes `this.driver = driver` so every
   step file can use `this.driver` (the convention in `categories` / `edit_rule` /
   `hide_transaction` / …) while this file's own steps use the module-level
   `driver`.

## CI gating: undefined steps

`scripts/run_feature_tests.sh` (run by the pre-commit hook and CI) does a
`cucumber-js --dry-run` over this directory and fails if any non-`@wip`
scenario has an undefined step. This is what stops the suite from silently
growing scenarios that never execute.

Scenarios whose step definitions are not yet wired up must be tagged
`@wip` at the feature level. Removing the tag is the trigger for actually
implementing the step file. `@appium` scenarios are mac-only and excluded
from the gate the same way.
