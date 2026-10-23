# Architecture

EmuAcc treats your finances like an IDE treats code: raw source files are
**compiled** through transforms and rules into a unified ledger, and every view
and report is derived from that ledger. The principles behind it are in the
[README](README.md#principles).

```
 sources/  ──▶  transforms + rules  ──▶  generated/  ──▶  reports + UI
 CSV · OFX        _transform.rhai        per-folder ledgers
 manual.txns      _rules.json            + a unified active ledger per account set
```

- **Sources** (`sources/`) are the source of truth — per-account folders holding
  raw exports (CSV, OFX) and hand-written `manual.transactions` (hledger-style). A
  folder's path *is* the account: `alex/crypto/wallet/ethereum/` → `assets:ethereum`.
- **Transforms & rules** are the compiler. Each import folder has a `_transform.rhai`
  (a small [Rhai](https://rhai.rs) script mapping CSV/OFX columns to ledger postings)
  and an optional `_rules.json` (payee/category rules). A missing transform surfaces
  as a warning badge on the account rather than failing the whole build.
- **Incremental build.** A SHA256 content hash plus a directory fingerprint per
  folder lets a rebuild skip everything that didn't change. A file watcher on
  `sources/` recompiles automatically whenever a file changes, except the relay
  sync state (see [Data file locations](USER_GUIDE.md#data-file-locations)).
- **Generated** (`generated/`) is the compiled output: per-folder ledgers plus a
  unified active ledger per account set. It is derived and never hand-edited —
  delete it and it rebuilds from `sources/`.
- **One source of truth, written back.** Every view, report and figure is
  derived from the ledger, never patched at display time. When you link the two
  legs of a trade, classify a payee, set an opening balance or import prices,
  the result is written back into `sources/` — a rule, a transaction, a price
  line — and the ledger is rebuilt from it.
- **Reports** (CGT, income, balances, performance) are generated from the active
  ledger, per account set and financial year. Each `cgt-YYYY` report carries
  only its own year's warnings; the whole list is `reports/cgt-warnings.md`.
  FIFO matches same-timestamp acquisitions in `txn:` id order, then `leg:` id —
  never by payee text — so a rebuild on any machine gives the same figures.
  The tax-year boundary and the CGT discount are configuration (`tax` in the
  vault's `config.json`); the defaults are Australian.
- **Stable output.** `generated/` is a function of (sources, prices, config,
  generator build): a full rebuild leaves every unaffected file byte-identical,
  so a vault that commits `generated/` gets diffs that read as "this data
  changed, and this is its impact". Report JSON is rounded at the serialisation
  boundary (money to 6 decimals, quantities and prices to 12 significant
  digits) and lists are persisted in commodity order — the UI ranks them by
  value. `generated/generator.json` records the build that produced the output
  (git sha, commit time, crate and matcher version); a build older than the one
  recorded warns before it regenerates.
- **Base currency.** Every balance, header total and report is valued in one
  currency. The data folder's default lives in `generated/config.json` as
  `base_currency` and is inherited by every account set; a set can override it
  in its own `generated/<set>/config.json`. A folder without one is gated behind
  a setup screen on launch (after the data-folder picker) — without it holdings
  cannot be valued, and the CLI report generators refuse to run rather than
  guess.
- **Desktop app.** A [Tauri](https://tauri.app) app — a Rust core with a WebView
  UI. The UI queries the generated ledgers; user actions (import, add/classify
  transactions, link trades) write back to `sources/` and trigger a rebuild, so the
  ledger stays the single source of truth.
- **Plugins** fetch external data and only ever write to `sources/`, where it
  flows through the normal pipeline ([PLUGINS.md](PLUGINS.md)).
- **Sync.** Changes are stored locally in an [Automerge](https://automerge.org) CRDT
  and can be synced across devices through a self-hosted relay (see
  [Multi-device sync](USER_GUIDE.md#multi-device-sync)).

The CLI mirrors the app: `emuacc-regenerate` runs the same pipeline headlessly,
and `emuacc-query` reads the generated ledgers (see
[Command line](USER_GUIDE.md#command-line)).

## Format

The ledger is a double-entry journal in the ledger/hledger family: dated
transactions with a payee, a narration and balanced postings, cost and price
annotations, and a metadata comment carrying stable ids.

```
2025-01-15 * "CommSec" "Buy 100 BHP @ 40.00 AUD" ; txn:buy-bhp
    assets:broker 100 BHP @@ 4000.00 AUD
    assets:broker:cash -4019.95 AUD
    expenses:fees:brokerage 19.95 AUD
```

Commodities are whatever you count: currencies, shares, coins, venue-scoped
instruments such as `xyz:WTIOIL`.

[`Ledger.g4`](./Ledger.g4) is the authoritative ledger record grammar.
[`LedgerLexer.g4`](./LedgerLexer.g4) supplies its shared tokens and
[`Prices.g4`](./Prices.g4) defines `P` price directives from the same token
vocabulary. The runtime parser is the handwritten one in
`src-tauri/src/ledger_parser.rs`. A parser generated from the grammar
(antlr4rust, a dev-dependency) is committed and exists only in test builds,
where `src-tauri/src/ledger_parser/grammar_parity.rs` holds the two to the
same accept/reject decision (see `.backlog/decisions/decision-003`):

- every fixture in `src-tauri/features/fixtures/`: a stem of `invalid` or
  ending in `_invalid` is expected to be rejected, anything else accepted;
- the commodity, account and amount validators against the grammar's
  `commodityInput` / `accountInput` / `amountInput` entry points, over
  `src-tauri/features/fixtures/validator_corpus.tsv`;
- opt-in, every `*.transactions`, `*.prices` and `_prices/*.txt` file in a
  vault, reporting each disagreement with its path, line and both decisions:

  ```bash
  EMUACC_PARITY_VAULT=~/vault \
    cargo test --manifest-path src-tauri/Cargo.toml --lib \
    grammar_parity::whole_vault -- --ignored --nocapture
  ```

To change the syntax, change the grammar first, regenerate with
`scripts/generate_ledger_parsers.sh` (Java 21; `--check` reports drift and
runs in CI), update the fixtures, then the handwritten parser. Ordinary builds
do not require Java.
Toolchain versions (Rust 1.98.0, OpenJDK 21) and how to select them are
documented in [`TOOLCHAINS.md`](./TOOLCHAINS.md).
