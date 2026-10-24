# Multi-device sync (opt-in build)

Multi-device sync is unfinished, so only an opt-in build has it. A default
build — `cargo build`, `npx tauri build`/`dev`, the dev launchers, every
release — compiles none of it: no Sync section in the sidebar, no sync
commands in the backend, no `emuacc-relay` binary.

It is not the data plugins that "sync" an account (crypto-sync, simplefin-sync,
IBKR Flex, the daily price sync). Those are part of every build.

## What it does

Each device keeps an [Automerge](https://automerge.org) document,
`sources/emuacc-metadata.automerge`, listing its transactions, files, rules,
known devices and a sync log, and a content-addressed blob store,
`sources/cas/`. Two devices
pair through a self-hosted relay (`emuacc-relay`) with a 6-digit code; **Sync
Now** uploads the document, takes back the relay's merge and exchanges the
missing blobs. The sidebar's Sync section holds Pair Device, Sync Now, View
Sync Log and the known devices. Open work: automatic sync (TASK-086) and the
relay client's retry and timeout tests (TASK-081).

## Build and run it

One switch builds the app with sync: the Tauri config overlay
[`src-tauri/tauri.sync.conf.json`](../src-tauri/tauri.sync.conf.json) turns on
the `sync` cargo feature and builds the frontend with `VITE_SYNC=1`
(`npm run dev:sync` / `npm run build:sync`).

```sh
npx tauri dev --config src-tauri/tauri.sync.conf.json
npx tauri build --config src-tauri/tauri.sync.conf.json
```

The fast debug build (see [DEVELOPMENT.md](../DEVELOPMENT.md)) takes the flag
in both halves:

```sh
npm run build:sync && \
  cargo build --manifest-path src-tauri/Cargo.toml \
    --bin emuaccounts --features "sync webdriver tauri/custom-protocol"
```

The relay server:

```sh
cargo run --manifest-path src-tauri/Cargo.toml --features sync --bin emuacc-relay -- \
  --bind 127.0.0.1:8384 --data-dir /tmp/emuacc-relay
```

It listens on `0.0.0.0:8384` and keeps its data in `/tmp/emuacc-relay` unless
told otherwise. To pair, click **Pair Device** on one device and **Create
Code**, then **Pair Device** → **Enter Code** on the other; **Sync Now** on
either device exchanges the data.

## Files in the vault

- `sources/emuacc-metadata.automerge` — the sync metadata. Only the sync build
  writes it; the sync UI's `init_sync` opens it at startup and registers the
  device and the vault's sources unless this device already did.
- `sources/relay-config.json` (the paired relay and group) and `sources/cas/`
  (the blob store) are written by the sync build only. A default build ignores
  them.

None of them is pipeline input: changing them neither triggers a rebuild nor
defeats the pipeline's early exit (`src-tauri/src/sources_state.rs`).

## Trade links

Every build, this one included, keeps the trade links in
`sources/_trade_links.json` (`src-tauri/src/trade_links.rs`). The sync build
neither reads nor writes links in its document: to it the file is a source
file like the `_rules.json` files that hold the rules each link writes, and it
travels the way they do. Moving source files between devices is itself
unfinished: the manifest lists them, but nothing writes another device's
version into `sources/`.

Before TASK-133 the links lived in the document's `trade_links` map, in every
build. The first time any build reads the links of a vault with no
`_trade_links.json` yet, it copies that map's links into the file; it never
modifies the document. The sync build leaves the map in place, so a vault's
document keeps its pre-TASK-133 links, and an older build on another device
still reads them. A link saved since then is only in the file.

A default build reads the map with its own decoder
(`src-tauri/src/trade_links/automerge_doc.rs`), so only the sync build links
automerge; `scripts/run_sync_tests.sh` checks the decoder against automerge.

## Where the code lives

- Backend: [`src-tauri/src/sync/`](../src-tauri/src/sync/) — the library
  modules (`metadata.rs`, `protocol.rs`, `content_store.rs`, `relay/`,
  `relay_client.rs`, `state.rs`), plus the app's sync Tauri commands
  (`commands.rs`, a module of `main.rs`) and the relay server
  (`relay_main.rs`, the `emuacc-relay` binary).
- Frontend: [`src/sync.ts`](../src/sync.ts) and [`src/sync.css`](../src/sync.css).
- Tests: the unit tests in those modules, the scenarios in
  [`src-tauri/features/sync/`](../src-tauri/features/sync/) and their steps in
  [`src-tauri/tests/sync/steps.rs`](../src-tauri/tests/sync/steps.rs).

Outside them only these hook points refer to sync; the scenario
`src-tauri/features/architecture/sync_is_opt_in.feature` fails if a Rust file
outside `src/sync/` starts referring to it, so update this list with it:

- `src-tauri/Cargo.toml` — the `sync` feature (it also enables the
  `hostname`, `automerge` and `autosurgeon` dependencies) and
  `required-features` on `emuacc-relay`.
- `src-tauri/src/lib.rs` — `#[cfg(feature = "sync")] pub mod sync;`.
- `src-tauri/src/main.rs` — `mod sync_commands`, the command registration
  (`sync_commands::with_sync_commands!`) and its `MetadataState`.
- `src-tauri/src/sources_state.rs` — the relay files are not pipeline input.
- `src-tauri/tests/bdd.rs` — `mod sync_steps`, the `LedgerWorld::sync` field,
  and the runner skipping `features/sync/` without the feature.
- `src/main.ts` — the dynamic import behind `import.meta.env.VITE_SYNC === "1"`
  and the `syncUi?.…` calls (sidebar, modals, handlers, click-outside dismiss,
  vault switch, startup).
- `vitest.config.ts` — `src/sync.ts` is left out of the coverage measure.
- `src-tauri/tauri.sync.conf.json`, the `dev:sync` / `build:sync` npm scripts.

## Tests and gates

`scripts/run_sync_tests.sh` runs the Rust unit and integration tests and the
`features/sync/` scenarios with `--features sync`; CI's `unit_bdd` job runs it
too. The pre-push gate only type-checks the sync build (`cargo clippy
--features sync` and `cargo check --tests --features sync`), so run the script
after changing the sync code or anything it uses. The coverage ratchet measures
the default build.
