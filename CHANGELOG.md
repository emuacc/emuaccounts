# Changelog

What changed for a user of EmuAcc, newest release first, one section per
release. Each line ends with the task it came from. A release lists its new
features, fixes and breaking changes under their own headings; the breaking
changes heading is always present, with "None." when there are none, so an
upgrader never has to guess.

A release's section is also the body of its GitHub release:
`scripts/release_notes.py <version>` extracts it when the `v<version>` tag is
built (`.github/workflows/build.yml`), and `scripts/release_ready.py` refuses
to hand out the tag command while the section is missing. Use absolute links
inside a section: it is read on the release page as well as in this file.

## 0.1.2 - unreleased

This release removes obvious errors: a window that froze while a large data
folder rebuilt, label edits that could wipe a folder's labels, and price files
that mixed quote currencies. The app is now called EmuAcc in every name it
shows, and multi-device sync leaves the released app. The app and its
installers are version 0.1.2, and `generated/generator.json` records 0.1.2 as
the version that wrote a data folder. (TASK-130)

Getting started, in one take recorded from this release — choose a data
folder and add an account, import a CSV, add a manual transaction, then open
the ledger text file the pipeline wrote from all of it:
[watch the recording](https://github.com/emuacc/emuaccounts/blob/v0.1.2/docs/demos/getting-started.mp4)
(the README plays it as a looping GIF). (TASK-132)

### New

- The tagline under the EmuAcc name at the top of the sidebar reads "Understand your finances", the same words as the README headline (it said "Understand your accounts"). (TASK-127)
- The demo recordings play by themselves, as looping GIFs: the getting-started take in the README, and the CSV import and manual transaction in the user guide, re-recorded to show the app as it is now. Before, each was a still image to click through to the video. The full-resolution MP4 and WebM are linked under each one. (TASK-129)
- Trade links are kept in `sources/_trade_links.json`: plain JSON, one entry per link sorted by id, so in a vault under git a new or removed link shows as a small diff. The first launch copies the links from `sources/emuacc-metadata.automerge` into it and leaves that file as it was; commit the new file. Keep `_trade_links.json` once it exists: while the old file is still there, deleting it copies the links again, and any you removed since come back. (TASK-133)

### Fixed

- The window no longer freezes ("Not Responding") while a large data folder rebuilds, at launch or after you choose another folder: a "Rebuilding cache" screen shows until the rebuild ends, then the app opens as before. Other slow work, such as gathering the issues list, no longer freezes the window either. (TASK-125)
- Saving, editing or deleting a payee label no longer rewrites a folder's `_labels.json` when that file cannot be read or parsed (a trailing comma from a hand edit, conflict markers left by a sync). The app reports the broken file and leaves it untouched; before, saving replaced the file with the one new label, so every other payee and commodity label in the folder was lost, and editing or deleting said the label was not found. (TASK-111)
- `emuacc-query`, `emuacc-reports`, `emuacc-regenerate`, `emuacc-issues`, `emuacc-dedupe`, `emuacc-migrate-rules`, `emuacc-repair-trade-links` and `emuacc-web` read the app settings from the folder `EMUACC_APP_DATA_DIR` names, as `emuacc-update` and `emuacc-ai-import` already did; before, they always read the default settings folder. (TASK-128)
- The bundled price plugins no longer mix quote currencies in one price file. With their default settings, Fiat Price Sync wrote `_prices/GBP.txt` quoted in USD and Non-USD Prices wrote the same file quoted in AUD, so its rows switched currency from date to date. Each price plugin now leaves a file that already holds prices in another quote currency untouched and reports it, and Non-USD Prices by default writes only `USD.txt`: list the legs your own holdings need under its `yahoo` setting. A plugin already added to a vault keeps its installed copy, since the app adds plugins but does not update them. (TASK-115)

### Breaking changes

- The app is called EmuAcc everywhere: the window title, the installed app and the installer files no longer say "COVID" (`EmuAcc.app`, `EmuAcc_<version>_amd64.deb`). Its bundle identifier is now `com.emusoftware.emuaccounts`, so the system sees a new app: remove the old "EmuAcc COVID" from Applications after installing. The app also keeps its settings in a new folder named after that identifier (see [Data file locations](https://github.com/emuacc/emuaccounts/blob/main/USER_GUIDE.md#data-file-locations)), so on its first launch it asks for your data folder again; choose the same folder, nothing in it changes. To keep your other settings (the AI assistant, the default expense account), move `config.json` from the old `com.cog32.emuacccovid` folder into the new one before launching. (TASK-128)
- Multi-device sync is no longer part of the app. It was unfinished: the sidebar's Sync section (Pair Device, Sync Now, View Sync Log) is gone, and releases no longer include the `emuacc-relay` server. It is now an opt-in developer build ([docs/SYNC.md](https://github.com/emuacc/emuaccounts/blob/v0.1.2/docs/SYNC.md)). Trade links are unaffected. (TASK-126)

## 0.1.1 - 2026-10-23

The initial release: plain-text, local-first accounting for every account you
hold, compiled into one ledger that you own, with reports you can audit line
by line. Since the 0.1.0 build:

Getting started, in one take — choose a data folder and add an account, import
a CSV, add a manual transaction, then open the ledger text file the pipeline
wrote from all of it:
[watch the recording](https://github.com/emuacc/emuaccounts/blob/v0.1.1/docs/demos/getting-started.mp4)
(re-recorded for every release; the README links it from its poster).

### New

- The README opens with the philosophy: the north star, the principles the code holds to (files over app, local-first, a ledger that is compiled rather than edited, one source of truth written back, deterministic and diffable output, a format that is a specification, a UI that is a thin client over the CLI, extension at the edges), what makes it a base for scripts and AI agents, who it is for, and its status. (TASK-096)
- Release notes: this changelog, with each release's section published as the body of its GitHub release. (TASK-097)

### Fixed

- Reports no longer count a ledger twice when `generated/` holds a stale or differently-cased copy of an account folder, so macOS and Linux give the same figures. The pipeline removes such folders and reports what it removed, and a transaction that appears in two files is reported rather than counted twice. (TASK-093)
- Regenerating a vault changes only the files a data change affects: report figures are rounded at the output, holdings and attribution lists are written in a fixed order, each year's capital gains report carries only its own warnings (the vault-wide list is `reports/cgt-warnings.md`), and `generated/generator.json` records the build that wrote the vault. A vault that tracks `generated/` in git sees one reordering diff on the first regeneration, then only its data changes. (TASK-094)
- The transaction list values a leg that carries its own price in another currency at that price converted at the day's rate, not quantity × the commodity's daily close, so a broker fill matches the capital gains report to the cent. The Value cell's tooltip states the basis, Set Price opens on the rate the value used, and grouped rows and the newest/oldest flip no longer read "—". (TASK-095)
- An on-chain swap whose hash is shared with a 0-amount contract-interaction leg is priced from its real partner leg, never from that leg or from itself, and a partner worth nothing no longer stamps a zero price, so the capital gains report does not show a zero cost base or zero proceeds for such swaps. (TASK-103)
- Two entities with the same account folder (`alex/cash/bank/cba` and `acme/cash/bank/cba`, both the account `assets:cash:bank:cba`) each keep their own ledger and account declaration: `pipeline-metadata.json` now records folders per entity, so neither ledger is skipped as an orphan and the Delete and reveal actions resolve the selected entity's folder. (TASK-107)
- A search's accounts and balances cover every transaction it matches, whatever page of the list is showing: the transaction list header no longer changes its accounts and totals as you scroll, and `emuacc-query --balances` reports the same figures with or without `--limit`. (TASK-108)
- Merge-importing prices in one currency, or setting a price from the transaction list, replaces only the quote for that date in the same currency: quotes in other currencies on the same dates (an AUD quote beside a USD one) survive instead of being deleted, so the conversions and capital gains that relied on them keep their values. (TASK-101)
- Several trade legs booked at one second in one exchange account across more than two commodities (a two-hop convert such as BTC to USDT to ETH, or BTC-for-ETH and ETH-for-BTC fills at the same second) now link each sale to the purchase of a different commodity it belongs to, never to a leg of the same commodity, so the capital gains report prices each disposal from what was actually received. (TASK-105)
- A search date bound given as a month or a year now covers that whole period: `date:2025-01..2025-03` and `date:<=2025-03` keep every transaction in March 2025 and `date:<=2025` keeps all of 2025, where before the final month or year was silently dropped from the list, its counts and its balances. `date:<2025-03` ends with February and `date:>2025-03` starts in April, in the search box and in `emuacc-query` alike. (TASK-109)
- Hiding a transaction, or saving, moving or deleting a rule or trade link, no longer rewrites a folder's `_rules.json` when that file cannot be read or parsed (a trailing comma from a hand edit, conflict markers left by a sync). The app reports the broken file and leaves it untouched; before, it replaced the file with the one new rule and every other rule in the folder was lost. The `emuacc-classify`, `emuacc-migrate-rules` and trade-link repair tools skip such a folder and say so. (TASK-100)
- A vault with several entities whose base currencies differ (one on AUD, another on USD in its own `config.json`) prices each entity's on-chain swaps in that entity's currency: before, every entity's swap legs were stamped in the alphabetically first entity's currency, so the other entities' ledgers and capital gains figures came out in the wrong currency. Two entities trading from a same-named wallet account at the same second also no longer have their legs linked to each other. (TASK-104)
- A transaction hidden through the legacy `sources/_ignored.txt` no longer takes others with it: the listed id must now equal a whole segment of a transaction's meta, so the other leg of an auto-linked swap (whose meta names the hidden leg as its partner) and any transaction whose id merely starts with the hidden one stay in the ledger and the balances. (TASK-102)
- The first account in a brand-new data folder can be added: with no account set yet, the Add Account form starts on "New entity" and asks for its name instead of refusing with "Entity name is required." (TASK-098)

### Breaking changes

None.

## 0.1.0 - 2026-02-09

The first tagged build, the one that moved the app to Tauri 2. No notes were
kept for it.
