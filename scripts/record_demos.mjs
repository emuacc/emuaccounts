#!/usr/bin/env node
/**
 * record_demos.mjs — generate the muted screencasts embedded in the README.
 *
 * Records three flows of the real Tauri app, driving the WebView with
 * selenium-webdriver (via the macOS `tauri-wd` W3C WebDriver server) and the
 * native OS file/folder dialogs with `osascript`. The screen is captured with
 * macOS `screencapture -v`, then cropped to the app window and encoded to
 * MP4 + WebM with ffmpeg.
 *
 * `getting-started` is the README's getting-started video: one continuous take
 * of all three flows (choose a data folder and add an account, import a CSV,
 * add a manual transaction) in a single app session and one throwaway vault,
 * ending on the account's generated ledger file in TextEdit (a second capture,
 * joined with a hard cut). It is re-recorded for every release — see
 * docs/demos/RECORDING.md.
 *
 *   node scripts/record_demos.mjs --flow manual-txn
 *   node scripts/record_demos.mjs --all
 *   node scripts/record_demos.mjs --flow import-csv --keep-mov
 *   node scripts/record_demos.mjs --flow manual-txn --no-record   # drive only
 *
 * Requirements (one-time): the controlling terminal needs macOS TCC grants —
 * Accessibility + Automation (for osascript), Screen Recording (for
 * screencapture). See docs/demos/RECORDING.md. The app binary must be built
 * with the webdriver feature: run `scripts/run_debug_mcp.sh`.
 *
 * macOS only (uses screencapture/osascript). See AD_HOC_OR_SCRIPT.md process.
 */

import { spawn, spawnSync } from "node:child_process";
import { createRequire } from "node:module";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

const require = createRequire(import.meta.url);
const webdriver = require("selenium-webdriver");
const { Builder, By, Capabilities, until } = webdriver;

const repoRoot = path.resolve(path.dirname(new URL(import.meta.url).pathname), "..");

// ---------------------------------------------------------------------------
// Config / paths
// ---------------------------------------------------------------------------
const WD_URL = "http://127.0.0.1:4444/";
const WD_PORT = 4444;
const OUT_DIR = path.join(repoRoot, "docs", "demos");
// A clean throwaway folder for the import demo, so the file dialog never reveals
// repo fixtures or personal folders. Staged fresh before the flow runs.
const DEMO_IMPORT_DIR = "/tmp/test";
const DEMO_CSV = path.join(DEMO_IMPORT_DIR, "transactions.csv");
const DEMO_CSV_BODY = "Date,Description,Amount\n2025-01-15,Coffee Shop,-4.50\n2025-01-16,Salary,3500.00\n";

// Real app config.json — protected: only flow 1 moves it aside, always restored.
const APP_DATA = path.join(os.homedir(), "Library", "Application Support", "com.cog32.emuacccovid");
const CONFIG = path.join(APP_DATA, "config.json");
const CONFIG_BAK = path.join(APP_DATA, `config.json.demobak-${process.pid}`);

const HOME = os.homedir();
const APP_BINARY_CANDIDATES = [
  path.join(HOME, ".cache", "emuacc-target-mcp", "debug", "emuacc-covid"),
  path.join(repoRoot, "src-tauri", "target", "debug", "emuacc-covid"),
];

// ---------------------------------------------------------------------------
// Args
// ---------------------------------------------------------------------------
function parseArgs(argv) {
  const a = { flows: [], record: true, encode: true, keepMov: false, pace: 1600, width: 1200 };
  for (let i = 0; i < argv.length; i++) {
    const t = argv[i];
    if (t === "--all") a.flows = ["getting-started", "import-csv", "manual-txn"];
    else if (t === "--flow") a.flows.push(argv[++i]);
    else if (t === "--no-record") a.record = false;
    else if (t === "--no-encode") a.encode = false;
    else if (t === "--keep-mov") a.keepMov = true;
    else if (t === "--pace") a.pace = parseInt(argv[++i], 10);
    else if (t === "--width") a.width = parseInt(argv[++i], 10);
    else throw new Error(`Unknown argument: ${t}`);
  }
  if (a.flows.length === 0) a.flows = ["getting-started", "import-csv", "manual-txn"];
  return a;
}

const args = parseArgs(process.argv.slice(2));
const PACE = args.pace;

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const log = (...m) => console.log("[record]", ...m);

// Timed step captions (BDD-style), burned into the video at encode time.
// `recStart` is when screencapture was launched; each caption records its
// wall-clock offset from that. The capture's own clock starts later (0.3 s on
// an idle machine, ~2.5 s under load), which `measureStartLag` reads back from
// the file so the captions can be shifted onto the frames they describe.
let captions = [];
let recStart = 0;
const DEFAULT_START_LAG = 0.3;
function caption(text) {
  log(`  • ${text}`);
  if (args.record && recStart) captions.push({ t: (Date.now() - recStart) / 1000, text });
}

function which(bin) {
  const r = spawnSync("which", [bin], { encoding: "utf8" });
  return r.status === 0 ? r.stdout.trim().split(/\r?\n/)[0] : null;
}

function resolveAppBinary() {
  for (const c of APP_BINARY_CANDIDATES) if (fs.existsSync(c)) return c;
  throw new Error(
    `No app binary found. Build it first:\n  ./scripts/run_debug_mcp.sh\n` +
      `Looked in:\n  ${APP_BINARY_CANDIDATES.join("\n  ")}`,
  );
}

