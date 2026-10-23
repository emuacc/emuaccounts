# User guide

Installing EmuAcc, using the app, and the command-line tools that do the same
work without it. The overview is in the [README](README.md).

## Install

Download the latest release from the [Releases](../../releases) tab. What
changed in each release is in [CHANGELOG.md](CHANGELOG.md).

### macOS

1. Download the `.dmg` file
2. Open the disk image and drag **EmuAcc COVID** to Applications
3. On first launch, macOS may block the app. Go to **System Settings > Privacy & Security** and click **Open Anyway**

Alternatively, download the `.app.zip`, unzip it, and move the `.app` to Applications.

### Windows

Download **either** installer:

- `.msi` — standard Windows Installer (add/remove programs support)
- `.exe` — NSIS installer

Run the installer and follow the prompts.

### Linux (Debian / Ubuntu)

```sh
sudo dpkg -i emuacc-covid_*_amd64.deb
```

### Linux (Arch)

Use the `.AppImage` (below), or convert the `.deb` with [`debtap`](https://aur.archlinux.org/packages/debtap):

```sh
debtap emuacc-covid_*_amd64.deb
sudo pacman -U emuacc-covid-*.pkg.tar.zst
```

### Linux (other)

The `.AppImage` runs on any distribution with FUSE support:

```sh
chmod +x emuacc-covid_*_amd64.AppImage
./emuacc-covid_*_amd64.AppImage
```

## Using the app

- **Every account in one ledger** — bank, broker, exchange and wallet exports (CSV, OFX, or a PDF statement read by the AI assistant) compiled into one double-entry ledger.
- **Reports for tax time** — capital gains with FIFO lot matching and discount rules, income, balances, performance and a loss-harvesting view, every figure traceable to the lines that produced it. The defaults are Australian; the tax year and the CGT discount are configurable.
- **Data that keeps itself current** — [plugins](PLUGINS.md) fetch daily prices, and a transaction source for a wallet, exchange, broker or bank is a short script against the same contract.
- **Sync between devices** through a relay you host yourself ([Multi-device sync](#multi-device-sync)), and a read-only [phone view](docs/PWA.md).
- **A command line for everything the app does** — see [Command line](#command-line).

### The main window

- Click `Import` to bring a CSV or OFX file into the selected account (EmuAcc suggests a column mapping). Other documents — PDF or text bank/dividend statements — are interpreted by the AI into a normalised CSV behind a preview‑and‑confirm step; the original is kept in the account folder's `originals/`. The same path runs headlessly as [`emuacc-ai-import`](#importing-a-pdf-or-text-statement).
- The left sidebar lists accounts and their running balances (per commodity, preferring `USD` for display).
- The main table shows transactions affecting the selected account, with `Payment` / `Deposit` split from the selected account’s posting amount.
- The **Plugins** view runs the [plugins](PLUGINS.md) that fetch prices and sync transactions.
- On your phone, the read-only web client serves the same reports: [docs/PWA.md](docs/PWA.md).

### Import a CSV

<video controls muted loop playsinline width="720" poster="docs/demos/import-csv-poster.png">
  <source src="docs/demos/import-csv.mp4" type="video/mp4" />
  <source src="docs/demos/import-csv.webm" type="video/webm" />
</video>

### Add a manual transaction

<video controls muted loop playsinline width="720" poster="docs/demos/manual-txn-poster.png">
  <source src="docs/demos/manual-txn.mp4" type="video/mp4" />
  <source src="docs/demos/manual-txn.webm" type="video/webm" />
</video>

### Terminology

Some nouns in the UI are configurable. **Global Settings → Terminology** lists every term from
`src-tauri/terminology.json` with its description and default wording; type a replacement
singular/plural or leave the fields blank to keep the default. The default for one holding is
*asset* / *assets* (a coin, a share, a currency, a property…), used by the Balances and Tax
Savings reports. Overrides are saved under `terms` in the app `config.json`.

### AI assistant

AI features (document import, transform generation, categorisation suggestions) talk to any
OpenAI‑compatible endpoint configured in **Global Settings → AI assistant** (API URL, key, model,
reasoning effort, your name). For LM Studio use `http://localhost:1234/v1` with a blank key.

> **Known issue — reasoning effort with Qwen on LM Studio (as of Aug 2026):** the app sends
> `reasoning_effort` with every request and LM Studio validates it, but for Qwen 3.8 builds that
> lack "custom reasoning fields" metadata (e.g. `lmstudio-community/Qwen3.8-27B-MLX-8bit`) LM Studio
> cannot bridge the value to the model — it logs
> `No valid custom reasoning fields found … cannot be converted to any custom KVs` and the model's
> own chat template default applies instead (**upstream default: `xhigh`**, i.e. maximum thinking —
> very slow). `chat_template_kwargs` and URL parameters are also dropped before the Jinja render,
> and the 3.8 template has no `/think`/`/no_think` tag handling. **Workaround:** edit the model's
> `chat_template.jinja` (LM Studio: My Models → model → Prompt tab, or the file under
> `~/.lmstudio/models/<publisher>/<model>/`) and change
> `reasoning_effort|default('xhigh')` to `default('medium')` (or `'low'`), then reload the model.
> Re‑downloading the model overwrites the edit. The in‑app setting should start working
> per‑request once LM Studio bridges the parameter — retest after LM Studio updates by watching
> for that WARN line to disappear.

## Multi-device sync

EmuAcc uses a self-hosted relay server (`emuacc-relay`) to sync between devices.

The app is **local-first** — all changes are saved immediately to your local Automerge document and content store, regardless of whether the relay server is running. The relay is purely optional and only used when you click **Sync Now**. If the relay is unavailable, you'll see an error but no data is lost. When the relay comes back online, the next sync performs a full reconciliation and all offline changes from every device are merged automatically.

### Run the relay server

The relay binary is included in each release. Download it for your platform from the [Releases](../../releases) tab, then run:

```sh
emuacc-relay
```

By default it listens on `0.0.0.0:8384` and stores data in `/tmp/emuacc-relay`.

**Options:**

```
--bind <ADDR>      Bind address (default: 0.0.0.0:8384)
--data-dir <PATH>  Data directory (default: /tmp/emuacc-relay)
```

Example with a persistent data directory:

```sh
emuacc-relay --bind 0.0.0.0:8384 --data-dir /var/lib/emuacc-relay
```

### Pair devices

1. On one device, click **Pair Device** in the sidebar and choose **Create** — note the 6-digit code.
2. On the other device, click **Pair Device**, choose **Join**, and enter the code.
3. Click **Sync Now** on either device to exchange data.

### Data file locations

All app data lives under the platform app-data directory:

| Platform | Path |
|----------|------|
| macOS    | `~/Library/Application Support/com.cog32.emuacccovid/sources/` |
| Linux    | `~/.local/share/com.cog32.emuacccovid/sources/` |
| Windows  | `%APPDATA%\com.cog32.emuacccovid\sources\` |

Key files:

- `emuacc-metadata.automerge` — CRDT sync metadata (delete to reset sync state)
- `relay-config.json` — paired relay URL and group ID (delete to unpair)
- `cas/` — the relay blob store

These three are each device's relay sync state, not pipeline input: changing
them neither triggers a rebuild nor defeats the pipeline's early exit. If the
vault is a git repository, ignore them there; the relay keeps devices in step.

## Command line

The app is a thin client over the same library as the `emuacc-*` command-line
tools, so each of these does what the app does, without a window open:
`emuacc-regenerate` builds the ledger the app builds, `emuacc-query` answers
the searches the app's search box answers, `emuacc-reports` produces the
reports the app shows, `emuacc-web` serves the read-only [phone client](docs/PWA.md),
and `emuacc-relay` is the [sync relay](#run-the-relay-server).

### Keeping data fresh from cron

`emuacc-update` refreshes the configured vault without launching the app: it
runs every daily-flagged plugin (the same batch as "update prices on
startup"), then rebuilds the generated ledger and reports. It is
non-interactive and cron-safe — exit 0 on success, non-zero when any plugin
or the rebuild fails, per-plugin outcomes on stdout, failures on stderr. The
once-per-day guard is shared with the app, so a second run on the same day
(or an app launch after a cron run) skips the fetch and still exits 0.

The vault resolves like the other CLIs: `EMUACC_SOURCES_DIR` /
`EMUACC_GENERATED_DIR` env overrides win, otherwise the app config's
`current_root`. `--force` re-runs plugins that already succeeded today;
`--sources-dir` / `--generated-dir` override the directories explicitly.

Build it once with `cargo build --release --bin emuacc-update` (from
`src-tauri/`), then schedule it.

Example crontab entry (weekdays at 07:30, log to a file):

```cron
30 7 * * 1-5 $HOME/.local/bin/emuacc-update >> $HOME/Library/Logs/emuacc-update.log 2>&1
```

Example launchd agent (macOS) — save as
`~/Library/LaunchAgents/com.cog32.emuacc-update.plist` and load with
`launchctl load ~/Library/LaunchAgents/com.cog32.emuacc-update.plist`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>com.cog32.emuacc-update</string>
  <key>ProgramArguments</key>
  <array><string>/Users/you/.local/bin/emuacc-update</string></array>
  <key>StartCalendarInterval</key>
  <dict><key>Hour</key><integer>7</integer><key>Minute</key><integer>30</integer></dict>
  <key>StandardOutPath</key><string>/Users/you/Library/Logs/emuacc-update.log</string>
  <key>StandardErrorPath</key><string>/Users/you/Library/Logs/emuacc-update.log</string>
</dict>
</plist>
```

### Importing a PDF or text statement

`emuacc-ai-import` is the Import button's AI path without the app: it extracts
the document's text, has the model configured in Global Settings turn it into a
normalised CSV, re-validates every row in Rust (dates, amounts, the
running-balance chain and the folder's transform), and prints that CSV on
stdout. Nothing is written unless `--confirm` is passed; then the CSV is staged
through `imports/` into the account folder, the original is filed under
`originals/`, and the ledger and reports are rebuilt. `--confirm` needs a
transform already in place for the folder, so bring a new folder's first
document in through the app, which writes it.

```sh
emuacc-ai-import --account alex/bank/cdia ~/Downloads/Statement.pdf             # preview only
emuacc-ai-import --account alex/bank/cdia ~/Downloads/Statement.pdf --confirm   # import
```

The vault resolves like the other CLIs (`EMUACC_SOURCES_DIR` /
`EMUACC_GENERATED_DIR`, else the app config's `current_root`), and
`EMUACC_APP_DATA_DIR` points it at a different app config. `--currency`
overrides the vault base currency assumed in the prompt. Progress and warnings
go to stderr, so `> preview.csv` captures just the rows. Exit codes: 0 on
success, 1 when extraction, validation or the import fails, 2 for usage errors.
