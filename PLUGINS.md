# Plugins

Plugins are Python scripts that fetch external data (prices, exchange trades, etc.) and write it into your `sources/` folder. The pipeline rebuilds automatically when new files appear.

## Interface

Plugins are subprocesses spawned by the EmuAcc UI when you click **Plugins → Run**. They inherit:

- **Input**: a JSON context on stdin (see [Script Interface](#script-interface) below) plus environment variables.
- **Tools**: the full suite of `emuacc-*` CLI binaries — `emuacc-query` in particular — are on `$PATH` and their absolute paths are provided in `ctx.bin` and `EMUACC_QUERY_BIN`. Plugins should use these to read ledger data rather than parsing files directly; if the ledger file layout changes, plugins keep working.
- **Permissions**: plugins may write to the vault's `sources/` folder. They **must not** touch `generated/` — that folder is owned by the pipeline and is rebuilt from `sources/` on every change.
- **Output**: files in `sources/`, plus a JSON status summary printed to stdout.

## Location

Plugins live in `plugins/` inside your **data root folder** (the same folder that contains `sources/` and `generated/`). This is the folder you selected when you first launched EmuAcc.

```
your-data-root/
  sources/
  generated/
  plugins/          <-- put plugins here
    my-plugin/
      plugin.toml
      sync.py
```

The app ships a plugin catalog (the repo's `plugins/`, bundled as app resources). Use the **Add Plugins** button on the Plugins page to install any catalog plugin into the current data root — installing copies the plugin folder in, excluding runtime state (`.data/`, caches). In dev builds the catalog location comes from the `EMUACC_PLUGIN_CATALOG` env var (exported by `scripts/run_debug.sh` and `scripts/run_debug_mcp.sh`).

`scripts/run_debug.sh` also installs the whole catalog into the vault it launches: it `rsync`s `plugins/` → `<vault>/plugins/` with `--delete`, excluding `.data/` and the per-vault `wallets.json` / `accounts.json`, so each vault keeps its config and secrets. Author plugins **in the repo, not the vault** — a deploy overwrites anything vault-only.

## Structure

A plugin folder contains a manifest and a script:

```
my-plugin/
  plugin.toml    # manifest (name, config fields, secrets)
  sync.py        # entry point (python3, PEP 723 inline metadata for deps)
  README.md      # optional setup guide, shown behind the plugin card's Setup toggle
  .data/         # per-vault state: config.json, secrets.json, last_run.json (git-ignored)
```

The README travels with the plugin, so setup instructions (which broker query to build, where the token comes from) show where the plugin is configured instead of living only in the repo.

### Dependencies (PEP 723 + uv)

Plugins declare their Python dependencies inline in the script, using
[PEP 723 script metadata](https://peps.python.org/pep-0723/). The runner
detects the marker and invokes `uv run --script <file>`, which provisions
an ephemeral, hardlinked venv from a global wheel cache on first run.

Each plugin gets its own venv — there is no shared ecosystem environment,
so plugins can pin different versions and removing a plugin removes its
deps cleanly.

```python
#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["yfinance>=0.2", "pandas"]
# ///
"""My plugin docstring."""
import yfinance as yf
...
```

Stdlib-only plugins still declare an empty `dependencies = []` block so
the runner can route through `uv` consistently. If `uv` is not on PATH,
the runner falls back to bare `python3` (which works for stdlib-only
plugins and breaks loudly on plugins that need third-party libraries —
install uv via `brew install uv` or `curl -LsSf https://astral.sh/uv/install.sh | sh`).

`uv` resolves a script's deps lazily on first invocation (~150ms with
the hardlinked wheel cache); subsequent runs are instant. `scripts/run_debug.sh`
does **not** pre-warm — running plugin scripts has side effects (hits APIs,
writes into `sources/`), so we leave dep resolution to the first real click.
The script does check that `uv` is on PATH and prints an install hint if not.

## Manifest

`plugin.toml` declares the plugin metadata, configurable fields, and secret fields:

```toml
[plugin]
name = "My Plugin"
version = "0.1.0"
description = "What this plugin does"
script = "sync.py"
daily = true   # optional; include in the on-startup price backfill (default false)

[config]
# Fields the user can configure in the UI
some_option = { type = "string", default = "hello", description = "An option" }
count = { type = "integer", default = 10, description = "How many things" }
items = { type = "list", default = ["a", "b"], description = "A list of items" }

[secrets]
# Fields stored securely (shown as password inputs, never committed)
api_key = { type = "string", required = true, description = "API key" }
```

### `daily` — automatic startup backfill

Set `daily = true` in `[plugin]` to include the plugin in the **"Update prices
on startup"** batch. When that checkbox (in the Plugins view) is enabled, the
app runs every `daily` plugin shortly after launch, then rebuilds the pipeline
once. The setting is off by default and persists in the app config.

Three rules make this safe to run on every launch:

- **Once per day:** a plugin that already succeeded today (per its
  `.data/last_run.json`) is skipped without running, so repeated launches on the
  same day make no network calls.
- **Incremental & idempotent:** a `daily` plugin **must** fetch only what's
  missing (read its existing `_prices/*.txt`, fetch from the last recorded date
  forward, merge) and treat "nothing new" as a clean no-op — never a wholesale
  re-download or an error. This is what lets a single startup run backfill any
  gap, whether the app was closed for a day or a month.
- **One quote currency per file:** merging by date and rewriting the file in
  the plugin's own quote currency would relabel any row quoted in another
  (`GBP 1.9053 AUD` becoming `GBP 1.9053 USD`). A plugin about to write a file
  that holds such a row **must** leave it untouched and report the conflict as
  a warning; each shipped price plugin carries the same `quote_conflict` for
  this.

The price plugins shipped in this repo (`crypto-spot-prices`, `fiat-prices`,
`stock-prices`, `non-usd-prices`) are all marked `daily`.

## Script Interface

Your script receives a JSON object on **stdin** with everything it needs:

```json
{
  "sources_dir": "/path/to/sources",
  "plugin_dir": "/path/to/plugins/my-plugin",
  "data_dir": "/path/to/plugins/my-plugin/.data",
  "config": { "some_option": "hello", "count": 10 },
  "secrets": { "api_key": "sk-..." },
  "bin": { "emuacc_query": "/path/to/emuacc-query" }
}
```

The same values are also available as environment variables:
- `EMUACC_SOURCES_DIR`
- `EMUACC_PLUGIN_DIR`
- `EMUACC_PLUGIN_DATA_DIR`
- `EMUACC_QUERY_BIN` (absolute path to `emuacc-query`)
- `PATH` is prepended with the EmuAcc bin directory so bare `emuacc-query` also works.

### Querying the ledger

To read transactions, **always go through `emuacc-query`** — never walk `generated/` yourself. The CLI takes an optional base folder (defaults to the whole vault) and walks every `ledger.transactions` beneath it. See `emuacc-query --help` for the full search grammar.

```python
import json, os, subprocess
query_bin = ctx.get("bin", {}).get("emuacc_query") or os.environ["EMUACC_QUERY_BIN"]
r = subprocess.run(
    [query_bin, "commodity:HNT", "--format", "json"],
    capture_output=True, text=True, check=True,
)
txns = json.loads(r.stdout)["transactions"]
```

### Output

Write files directly into `sources/`:
- **Prices** → `sources/_prices/BTC.txt` (P-directive format: `P 2026-01-15 BTC 42000.00 USD`)
  - The file name is the commodity with filename-unsafe characters percent-encoded: an `EXCHANGE:SYMBOL`
    commodity such as `xyz:WTIOIL` lives in `_prices/xyz%3AWTIOIL.txt` (`:` is NTFS stream syntax on
    Windows and renders as `/` in Finder). The `P` line inside always carries the true commodity —
    the price graph keys on the line, not the file name.
- **Transactions** → `sources/{account-folder}/trades.csv` (processed by existing Rhai transforms)

Never write to `generated/`. The pipeline rebuilds it from `sources/` — any files you put there will be overwritten or stale.

Print a JSON status summary to **stdout**:

```json
{ "files_written": ["_prices/BTC.txt"], "records_fetched": 365, "warnings": [] }
```

### Exit codes

- `0` — success
- `1` — failure (stderr has error message)
- `2` — partial success (some data fetched, some failed)

### Persistent state

Use `data_dir` (the `.data/` folder) to store cursors, last-sync timestamps, or any state between runs. This folder is auto-created and git-ignored.

## Running Plugins

Click **Plugins** in the sidebar, then click **Run** on any plugin. Results (stdout/stderr) are shown in the main content area.

The "Update prices on startup" option runs every [`daily`](#daily--automatic-startup-backfill) plugin once per day. The same batch runs headlessly via the `emuacc-update` CLI (see [Keeping data fresh from cron](USER_GUIDE.md#keeping-data-fresh-from-cron)), so a timer can keep the vault current without launching the app.

## Included Plugins

The catalog bundled with the app is the repo's `plugins/`: the price plugins
below and the example. They all work against public APIs with no account or
API key (CoinGecko's free demo key is optional), and none carries personal
data. Each manifest's `[config]` block documents its options. The four price
plugins are marked `daily` and are incremental: they merge new dates into the
existing `sources/_prices/{SYMBOL}.txt` files rather than rewriting history,
and they leave a file holding prices in another quote currency untouched and
report it. The `fiat-prices` and `non-usd-prices` defaults write different
files.

### Crypto Spot Prices (`crypto-spot-prices`)

Daily crypto spot prices, one source per commodity tried in order — Binance →
CoinGecko → yfinance — taking the first that is *current* (it rejects data
frozen at a delisting and falls through). Wrapped/staked tokens (WETH, WBTC,
stETH, mSOL) are priced from their underlying. Backfills full history,
discovers the commodities you hold but have not declared, and writes
`_prices/{SYMBOL}.txt`. A free CoinGecko demo key (`coingecko_api_key`
secret) lifts the fallback's rate limit.

### Fiat Price Sync (`fiat-prices`)

Daily fiat/USD historical exchange rates from Yahoo Finance via `yfinance`
(tickers of the form `AUDUSD=X`). Writes `_prices/AUD.txt`, `_prices/EUR.txt`,
etc., quoted in USD.

**Config:**
- `currencies` — comma-separated codes (default `AUD,EUR,GBP`)
- `period` — yfinance period string (default `max`)

### Stock Yahoo Finance Price Sync (`stock-prices`)

Daily equity, ETF and US option prices via `yfinance` for every market you
hold: tickers are discovered under each broker account prefix (ASX `.AX`, US
bare tickers) and merged into one `_prices/{SYMBOL}.txt` per commodity.

### Non-USD Prices (`non-usd-prices`)

The price files that must be quoted in the vault's base currency rather than
USD, because valuation reaches the base currency by a direct price or *at most
one hop*: an AUD-base vault needs `USD.txt` quoted in AUD — the leg every
USD-priced commodity crosses — plus any holding whose own quote currency keeps
it one hop away (a GBP-quoted London listing converts via GBP, not via USD).
The USD leg is derived by inverting what `fiat-prices` writes, the rest come
from Yahoo, and only a recent window is ever rewritten, so a hand-built
history stays byte-identical.

**Config:**
- `invert` — `SYMBOL=QUOTE` pairs, each writing `SYMBOL.txt` quoted in `QUOTE`
  by inverting the `QUOTE.txt` that `fiat-prices` writes (default `USD=AUD`, an
  AUD base; set it to `USD=<your base>`)
- `yahoo` — `SYMBOL=TICKER` pairs for the legs your own holdings need (default
  none). A GBP-quoted London listing needs `GBP=GBPAUD=X`; take `GBP` out of
  `fiat-prices`' `currencies` first, since that writes `GBP.txt` quoted in USD
  and the two would otherwise target one file
- `forward_fill` — symbols carried onto every calendar day (default `USD`); add
  the FX legs you list under `yahoo`
- `refresh_days`, `period`, `decimals` — the rewrite window (default 7 days),
  yfinance period (default `max`) and decimal places (default 4)

### Hello World (`hello-world`)

A minimal plugin that demonstrates the interface without calling any external
APIs. See `plugins/hello-world/`.

### Transaction sources

The private repository EmuAcc is developed in also holds transaction-source
plugins — wallet and exchange history (`crypto-sync`), Interactive Brokers
through the Flex Web Service (`ibkr-flex-sync`), bank and card accounts
through a SimpleFIN server (`simplefin-sync`) — and a few labelling and tax
helpers. Those need per-provider tokens and are tuned to one person's
accounts, so they are not published. A transaction source of your own is a
short script against the [contract above](#script-interface): seed the
boundary from the files already in the folder (CSV dates, OFX `DTPOSTED`) so
a manual export and the sync never overlap — the export owns history up to
the cut-over, the sync owns everything after.

## Writing Your Own Plugin

1. Create a folder in `plugins/` with a `plugin.toml` and a Python script
2. Read config from `json.load(sys.stdin)`
3. Do your work (fetch data, transform, etc.)
4. Write output files to `sources_dir`
5. Print a JSON summary to stdout
6. Exit with code 0 (success), 1 (failure), or 2 (partial)

The runner invokes Python via `uv run --script` when the script has a PEP 723
header and `uv` is on PATH; otherwise it falls back to bare `python3`. Any
language works as long as it reads JSON from stdin and writes files — but
the runner currently only knows how to launch Python scripts.