function resolveTauriWd() {
  return which("tauri-wd") || path.join(HOME, ".cargo", "bin", "tauri-wd");
}

// ---------------------------------------------------------------------------
// osascript helpers (native dialogs, window geometry)
// ---------------------------------------------------------------------------
function osa(script) {
  const r = spawnSync("osascript", ["-e", script], { encoding: "utf8" });
  if (r.status !== 0) {
    const err = (r.stderr || "").trim();
    const e = new Error(`osascript failed: ${err}`);
    e.osascript = err;
    throw e;
  }
  return (r.stdout || "").trim();
}

/** TCC probe — fails clearly if Accessibility/Automation is not granted. */
function probeAutomationTcc() {
  try {
    osa('tell application "System Events" to return name of first process');
    return true;
  } catch (e) {
    const msg = e.osascript || "";
    if (/-1719|-25211|not allowed|assistive|accessibility/i.test(msg)) {
      throw new Error(
        "osascript can't control System Events — grant your terminal " +
          "Accessibility + Automation in System Settings → Privacy & Security, then re-run.\n" +
          `  (${msg})`,
      );
    }
    throw e;
  }
}

/** Find the running app's System Events process name (emuacc-covid). */
function findAppProcessName() {
  const names = osa('tell application "System Events" to get name of every process whose background only is false');
  const match = names.split(",").map((s) => s.trim()).find((n) => /emuacc/i.test(n));
  if (!match) throw new Error(`Could not find the app process. Visible processes: ${names}`);
  return match;
}

// Capture by CoreGraphics window id (`screencapture -l`) rather than a screen
// region: it's immune to multi-display coordinate offsets and window occlusion
// (System Events AX coords and `screencapture -R` don't share an origin across
// displays). Getting the id needs CGWindowList, so a tiny Swift helper.
const GETWIN_SWIFT = `import CoreGraphics
import Foundation
let args = CommandLine.arguments
let owner = args.count > 1 ? args[1].lowercased() : ""
let title = args.count > 2 ? args[2].lowercased() : ""
let infos = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], CGWindowID(0)) as? [[String: Any]] ?? []
var best = -1, bestArea = -1.0
for w in infos {
  let o = ((w[kCGWindowOwnerName as String] as? String) ?? "").lowercased()
  let n = ((w[kCGWindowName as String] as? String) ?? "").lowercased()
  let layer = (w[kCGWindowLayer as String] as? Int) ?? -1
  guard o.contains(owner), layer == 0, title.isEmpty || n.contains(title),
        let b = w[kCGWindowBounds as String] as? [String: Any],
        let wd = b["Width"] as? Double, let ht = b["Height"] as? Double,
        let num = w[kCGWindowNumber as String] as? Int else { continue }
  if wd*ht > bestArea { bestArea = wd*ht; best = num }
}
if best >= 0 { print(best) }`;

let swiftPath;
/** The largest on-screen window whose owner contains `owner` and, when given,
 *  whose title contains `title` (titles are only readable with the Screen
 *  Recording grant, which the capture needs anyway). */
function getWindowId(owner = "emuacc", title = "") {
  if (!swiftPath) {
    swiftPath = path.join(os.tmpdir(), `emuacc-getwin-${process.pid}.swift`);
    fs.writeFileSync(swiftPath, GETWIN_SWIFT);
  }
  const r = spawnSync("swift", [swiftPath, owner, title], { encoding: "utf8" });
  const id = (r.stdout || "").trim().split(/\r?\n/)[0];
  if (!id) throw new Error(`Could not find the ${owner} window id (Screen Recording permission for the terminal?)`);
  return id;
}

/** `{x, y, w, h}` of a process's front window, from System Events. */
function windowFrame(proc) {
  const raw = osa(`tell application "System Events" to tell process ${JSON.stringify(proc)} to get {position, size} of window 1`);
  const [x, y, w, h] = raw.split(",").map((v) => parseInt(v.trim(), 10));
  return { x, y, w, h };
}

// Make the window a bit larger than the app's 900x600 default so the demos
// are more legible (only affects this recording, not the app's real default).
const DEMO_WIN = { w: 1280, h: 860 };
function resizeWindow(proc) {
  try {
    osa(`tell application "System Events" to tell process ${JSON.stringify(proc)} to set size of window 1 to {${DEMO_WIN.w}, ${DEMO_WIN.h}}`);
  } catch {
    /* not resizable / not ready — non-fatal */
  }
}

// NOTE: Tauri's NSOpenPanel is not enumerable via System Events (it reports
// no sheet/window), so we can't poll for it — we wait a fixed beat for it to
// animate in, then drive it with blind keystrokes. This requires the
// controlling terminal to have macOS Accessibility permission (synthetic
// keystrokes); without it the keystrokes are silently dropped and the app
// stays blocked on open(). See docs/demos/RECORDING.md.

/** "Go to folder" (Cmd+Shift+G) → type a path → Return. Returns the key-code 36
 *  helper count; callers add the final confirming Return(s). */
function goToFolder(proc, p) {
  return (
    `tell application "System Events"\n` +
    `set frontmost of process ${JSON.stringify(proc)} to true\n` +
    `delay 0.6\n` +
    `keystroke "g" using {command down, shift down}\n` +
    `delay 1.0\n` +
    `keystroke ${JSON.stringify(p)}\n` +
    `delay 0.8\n` +
    `key code 36\n` // navigate to the path
  );
}

