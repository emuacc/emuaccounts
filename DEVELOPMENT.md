# Development

Running EmuAcc from source, testing it and debugging it. How the code fits
together is in [ARCHITECTURE.md](ARCHITECTURE.md); toolchain versions are in
[TOOLCHAINS.md](TOOLCHAINS.md).

## Run the app

- Web (Vite): `npm run dev`
- Desktop (Tauri): `npm run tauri:dev` (frontend changes hot-reload; restart
  the command after Rust changes)

## Run the relay server locally

From the `src-tauri` directory:

```sh
cargo run --bin emuacc-relay
```

With options:

```sh
cargo run --bin emuacc-relay -- --bind 127.0.0.1:8384 --data-dir /tmp/emuacc-relay
```

`cargo run` builds in debug mode by default. Add `--release` for an optimised build.

## Tests

- Unit tests (tsc, cargo nextest, Vitest with coverage): `scripts/run_unit_tests.sh`
- Rust BDD (cucumber-rs, scenarios under [`src-tauri/features/`](./src-tauri/features/)): `npm run test:bdd`
- UI E2E (Tauri WebDriver + Cucumber): a `selenium-webdriver` DOM suite. `npm run e2e`
  auto-selects the WebDriver server for your OS and skips (with a hint) if it isn't installed.
  - **Linux/Windows** — `tauri-driver`: `cargo install tauri-driver --locked`
  - **macOS** — `tauri-wd` (`tauri-driver` has no macOS backend):
    `cargo install tauri-webdriver-automation`, then grant the terminal Accessibility +
    Automation permission. Run it explicitly with `npm run e2e:wd`.
  - Run: `npm run e2e` (or `scripts/run_e2e_tests.sh`). Details: [`features/ui/README.md`](features/ui/README.md).
- macOS Appium mac2 — a separate native-AX smoke (`@appium` scenarios only, not the DOM suite):
  - Setup once: `./scripts/setup_appium_mac2.sh`
  - Run: `npm run e2e:mac`

## Interactive debugging (MCP + Claude Code)

- An [mcp-tauri-automation](https://github.com/danielraffel/mcp-tauri-automation) server is configured in `.mcp.json`
- Install `tauri-wd` (macOS WebDriver): `cargo install tauri-webdriver-automation`
- Start: `tauri-wd --port 4444`
- Build the app with the frontend embedded and WebDriver enabled:

  ```sh
  npm run build && \
    cargo build --manifest-path src-tauri/Cargo.toml \
      --bin emuacc-covid --features "webdriver tauri/custom-protocol"
  ```

  Without `tauri/custom-protocol` the binary looks for a Vite dev server instead of the embedded `dist/`, and shows a blank screen.
- Claude Code can then launch the app, take screenshots, click elements, and execute JS in the WebView

## Demos

The screencasts in the README and the [user guide](USER_GUIDE.md#using-the-app)
are recorded from the real app with `npm run record:demos` — see
[docs/demos/RECORDING.md](docs/demos/RECORDING.md). The README's getting-started
take is re-recorded for every release; the other two are only re-made when their
flow changes.
