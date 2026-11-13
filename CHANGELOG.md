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

## 0.1.5 - 2026-11-14

This release adds a Linux download for arm64 machines. The app and its
installers are version 0.1.5, and `generated/generator.json` records 0.1.5 as
the version that wrote a data folder. (TASK-171)

### New

- An arm64 Linux `.deb`, `EmuAcc_0.1.5_arm64.deb`, beside the x86_64 downloads. It installs the app and its command-line tools, so an arm64 Linux machine such as a Raspberry Pi can keep a data folder up to date from cron with `emuacc-update` and search it with `emuacc-query`, as [Keeping data fresh from cron](https://github.com/emuacc/emuaccounts/blob/v0.1.5/USER_GUIDE.md#keeping-data-fresh-from-cron) describes. There is no arm64 AppImage; [Linux (Debian / Ubuntu)](https://github.com/emuacc/emuaccounts/blob/v0.1.5/USER_GUIDE.md#linux-debian--ubuntu) says which file to install. (TASK-171)

### Breaking changes

None.

## 0.1.4 - 2026-11-13

This release makes the app feel live and polishes what you see. Edits in the
transaction table show at once, a new account appears in the sidebar and the
account lists as soon as you save it, a hidden transaction can be unhidden,
and fast typing in the search bar keeps up. An edit rebuilds only the account
folder it touches and no longer regenerates every report. The Categories pane
holds only income and expenses, the app reads in one font, a Linux dock shows
the emu, and a day priced in two currencies converts through its direct quote.
The app and its installers are version 0.1.4, and `generated/generator.json`
records 0.1.4 as the version that wrote a data folder. (TASK-143)

Getting started, in one take recorded from this release — choose a data
folder and add an account, import a CSV whose rows carry their categories, add
a manual transaction, see what was earned and spent in the Categories pane,
then open the ledger text file the pipeline wrote from all of it:
[watch the recording](https://github.com/emuacc/emuaccounts/blob/v0.1.4/docs/demos/getting-started.mp4)
(the README plays it as a looping GIF). (TASK-145)

### New

- Clearing accounts, the accounts money only passes through between two of yours (`assets:transfer`, a bridge between two chains): list them under **Global Settings → Clearing accounts**, saved with the data folder (`assets:transfer` when you list none). **Issues → Unbalanced clearing account** shows each one whose postings in a commodity don't net to zero across the data folder, with the amount left over, usually the other half of a transfer that was never imported or matched; `emuacc-issues --unbalanced-clearing` prints the same list. **Global Settings → Show all holdings** adds the holdings that have no source folder of their own (staking, lending, a card only rules post to) to the Accounts pane, in their groups. It changes only what is shown: the header, the group totals and the Balances report still count what **Accounts included in balance** lists, and a holding outside that list shows its own value, muted. See [Clearing accounts](https://github.com/emuacc/emuaccounts/blob/v0.1.4/USER_GUIDE.md#clearing-accounts) and [Holdings with no source folder](https://github.com/emuacc/emuaccounts/blob/v0.1.4/USER_GUIDE.md#holdings-with-no-source-folder). (TASK-141)
- Changes you make in the transaction table show at once: re-categorising a row from its category cell or with a rule, renaming a party from its payee cell or the party editor, and hiding or deleting a transaction update the table before the rebuild that applies them has finished. When the rebuild lands the table updates in place, keeping its scroll position and expanded rows instead of reloading, and rows that changed fade in. A row whose change did not apply goes back to what the data folder holds, flashes amber, and the status line says the change did not apply. **Show Ignored** updates the table the same way: turning it off takes the ignored rows out at once, and turning it on fades them in, without reloading the table. **Unhide** is new: a transaction you hid, shown under **Show Ignored**, has an ↺ button where ✕ was, and clicking it puts the transaction back in its account, at once like the other edits. Before, there was no button to bring a hidden transaction back. (TASK-149)

### Fixed

- An account folder's `summary.json` in `generated/` includes the opening balance declared in that folder's `accounts.transactions`, so a script or a person reading it sees the balance the app and `emuacc-query` show. Before, it summed the transactions only: an account opened at 1,823.92 AUD with 680.10 AUD of transactions read 680.10 there, while the app showed 2,504.02. The openings it includes are also listed under `openings`. The first launch after upgrading rebuilds the data folder once; only the summaries of folders that declare an opening change. (TASK-052)
- `emuacc-import` and `emuacc-gaps` work on the data folder the app has open, as the other command-line tools do. Before, they looked for it in a settings folder left from before the rename (`com.emuacc.app`), so without `EMUACC_SOURCES_DIR` and `EMUACC_GENERATED_DIR` set they pointed at folders that no longer exist. `emuacc-import` also books unmatched spending to the default expense account from the app's settings, as the app does, instead of always `expenses:unknown`. The [command line](https://github.com/emuacc/emuaccounts/blob/v0.1.4/USER_GUIDE.md#command-line) section states the rule every tool follows. (TASK-088)
- A commodity with prices in more than one currency on the same date (an AUD and a USD `P` line for that day) is valued in your base currency from the direct quote, whatever order the lines are in, by the reports, the capital gains figures and the pipeline's swap pricing; it converts through another currency only when that day has no direct quote. Before, whichever line came last decided: 10 HNT priced at 4.00 AUD and 2.70 USD could be valued at 40.50 AUD through the USD rate instead of 40.00, or not at all when that currency had no rate to the base. `emuacc-query --min-value-usd` had the same fault. (TASK-110)
- In the Categories pane, selecting **income** lists only income: tax payments booked to `expenses:tax:income` no longer appear under it, labelled with a bank account's name cut short. Each row's sub-account is named relative to the category you selected. Income earned now reads as money in, in green, and a loss on an income account, such as a foreign-exchange loss, as money out, in red; group totals follow. (TASK-139)
- On Linux, the AppImage shows the emu in a Wayland dock or taskbar, such as Waybar on Hyprland, instead of a generic cog. Run from an AppImage, EmuAcc adds itself to your applications menu: it keeps `~/.local/share/applications/emuaccounts.desktop` and its icons current, rewriting them only after you move or update the AppImage ([how to remove them](https://github.com/emuacc/emuaccounts/blob/v0.1.4/USER_GUIDE.md#linux-other)). On X11 the window icon is the 256-pixel emu, no longer 32 pixels. (TASK-140)
- Global Settings opens with your saved settings even when prices are not updated at startup. Before, those settings loaded only with the plugin list, so the dialog could open with an empty **Accounts included in balance** list, and saving it then wiped the saved list. A category's total values every commodity in it in the base currency (a crypto income category counted only some of them), and the account header no longer shows the total of an account you clicked away from before it arrived. (TASK-141)
- Text across the app is in one font, with one monospace font for the search box, patterns, ids and code, at one set of sizes and weights. Before, buttons, fields, the toolbar and the sidebar's navigation and folders were drawn in the system's control font (on Linux the GTK theme's font at its own size, on macOS an 11-pixel system font), and dialogs and reports fell back to the browser's 16-pixel default with heavier titles. Dialog fields are now 13 pixels, dialog titles 18 and report titles 22. (TASK-142)
- A new account shows in the sidebar's Accounts and Categories panes and in every account list you pick from as soon as you save it (a category typed into a transaction's category cell or a rule, or the other account of a manual transaction), and its totals fill in when the rebuild finishes. Before, a category a rule created stayed out of the sidebar and the account lists until a source file changed or the app restarted, and an account a manual transaction or an import created could be missing from the account lists. (TASK-148)
- Hiding a transaction you had re-categorised from its category cell hides it. Before, when the transaction had no per-leg id, the hide was silently skipped. (TASK-149)
- Typing quickly in the search bar keeps every character in order and the cursor where you are typing. Before, the results arriving for what you had typed so far put the cursor back where it was when that search started, so the next keys went into the middle of the word ("office" could come out as "ofcefi"); the rule editor's search box did the same. A cursor you move, or another field you click into, while a search is running stays where you put it. (TASK-150)
- A long report scrolls to the bottom and back up normally. Before, near the bottom of a report the app kept loading the selected account's transactions in the background and putting the scroll back after each page, so the report stuck to the bottom and scrolling up jumped straight back down until they had all loaded. Scrolling up in a transaction table while its next page loads also stays where you put it. (TASK-151)
- The list of account suggestions closes when you leave its field or the field goes away. Before, clicking back into a field where you had already chosen an account and then leaving it, or going back with Alt+Left while typing, could leave the list on screen until you reloaded the app, moving to the window's top-left corner on the next scroll. In a list reopened on a field you had already chosen for, Escape, Enter and clicking a suggestion did nothing; they now work. (TASK-153)
- A change you make in the transaction table, such as re-categorising a row or adding a rule, rebuilds only the account folder it touches and leaves the rest of the data folder as it is on disk. Before, every edit re-read the whole data folder and then regenerated every report in the background: on a data folder of about 60,000 transactions a rule change took 3 seconds to rebuild and set off 19 seconds of report generation. It now rebuilds in under a second, with no report generation. A report whose saved copy is out of date is worked out when you open it, and the Issues list and the trade-link suggestions are worked out once after a run of edits, not after each one. An edit in a folder that holds trades still re-reads the rest of the data folder, because a trade can pair with one in another folder. (TASK-155, TASK-156, TASK-157, TASK-158, TASK-159, TASK-160, TASK-161)
- An expanded trade row no longer offers **Edit Rule**, and a linked swap row no longer offers **Edit Rule** or **Edit Party**. A trade has no single category for a rule to change and a swap row merges two transactions, so these buttons did nothing when clicked. **Edit Party** still works on a trade row. (TASK-164)
- After you add a manual transaction, the status line says **Added manual transaction** once it has been saved. Before, it stayed on **Adding transaction…** after the save had finished. (TASK-168)
- The report you choose in Reports is the one shown, even when you choose it while another report is still loading. Before, choosing one before Capital Gains Tax, which Reports opens on, had finished could leave Capital Gains Tax on the page with the report you chose highlighted in the menu, until you chose it again. Applying a custom date range or an account scope now shows the loading screen while the report generates, as choosing a year does. (TASK-167)

### Breaking changes

- The Categories pane lists only income and expenses: assets, liabilities and equity have left it, and equity accounts and clearing accounts (`assets:transfer` and its sub-accounts unless you list others) appear in neither sidebar pane. A holding with no source folder of its own, which only the Categories pane listed before, shows in the Accounts pane once **Global Settings → Show all holdings** is on. Clearing accounts no longer appear under **Issues → Unverified Balance**; an imbalance in one is listed under **Unbalanced clearing account** instead. (TASK-141)
- In `generated/`, the `balances` of an account folder's `summary.json` now include the opening balance declared in that folder's `accounts.transactions`. A script of your own that added the openings to those figures counts them twice: drop that step, or subtract what the new `openings` field lists. (TASK-052)

## 0.1.3 - 2026-10-26

This release irons out installing: a download opens on macOS again, and the
Linux AppImage opens on distributions with a current graphics stack. The app
and its installers are version 0.1.3, and `generated/generator.json` records
0.1.3 as the version that wrote a data folder. (TASK-136)

Getting started, in one take recorded from this release — choose a data
folder and add an account, import a CSV, add a manual transaction, then open
the ledger text file the pipeline wrote from all of it:
[watch the recording](https://github.com/emuacc/emuaccounts/blob/v0.1.3/docs/demos/getting-started.mp4)
(the README plays it as a looping GIF). (TASK-138)

### New

- EmuAcc's icon is the emu from the top of the sidebar, on the sidebar's navy, in place of the plain blue square: in the Dock or taskbar, on the installers, and on the [phone view](https://github.com/emuacc/emuaccounts/blob/v0.1.3/docs/PWA.md) added to a home screen. The 0.1.2 downloads already carried it, but their notes left it out. (TASK-123)

### Fixed

- The Linux AppImage opens on distributions with a current graphics stack, such as Arch and Fedora 44 (Mesa 26). Before, it shipped its own older copy of the Wayland client library, and the system's graphics driver failed to load against it: the app's web view aborted at launch ("Could not create default EGL display: EGL_BAD_PARAMETER") and the window stayed blank. The AppImage now uses the system's copy, and on a Wayland desktop it runs as a native Wayland window instead of through XWayland. (TASK-134)
- On macOS, a downloaded EmuAcc opens again. macOS refused 0.1.2 with "“EmuAcc” is damaged and can't be opened", with no way past it, because the app as a whole carried no valid signature. The app is now signed, so macOS asks once instead: click **Open Anyway** in System Settings > Privacy & Security, as the [install steps](https://github.com/emuacc/emuaccounts/blob/v0.1.3/USER_GUIDE.md#macos) describe. `EmuAcc.app.zip` opens too; in 0.1.2 the programs inside it had lost their permission to run. (TASK-135)

### Breaking changes

None.

## 0.1.2 - 2026-10-24

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