/** Choose a folder in a directory NSOpenPanel: go into it, then confirm. */
async function pickFolderInDialog(proc, folderPath) {
  await sleep(1500); // let the panel animate in
  // The long beat before Choose matters: while screencapture is running the
  // panel navigates slower, and an early Choose picks the folder it was
  // showing before — its remembered last directory, not the one typed.
  osa(goToFolder(proc, folderPath) + `delay 1.8\nkey code 36\nend tell`); // Return → Choose
  await sleep(600);
}

/** Select a file in a file NSOpenPanel: go to its folder, type-select the
 *  filename, then open. ("Go to folder" only navigates dirs, so a full file
 *  path would land in the parent without selecting the file.) */
async function pickFileInDialog(proc, filePath) {
  await sleep(1500);
  const dir = path.dirname(filePath);
  const base = path.basename(filePath);
  osa(
    goToFolder(proc, dir + "/") +
      `delay 1.8\n` +
      `keystroke ${JSON.stringify(base)}\n` + // type-select highlights the file
      `delay 0.8\n` +
      `key code 36\n` + // Open
      `end tell`,
  );
  await sleep(600);
}

// ---------------------------------------------------------------------------
// WebDriver helpers
// ---------------------------------------------------------------------------
async function newSession(appBinary) {
  // NOTE: tauri-wd ignores tauri:options.env — the app's env is inherited from
  // the tauri-wd process instead (set in startTauriWd).
  const caps = new Capabilities();
  caps.set("tauri:options", { application: appBinary, args: [] });
  caps.setBrowserName("wry");
  return new Builder().withCapabilities(caps).usingServer(WD_URL).build();
}

/** The WebView window handle appears ~1-2s after the session is created. */
async function waitForWindow(driver, timeout = 20000) {
  const deadline = Date.now() + timeout;
  let handles = [];
  while (Date.now() < deadline) {
    try { handles = await driver.getAllWindowHandles(); } catch { handles = []; }
    if (handles.length) break;
    await sleep(200);
  }
  if (!handles.length) throw new Error("App window did not appear");
  await driver.switchTo().window(handles[0]);
}

const css = (sel) => By.css(sel);
async function waitFor(driver, sel, timeout = 20000) {
  return driver.wait(until.elementLocated(css(sel)), timeout);
}
async function click(driver, sel, timeout = 20000) {
  const el = await driver.wait(until.elementLocated(css(sel)), timeout);
  await driver.wait(until.elementIsEnabled(el), timeout);
  await el.click();
  await sleep(PACE);
}
// Type one character at a time with a small randomized delay so it reads like a
// real person typing — applied to every field (short inputs and the rhai alike).
async function type(driver, sel, text) {
  const el = await waitFor(driver, sel);
  await el.clear();
  await sleep(180);
  for (const ch of text) {
    await el.sendKeys(ch);
    await sleep(45 + Math.random() * 85); // ~45-130ms/key, with natural jitter
  }
  await sleep(PACE);
}
async function waitGone(driver, sel, timeout = 20000) {
  await driver.wait(async () => (await driver.findElements(css(sel))).length === 0, timeout);
}
/** The app opens on the Reports view; account actions live on the Accounts view. */
async function gotoAccountsView(driver) {
  const nav = await driver.findElements(css('button[data-view="accounts"]'));
  if (nav.length) { await nav[0].click(); await sleep(PACE); }
  await waitFor(driver, "#addAccountBtn");
}

// ---------------------------------------------------------------------------
// config.json safety (flow 1 only)
// ---------------------------------------------------------------------------
function moveConfigAside() {
  if (fs.existsSync(CONFIG)) {
    fs.renameSync(CONFIG, CONFIG_BAK);
    log(`moved real config.json aside → ${path.basename(CONFIG_BAK)}`);
  }
}
function restoreConfig() {
  if (fs.existsSync(CONFIG_BAK)) {
    fs.renameSync(CONFIG_BAK, CONFIG); // overwrites any config the demo wrote
    log("restored real config.json");
  }
}
// Point the app at a throwaway vault (current_root) so it skips the picker and
// never reads — or even names — the user's real vault. Always paired with
// moveConfigAside; restoreConfig() puts the real config back afterwards.
function writeTempConfig(vaultDir) {
  fs.mkdirSync(APP_DATA, { recursive: true });
  fs.writeFileSync(
    CONFIG,
    JSON.stringify({ current_root: vaultDir, known_roots: [vaultDir], update_prices_on_startup: false }, null, 2),
  );
}
// Restore on any exit so the user's vault config is never lost, and don't leak
// tauri-wd processes. SIGINT/SIGTERM must exit explicitly — registering a handler
// otherwise suppresses the default termination (the process would hang).
function cleanupOnExit() {
  try { restoreConfig(); } catch { /* best effort */ }
  try { restoreTextEditFontSize(); } catch { /* best effort */ }
  for (const p of childTauriWds) { try { p.kill("SIGTERM"); } catch { /* best effort */ } }
  try { if (swiftPath) fs.rmSync(swiftPath, { force: true }); } catch { /* best effort */ }
}
process.on("exit", cleanupOnExit);
for (const sig of ["SIGINT", "SIGTERM"]) {
  process.on(sig, () => { cleanupOnExit(); process.exit(130); });
}

// ---------------------------------------------------------------------------
// Recording + encoding
// ---------------------------------------------------------------------------
function startRecording(movPath, windowId) {
  // Capture just this window (-l), shadow excluded (-o), clicks shown (-k).
  const proc = spawn("screencapture", ["-v", "-k", "-o", "-l" + windowId, movPath], { stdio: ["ignore", "inherit", "inherit"] });
  return proc;
}
function stopRecording(proc) {
  return new Promise((resolve) => {
    proc.on("exit", resolve);
    proc.kill("SIGINT"); // SIGINT finalizes the .mov; SIGKILL would corrupt it
  });
}

function ffprobeDuration(file) {
  const r = spawnSync("ffprobe", ["-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", file], { encoding: "utf8" });
  return parseFloat((r.stdout || "").trim()) || 60;
}
function ffprobeLastFramePts(file) {
  const r = spawnSync("ffprobe", ["-v", "error", "-select_streams", "v:0", "-show_entries", "frame=pts_time", "-of", "csv=p=0", file], { encoding: "utf8" });
  const lines = (r.stdout || "").trim().split(/\r?\n/).filter(Boolean);
  return parseFloat(lines[lines.length - 1]);
}

/** Paint a 2 px dot in the window corner and remove it again: two frames at a
 *  known wall-clock moment, the last ones in the capture (screencapture only
 *  writes a frame when the screen changes). Returns the removal's offset from
 *  `recStart`, in seconds. */
async function syncMark(driver) {
  await driver.executeScript(
    "const d = document.createElement('div'); d.id = '__recmark';" +
      "d.style.cssText = 'position:fixed;left:0;bottom:0;width:2px;height:2px;background:#fff;z-index:2147483647;pointer-events:none';" +
      "document.body.appendChild(d);",
  );
  await sleep(150);
  const removedAt = (Date.now() - recStart) / 1000;
  await driver.executeScript("document.getElementById('__recmark')?.remove();");
  await sleep(400); // let screencapture write the frame before it is stopped
  return removedAt;
}

/** How long after launch the capture's clock started: the sync mark's wall
 *  offset minus the timestamp of the last frame, which is that mark. */
function measureStartLag(movPath, markOffsetSec) {
  const lag = markOffsetSec - ffprobeLastFramePts(movPath);
  return Number.isFinite(lag) && lag >= 0 && lag < 10 ? lag : DEFAULT_START_LAG;
}
function srtTime(s) {
  const ms = Math.max(0, Math.round(s * 1000));
  const p = (n, w = 2) => String(n).padStart(w, "0");
  return `${p(Math.floor(ms / 3600000))}:${p(Math.floor((ms % 3600000) / 60000))}:${p(Math.floor((ms % 60000) / 1000))},${p(ms % 1000, 3)}`;
}
function writeSrt(srtPath, caps, durationSec) {
  let out = "";
  caps.forEach((c, i) => {
    const start = c.t;
    const end = i + 1 < caps.length ? caps[i + 1].t : durationSec;
    out += `${i + 1}\n${srtTime(start)} --> ${srtTime(Math.max(start + 0.7, end))}\n${c.text}\n\n`;
  });
  fs.writeFileSync(srtPath, out);
}

/** Seconds the final state of a segment is held (added at encode time, see
 *  `tpad` below). */
const HOLD_SEC = 3;
const CAPTION_STYLE = "FontName=Helvetica,FontSize=16,PrimaryColour=&H00FFFFFF,OutlineColour=&H00000000,BorderStyle=1,Outline=2,Shadow=1,MarginV=26,Alignment=2";

function ffprobeDims(file) {
  const r = spawnSync("ffprobe", ["-v", "error", "-select_streams", "v:0", "-show_entries", "stream=width,height", "-of", "csv=p=0", file], { encoding: "utf8" });
  const [w, h] = (r.stdout || "").trim().split(",").map(Number);
  return { w, h };
}

/** Encode one clip's segments, in order, to mp4 + webm + poster. A segment is
 *  a window capture with its captions (on the capture's clock) and its hold;
 *  one segment is the usual clip, the getting-started take adds the TextEdit
 *  shot as a second one, joined with a hard cut. */
async function encodeAll(segments, flowName) {
  const mp4 = path.join(OUT_DIR, `${flowName}.mp4`);
  const webm = path.join(OUT_DIR, `${flowName}.webm`);
  const poster = path.join(OUT_DIR, `${flowName}-poster.png`);

  // Every segment is fitted to the first one's frame (the windows are sized
  // alike; this guards the odd pixel) so concat accepts them. screencapture
  // writes variable-frame-rate video — no new frame while the screen is
  // static — so a file ends at its last screen change and the subtitles
  // filter could only change its text on a new frame (a caption would stick
  // until the next change). fps=30 supplies the frames the captions need, and
  // tpad clones the final frame so the payoff state holds instead of ending
  // the moment it appeared. Filtergraph quoting: the srt path and force_style
  // are single-quoted so the commas inside force_style aren't read as filter
  // separators (temp paths are alnum, so safe to quote).
  const first = ffprobeDims(segments[0].movPath);
  const outH = Math.round((first.h * args.width) / first.w / 2) * 2;
  const chains = segments.map((seg, i) => {
    let f = `[${i}:v]fps=30,tpad=stop_mode=clone:stop_duration=${seg.hold},` +
      `scale=${args.width}:${outH}:force_original_aspect_ratio=decrease:flags=lanczos,` +
      `pad=${args.width}:${outH}:(ow-iw)/2:(oh-ih)/2`;
    if (seg.captions.length) {
      const srtPath = path.join(path.dirname(seg.movPath), `${flowName}-${i}.srt`);
      writeSrt(srtPath, seg.captions, ffprobeDuration(seg.movPath) + seg.hold); // the last caption spans the held frame too
      f += `,subtitles='${srtPath}':force_style='${CAPTION_STYLE}'`;
    }
    return `${f}[v${i}]`;
  });
  const labels = segments.map((_, i) => `[v${i}]`).join("");
  const graph = [...chains, `${labels}concat=n=${segments.length}:v=1:a=0[v]`].join(";");
  const inputs = segments.flatMap((seg) => ["-i", seg.movPath]);

  log(`encoding ${flowName} (${segments.length} segment${segments.length === 1 ? "" : "s"}) → ${args.width}px mp4 + webm`);
  const run = (a) => {
    const r = spawnSync("ffmpeg", a, { stdio: ["ignore", "ignore", "inherit"] });
    if (r.status !== 0) throw new Error(`ffmpeg failed: ${a.join(" ")}`);
  };
  fs.mkdirSync(OUT_DIR, { recursive: true });
  run(["-y", ...inputs, "-filter_complex", graph, "-map", "[v]", "-an", "-pix_fmt", "yuv420p",
    "-c:v", "libx264", "-profile:v", "high", "-preset", "slow", "-crf", "24",
    "-movflags", "+faststart", mp4]);
  run(["-y", ...inputs, "-filter_complex", graph, "-map", "[v]", "-an",
    "-c:v", "libvpx-vp9", "-b:v", "0", "-crf", "33", "-row-mt", "1", webm]);
  run(["-y", "-i", mp4, "-update", "1", "-frames:v", "1", "-q:v", "3", poster]);
  log(`wrote ${path.relative(repoRoot, mp4)}, .webm, -poster.png`);
}

// ---------------------------------------------------------------------------
// Pre-seeding + account selection
// ---------------------------------------------------------------------------
// Flows 2 & 3 need an existing account to act on. Rather than driving the
// "+ Add Account" UI (which pops a native CSV dialog requiring Accessibility),
// we pre-seed a tiny manual.transactions ledger on disk so the account shows
// up with real history — fully in-WebView. "personal" is the account set
// (top-level folder); the account renders as `assets:cash`.
const SEED_ACCOUNT = "assets:cash";
function seedDemoAccount(ctx) {
  const dir = path.join(ctx.sources, "personal", "cash");
  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(
    path.join(dir, "manual.transactions"),
    [
      "2025-01-03 * Opening balance",
      "    assets:cash        1500.00 USD",
      "    equity:opening    -1500.00 USD",
      "",
      "2025-01-15 * Acme Payroll",
      "    assets:cash        2500.00 USD",
      "    income:salary     -2500.00 USD",
      "",
      "2025-01-16 * Woolworths",
      "    assets:cash         -85.40 USD",
      "    expenses:groceries    85.40 USD",
      "",
    ].join("\n"),
  );
}

// Clean import folder + CSV staged fresh.
function stageDemoCsv() {
  try { fs.rmSync(DEMO_IMPORT_DIR, { recursive: true, force: true }); } catch { /* ignore */ }
  fs.mkdirSync(DEMO_IMPORT_DIR, { recursive: true });
  fs.writeFileSync(DEMO_CSV, DEMO_CSV_BODY);
}

// The standalone import demo also needs an account to import into; its
// commodity stays USD.
function seedImportFlow(ctx) {
  seedDemoAccount(ctx);
  stageDemoCsv();
}

// A clean, hand-written transform typed into the editor during the import demo —
// shows users how to map columns and sets the commodity to USD.
const USD_TRANSFORM = [
  "#{",
  '  date: row["Date"],',
  '  payee: row["Description"],',
  '  amount: row["Amount"],',
  '  commodity: "USD",',
  "}",
].join("\n");

async function selectAccount(driver, account) {
  await click(driver, `[data-testid="account-item"][data-account="${account}"]`);
}

// ---------------------------------------------------------------------------
// On-camera steps, shared between the standalone flows and the combined
// getting-started take.
// ---------------------------------------------------------------------------
/** First launch: choose the data folder in the vault picker and the vault's
 *  base currency, then add an account. Ends with the app's own "import a file
 *  into the new account" dialog open — it offers one as soon as the account
 *  exists. */
async function chooseVaultAndAddAccountSteps(driver, proc, ctx) {
  caption("Choose a data folder for your accounts");
  await click(driver, "#vaultPickerOpen");
  caption("Pick a folder in the file dialog");
  await pickFolderInDialog(proc, ctx.vaultDir);
  // A new vault asks for its base currency before anything else.
  await waitFor(driver, '[data-testid="base-currency-setup"]', 30000);
  // The folder dialog is driven blind, so prove it landed on the demo vault
  // before anything is typed into it: opening a root runs the pipeline, which
  // writes its cache and metadata into that root's generated/ at once.
  if (fs.readdirSync(ctx.generated).length === 0) {
    throw new Error(
      `The vault picker did not land on ${ctx.vaultDir} (its generated/ is still empty) — ` +
        'the "Go to folder" keystrokes were lost or Choose fired before the panel had navigated.',
    );
  }
  caption("Choose the base currency your reports are in");
  await type(driver, '[data-testid="base-currency-input"]', "USD");
  await click(driver, '[data-testid="base-currency-continue"]');
  await waitFor(driver, '[data-testid="app-ready"]', 30000);
  await gotoAccountsView(driver);
  caption("Add an account");
  await click(driver, '[data-testid="add-account-btn"]');
  await waitFor(driver, '[data-testid="add-account-modal"]');
  // An empty vault has no account set yet, so the form asks for the entity
  // (the person or company the accounts belong to) before the account name.
  caption("Name the entity the accounts belong to, for example personal");
  await type(driver, "#newAccountEntityName", "personal");
  caption("Name the account, for example savings");
  await type(driver, "#newAccountName", "savings");
  await click(driver, "#addAccountSubmit");
}

/** Import the staged CSV into the selected account through the column-mapping
 *  modal. With `dialogOpen` the native file dialog is already up (the app opens
 *  one right after an account is added); otherwise the Import button opens it. */
async function importCsvSteps(driver, proc, { dialogOpen }) {
  if (dialogOpen) {
    caption("The app offers to import a bank export straight away");
  } else {
    caption("Import a CSV into the selected account");
    await click(driver, "#importFile");
  }
  caption("Choose the CSV file");
  await pickFileInDialog(proc, DEMO_CSV);
  // The column-mapping modal appears for a new CSV format.
  await waitFor(driver, "#transformScript", 15000);
  caption("Edit the Rhai transform to map the columns");
  await type(driver, "#transformScript", USD_TRANSFORM);
  caption("Save and import");
  await click(driver, "#transformSave");
  // The status line is the deterministic success signal ("Imported CSV (N…)").
  await driver.wait(
    until.elementTextContains(driver.findElement(css('[data-testid="parse-status"]')), "Imported CSV"),
    30000,
  );
  caption("Transactions imported");
  await sleep(PACE);
}

/** Add a transaction by hand. Done when a row shows its payee — the list may
 *  already hold imported rows, so a bare row count is no signal. */
async function manualTxnSteps(driver) {
  caption("Add a transaction by hand");
  await click(driver, "#addNew");
  await waitFor(driver, '[data-testid="manual-txn-modal"]');
  caption("Enter the payee, notes and amount");
  await type(driver, "#manualPayee", "Coffee Hut");
  await type(driver, "#manualNarration", "Flat white");
  await type(driver, "#manualAmount", "-3.50");
  caption("Pick the category account");
  await type(driver, "#manualContraAccount-0", "expenses:coffee");
  caption("Save");
  await click(driver, "#manualSave");
  await driver.wait(
    until.elementLocated(By.xpath('//*[@data-testid="txn-row"][contains(., "Coffee Hut")]')),
    30000,
  );
  caption("Transaction added");
}

// ---------------------------------------------------------------------------
// Closing shot: the account's generated ledger, as plain text in TextEdit.
// The app has no text viewer and a window-id capture shows one window, so
// this is a second capture (TextEdit's own window id) that encodeAll joins to
// the take with a hard cut.
// ---------------------------------------------------------------------------
const LEDGER_CAPTION = "Everything you just did is this text file";
const FINALE_HOLD_SEC = 5;
// TextEdit's plain-text default (11 pt) is too small once the window is scaled
// to the video width. The size is a preference read at launch; it is set for
// the shot and put back afterwards (on crash/Ctrl-C too, see cleanupOnExit).
const TEXTEDIT_FONT_SIZE = 16;
const TEXTEDIT_DOMAIN = "com.apple.TextEdit";
const TEXTEDIT_SIZE_KEY = "NSFixedPitchFontSize";
let textEditFontSizeBackup; // undefined = not touched; null = the key was absent

function setTextEditFontSize() {
  const r = spawnSync("defaults", ["read", TEXTEDIT_DOMAIN, TEXTEDIT_SIZE_KEY], { encoding: "utf8" });
  textEditFontSizeBackup = r.status === 0 ? r.stdout.trim() : null;
  spawnSync("defaults", ["write", TEXTEDIT_DOMAIN, TEXTEDIT_SIZE_KEY, "-int", String(TEXTEDIT_FONT_SIZE)]);
}
function restoreTextEditFontSize() {
  if (textEditFontSizeBackup === undefined) return;
  if (textEditFontSizeBackup === null) spawnSync("defaults", ["delete", TEXTEDIT_DOMAIN, TEXTEDIT_SIZE_KEY]);
  else spawnSync("defaults", ["write", TEXTEDIT_DOMAIN, TEXTEDIT_SIZE_KEY, "-int", textEditFontSizeBackup]);
  textEditFontSizeBackup = undefined;
}

/** The ledger the pipeline wrote under generated/ for the take's one account. */
function findLedger(dir) {
  for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      const hit = findLedger(p);
      if (hit) return hit;
    } else if (entry.name === "ledger.transactions") {
      return p;
    }
  }
  return null;
}

/** Open `file` in TextEdit, placed and sized like `frame` (the app window),
 *  the file name in its title. Driven through `open -a` and System Events, so
 *  it needs no permission beyond the recording's own. Returns the CG window
 *  id to capture and `close()`, which closes the document and quits TextEdit
 *  when it was not running before. */
async function openInTextEdit(file, frame) {
  const wasRunning = spawnSync("pgrep", ["-x", "TextEdit"]).status === 0;
  if (!wasRunning) setTextEditFontSize(); // a running TextEdit keeps its font (still legible)
  spawnSync("open", ["-a", "TextEdit", file]);
  await sleep(2000);
  osa(
    `tell application "System Events" to tell process "TextEdit"\n` +
      `set frontmost to true\n` +
      `set position of window 1 to {${frame.x}, ${frame.y}}\n` +
      `set size of window 1 to {${frame.w}, ${frame.h}}\n` +
      `end tell`,
  );
  await sleep(600);
  const windowId = getWindowId("TextEdit", path.basename(file));
  const key = (k) => osa(`tell application "System Events" to tell process "TextEdit" to keystroke "${k}" using command down`);
  return {
    windowId,
    async close() {
      try { key("w"); } catch { /* best effort */ }
      await sleep(500);
      if (!wasRunning) { try { key("q"); } catch { /* best effort */ } }
      restoreTextEditFontSize();
    },
  };
}

// ---------------------------------------------------------------------------
// Flows. `needsAccessibility` flows drive native OS dialogs via osascript.
// ---------------------------------------------------------------------------
const FLOWS = {
  // Flow 1 — the README's getting-started video: one continuous take of the
  // three flows in the order a new user meets them, in one throwaway vault
  // (the folder picked in step 1 is the vault steps 2 and 3 work in).
  "getting-started": {
    moveConfigAside: true,
    tempRoot: false, // we WANT the first-launch vault picker here
    seed: stageDemoCsv, // no account seed: the take adds the account itself
    needsAccessibility: true,
    initialReady: '[data-testid="vault-picker"]',
    prepare: null,
    async steps(driver, proc, ctx) {
      await chooseVaultAndAddAccountSteps(driver, proc, ctx);
      await importCsvSteps(driver, proc, { dialogOpen: true });
      await manualTxnSteps(driver);
    },
    // Closing shot: the ledger the pipeline wrote for the account, as text.
    finale(ctx) {
      const file = findLedger(ctx.generated);
      if (!file) throw new Error(`No ledger.transactions under ${ctx.generated} — the pipeline wrote nothing for the account`);
      return { file, caption: LEDGER_CAPTION };
    },
  },

  // Flow 2: import a CSV into an account (transactions appear).
  "import-csv": {
    moveConfigAside: true,
    tempRoot: true, // current_root → temp vault: no picker, real vault never read or named
    seed: seedImportFlow,
    needsAccessibility: true,
    initialReady: '[data-testid="app-ready"]',
    async prepare(driver) {
      await gotoAccountsView(driver);
      await selectAccount(driver, SEED_ACCOUNT);
    },
    async steps(driver, proc) {
      await importCsvSteps(driver, proc, { dialogOpen: false });
    },
  },

  // Flow 3: add a manual transaction (fully in-WebView, no native dialog).
  "manual-txn": {
    moveConfigAside: true,
    tempRoot: true, // current_root → temp vault: no picker, real vault never read or named
    seed: seedDemoAccount,
    needsAccessibility: false,
    initialReady: '[data-testid="app-ready"]',
    async prepare(driver) {
      await gotoAccountsView(driver);
      await selectAccount(driver, SEED_ACCOUNT);
    },
    async steps(driver) {
      await manualTxnSteps(driver);
    },
  },
};

// ---------------------------------------------------------------------------
// Orchestration
// ---------------------------------------------------------------------------
function makeCtx() {
  const base = fs.mkdtempSync("/tmp/emuacc-demo-"); // /tmp so the demo path is obviously throwaway
  const ctx = {
    base,
    vaultDir: path.join(base, "vault"),
    sources: path.join(base, "vault", "sources"),
    generated: path.join(base, "vault", "generated"),
  };
  fs.mkdirSync(ctx.sources, { recursive: true });
  fs.mkdirSync(ctx.generated, { recursive: true });
  return ctx;
}

// tauri-wd is restarted per flow so each run starts from a clean app process.
const childTauriWds = [];
function startTauriWd() {
  spawnSync("pkill", ["-f", "tauri-wd"]); // take exclusive ownership of the port
  const bin = resolveTauriWd();
  if (!fs.existsSync(bin)) throw new Error(`tauri-wd not found at ${bin}. Install: cargo install tauri-webdriver-automation`);
  // The app inherits this env. An empty EMUACC_INSTANCE keeps the release
  // window title ("EmuAcc COVID") instead of the worktree suffix a dev build adds.
  const proc = spawn(bin, ["--port", String(WD_PORT)], {
    stdio: ["ignore", "ignore", "inherit"],
    env: { ...process.env, EMUACC_INSTANCE: "" },
  });
  childTauriWds.push(proc);
  return proc;
}
function stopTauriWd(proc) {
  try { proc.kill("SIGTERM"); } catch { /* ignore */ }
}

function preflight() {
  if (process.platform !== "darwin") throw new Error("record_demos.mjs is macOS-only (uses screencapture/osascript).");
  resolveAppBinary();
  for (const bin of ["ffmpeg", "ffprobe", "screencapture", "osascript"]) {
    if (!which(bin)) throw new Error(`Required tool not on PATH: ${bin}`);
  }
  // CRITICAL: refuse to run if another emuacc-covid app is already live (e.g.
  // `tauri dev`). tauri-wd would attach to and drive THAT app — pointed at your
  // REAL vault — instead of the isolated temp-vault binary we launch, recording
  // real data. Stop the other app first.
  // -x matches the app's exact process name only — NOT cargo/rustc build
  // processes that merely reference the binary path in their args.
  const running = spawnSync("pgrep", ["-x", "emuacc-covid"], { encoding: "utf8" }).stdout.trim();
  if (running) {
    throw new Error(
      "A emuacc-covid app is already running (e.g. `tauri dev`) — refusing to record.\n" +
        "  tauri-wd would drive the running app on your REAL vault, not the isolated demo vault.\n" +
        "  Stop it, then re-run. Running pids: " + running.replace(/\s+/g, " "),
    );
  }
  probeAutomationTcc();
  fs.mkdirSync(OUT_DIR, { recursive: true });
}

/** On failure, log what the app was showing so a broken step can be read off
 *  the log instead of re-run with eyes on it. */
async function dumpUiState(driver) {
  const text = async (sel) => {
    const els = await driver.findElements(css(sel));
    return els.length ? (await els[0].getText()).trim() || "(empty)" : "(none)";
  };
  log(`  ✖ status line: ${await text('[data-testid="parse-status"]')}`);
  log(`  ✖ modal error: ${await text(".modal__error, .vaultPicker__error")}`);
  const screens = await driver.findElements(css('[data-testid="vault-picker"], [data-testid="base-currency-setup"], .modalOverlay[data-testid]'));
  const ids = [];
  for (const el of screens) ids.push(await el.getAttribute("data-testid"));
  log(`  ✖ screens/modals: ${ids.join(", ") || "(none)"}`);
}

async function recordFlow(flowName) {
  const cfg = FLOWS[flowName];
  if (!cfg) throw new Error(`Unknown flow: ${flowName} (have: ${Object.keys(FLOWS).join(", ")})`);
  const appBinary = resolveAppBinary();
  const ctx = makeCtx();
  const movPath = path.join(ctx.base, `${flowName}.mov`);
  let driver, recProc, tauriWd;
  log(`▶ flow: ${flowName}`);
  if (cfg.needsAccessibility) log("  (drives native OS dialogs — needs macOS Accessibility permission)");

  if (cfg.moveConfigAside) moveConfigAside();
  if (cfg.tempRoot) writeTempConfig(ctx.vaultDir);
  if (cfg.seed) cfg.seed(ctx); // write demo source files BEFORE the app launches
  try {
    tauriWd = startTauriWd();
    await sleep(1500); // let tauri-wd bind
    driver = await newSession(appBinary);
    await waitForWindow(driver);
    await waitFor(driver, cfg.initialReady, 30000);

    const proc = findAppProcessName();
    resizeWindow(proc);
    await sleep(600);

    if (cfg.prepare) {
      log("  preparing view (off-camera)…");
      await cfg.prepare(driver, proc, ctx);
    }

    captions = [];
    if (args.record) {
      recProc = startRecording(movPath, getWindowId());
      recStart = Date.now();
      await sleep(1100); // capture the opening frame
    }

    log("  running on-camera steps…");
    await cfg.steps(driver, proc, ctx);
    // Let the app settle on the payoff state (its heavy refresh lands a beat
    // after the row appears); the on-screen hold itself is added when encoding.
    await sleep(2200);

    const segments = []; // { movPath, captions (on the capture's clock), hold }
    if (recProc) {
      const markAt = await syncMark(driver);
      await stopRecording(recProc);
      const startLag = measureStartLag(movPath, markAt);
      log(`  captured ${ffprobeDuration(movPath).toFixed(1)} s; the capture began ${startLag.toFixed(2)} s after launch`);
      segments.push({ movPath, captions: captions.map((c) => ({ ...c, t: Math.max(0, c.t - startLag) })), hold: HOLD_SEC });
    }

    if (cfg.finale) {
      const frame = windowFrame(proc); // TextEdit goes exactly where the app window is
      const { file, caption: text } = cfg.finale(ctx);
      const textEdit = await openInTextEdit(file, frame);
      try {
        log(`  • ${text}`);
        if (args.record) {
          const finaleMov = path.join(ctx.base, `${flowName}-finale.mov`);
          const finaleRec = startRecording(finaleMov, textEdit.windowId);
          await sleep(2500); // the window is static; tpad supplies the hold
          await stopRecording(finaleRec);
          segments.push({ movPath: finaleMov, captions: [{ t: 0, text }], hold: FINALE_HOLD_SEC });
        } else {
          await sleep(1500);
        }
      } finally {
        await textEdit.close();
      }
    }
    await driver.quit();
    driver = null;

    if (args.record && args.encode) await encodeAll(segments, flowName);
    else if (args.record) log(`  raw recording(s): ${segments.map((seg) => seg.movPath).join(", ")}`);
    log(`✔ ${flowName} done`);
  } catch (err) {
    if (driver) await dumpUiState(driver).catch(() => { /* best effort */ });
    throw err;
  } finally {
    if (driver) { try { await driver.quit(); } catch { /* ignore */ } }
    if (tauriWd) stopTauriWd(tauriWd);
    if (cfg.moveConfigAside) restoreConfig();
    if (!args.keepMov) { try { fs.rmSync(ctx.base, { recursive: true, force: true }); } catch { /* ignore */ } }
    else log(`  kept temp dir: ${ctx.base}`);
  }
}

async function main() {
  preflight();
  log(`flows: ${args.flows.join(", ")} | record=${args.record} encode=${args.encode}`);
  for (const f of args.flows) await recordFlow(f);
  log("all done");
}

main().catch((err) => {
  console.error("[record] ERROR:", err.message);
  process.exitCode = 1;
});
