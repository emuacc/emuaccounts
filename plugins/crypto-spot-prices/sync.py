#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["yfinance>=0.2"]
# ///
"""Unified daily crypto spot-price sync — the single crypto price source.

The list of what to price lives in a plain-text **commodities file** (default
`.data/commodities.txt`), one block per commodity, e.g.:

    commodity NEAR
      binance NEARUSDT
      coingecko near

Sources are tried top-down (binance, coingecko, yfinance), each rejected if its
newest price is stale (delisting), so the next is tried. The file is the source
of truth: edit it by hand, or let the **discover** pass append commodities you
hold but haven't declared yet (it scans the generated ledgers, probes Binance
then CoinGecko, and only ever APPENDS — your edits are never touched). Held
commodities that resolve on no exchange are reported as warnings, not written.

BACKFILL + INCREMENTAL: the first time a commodity is seen it backfills full
history from `start_date`; afterwards only the last `refresh_days` are refreshed
(state in `.data/coverage.json`). The core pipeline never reads any of this — it
only consumes the `sources/_prices/*.txt` files this plugin writes.

Input (stdin JSON): {config, secrets, sources_dir, data_dir}
Output: JSON {files_written, backfilled, discovered, warnings}.
"""
import json
import os
import re
import sys
import time
import glob
import datetime
import urllib.request
import urllib.parse
from urllib.error import HTTPError, URLError

DAY_MS = 86_400_000
# The ledger commodity grammar, mirroring COMMODITY_RE in
# src-tauri/src/ledger_parser.rs (`^[\p{L}\p{N}_][\p{L}\p{N}_.\-:]*$`; Python's
# `\w` is exactly [\p{L}\p{N}_]): bare tickers (BTC, BRK.B, RAY-SOL, ATOM.S)
# and EXCHANGE:SYMBOL (xyz:WTIOIL). Anything the ledger accepts must be seen here.
COMMODITY = r"\w[\w.\-:]*"
POSTING_RE = re.compile(rf"\s+\S+\s+(-?[\d,]+\.?\d*)\s+({COMMODITY})")
TICKER_RE = re.compile(rf"^{COMMODITY}$")


# --- small helpers -----------------------------------------------------------
def _day(ms):
    return datetime.datetime.fromtimestamp(ms / 1000, datetime.timezone.utc).strftime("%Y-%m-%d")


def _to_ms(date_str):
    d = datetime.datetime.strptime(date_str[:10], "%Y-%m-%d")
    return int(d.replace(tzinfo=datetime.timezone.utc).timestamp() * 1000)


def _add_days(date_str, n):
    d = datetime.datetime.strptime(date_str[:10], "%Y-%m-%d").date()
    return (d + datetime.timedelta(days=n)).isoformat()


def _stale(prices, max_age_days):
    if not prices:
        return True
    try:
        latest = max(d for d, _ in prices)
        age = (datetime.date.today() - datetime.datetime.strptime(latest, "%Y-%m-%d").date()).days
        return age > max_age_days
    except Exception:
        return True


def _fmt(p):
    """Adaptive precision so sub-cent coins (SHIB) don't round to 0.00."""
    if p >= 1:
        return f"{p:.2f}"
    if p >= 0.01:
        return f"{p:.4f}"
    if p >= 0.0001:
        return f"{p:.6f}"
    return (f"{p:.10f}".rstrip("0") or "0")


# --- price-file names --------------------------------------------------------
# A commodity's prices live in `sources/_prices/<stem>.txt`. The stem
# percent-encodes the characters that are unsafe in a file name on some
# platform (':' is NTFS stream syntax on Windows and renders as '/' in Finder):
# xyz:WTIOIL -> xyz%3AWTIOIL. Everything else ('.', '-', '_', unicode letters)
# is kept, so existing file names are unchanged. The P lines inside always
# carry the TRUE commodity - the app keys on the line, not on the file name.
#
# This pair MUST stay identical to `price_file_stem` in
# src-tauri/src/ledger_parser.rs. Plugins are copied into vaults standalone,
# so there is no shared module: every price plugin carries the same copy.
_FILENAME_UNSAFE = ':/\\*?"<>|'


def price_file_stem(commodity):
    return "".join(f"%{ord(c):02X}" if c in _FILENAME_UNSAFE else c for c in commodity)


def commodity_from_price_stem(stem):
    """Inverse of price_file_stem, for deriving a commodity from a file name.
    Unambiguous because '%' is outside the commodity grammar."""
    return re.sub(r"%([0-9A-Fa-f]{2})", lambda m: chr(int(m.group(1), 16)), stem)


# --- commodities file (the source of truth) ----------------------------------
def parse_commodities(path):
    """Return {SYMBOL: [(source, value), ...]} preserving source order."""
    out = {}
    cur = None
    if not os.path.exists(path):
        return out
    for raw in open(path):
        line = raw.split("#", 1)[0].rstrip()
        if not line.strip():
            continue
        if line.startswith("commodity "):
            cur = line.split(None, 1)[1].strip()
            out.setdefault(cur, [])
        elif cur and raw[:1] in (" ", "\t"):
            parts = line.split()
            if len(parts) >= 2:
                out[cur].append((parts[0], parts[1]))
    return out


def append_commodity(path, sym, sources, today):
    """Append a discovered commodity block (raw text — never rewrites the file)."""
    block = [f"\ncommodity {sym}  # auto-discovered {today}"]
    for src, val in sources:
        block.append(f"  {src} {val}")
    with open(path, "a") as f:
        f.write("\n".join(block) + "\n")


# --- discover ----------------------------------------------------------------
def scan_held_commodities(generated_dir):
    """Distinct commodities appearing in the generated ledgers, with txn counts."""
    held = {}
    for f in glob.glob(os.path.join(generated_dir, "**", "ledger.transactions"), recursive=True):
        try:
            for line in open(f):
                m = POSTING_RE.match(line)
                if m:
                    held[m.group(2)] = held.get(m.group(2), 0) + 1
        except OSError:
            pass
    return held


# What a spot-market ticker on Binance / CoinGecko looks like: 2-8 ASCII
# alphanumerics. Scanning (POSTING_RE / TICKER_RE) must see every ledger-legal
# commodity, but only these are worth a network probe — venue-scoped ids
# (`xyz:WTIOIL`), staked/wrapped dotted names (`ATOM.S`), dashed LP-style names
# and unicode spam mints are never spot-listed, so they are left unpriced
# without a call (the pre-existing behaviour for spam).
PROBE_RE = re.compile(r"^[A-Za-z][A-Za-z0-9]{1,7}$")


def is_candidate(sym):
    """A ledger-legal commodity worth probing a spot market for."""
    return bool(TICKER_RE.match(sym)) and bool(PROBE_RE.match(sym)) and not sym.endswith("LP")


def binance_has(pair, max_age_days):
    """True only if the pair exists AND its newest candle is recent — so discover
    won't adopt a delisted pair (which exists but returns frozen data)."""
    url = f"https://api.binance.com/api/v3/klines?symbol={pair}&interval=1d&limit=1"
    try:
        with urllib.request.urlopen(url, timeout=15) as r:
            data = json.load(r)
    except Exception:
        return False
    if not data:
        return False
    try:
        latest = datetime.datetime.strptime(_day(data[0][0]), "%Y-%m-%d").date()
        return (datetime.date.today() - latest).days <= max_age_days
    except Exception:
        return False


def coingecko_search(sym, api_key):
    url = "https://api.coingecko.com/api/v3/search?query=" + urllib.parse.quote(sym)
    headers = {"Accept": "application/json"}
    if api_key:
        headers["x-cg-demo-api-key"] = api_key
    _coingecko_throttle(api_key)
    try:
        with urllib.request.urlopen(urllib.request.Request(url, headers=headers), timeout=15) as r:
            data = json.load(r)
    except Exception:
        return None
    for c in data.get("coins", []):
        if str(c.get("symbol", "")).upper() == sym.upper():
            return c.get("id")
    return None


def discover(commodities, path, sources_dir, generated_dir, quote, cg_key, skip_set, today, cap, max_age):
    """Find held-but-undeclared crypto, probe sources, append resolved ones.
    Returns (added_symbols, warnings). Mutates `commodities` and `skip_set`.

    Skips commodities that already have a price file (handled by this or another
    plugin — e.g. stocks/fiat). Binance is always probed; CoinGecko-search only
    when a key is present, to avoid hammering the free tier across many misses."""
    added, warnings = [], []
    if not os.path.isdir(generated_dir):
        return added, warnings
    prices_dir = os.path.join(sources_dir, "_prices")
    priced = (set(commodity_from_price_stem(f[:-4]) for f in os.listdir(prices_dir) if f.endswith(".txt"))
              if os.path.isdir(prices_dir) else set())
    held = scan_held_commodities(generated_dir)
    candidates = [s for s in held
                  if s not in commodities and s not in skip_set and s not in priced and is_candidate(s)]
    candidates.sort(key=lambda s: -held[s])  # busiest first
    for sym in candidates[:cap]:
        sources = []
        if binance_has(f"{sym}{quote}", max_age):
            sources.append(("binance", f"{sym}{quote}"))
        elif cg_key:
            cg_id = coingecko_search(sym, cg_key)
            if cg_id:
                sources.append(("coingecko", cg_id))
        if sources:
            append_commodity(path, sym, sources, today)
            commodities[sym] = sources
            added.append(f"{sym}<-{sources[0][0]}")
        else:
            skip_set.add(sym)
            tail = "" if cg_key else "; set a CoinGecko key to also search there, or add a source by hand"
            warnings.append(f"discover: {sym} held ({held[sym]} txns) — no Binance pair{tail}")
    return added, warnings


# --- price sources -----------------------------------------------------------
def fetch_binance(pair, start_date):
    cur = _to_ms(start_date)
    out = []
    while True:
        url = (f"https://api.binance.com/api/v3/klines?symbol={pair}"
               f"&interval=1d&startTime={cur}&limit=1000")
        try:
            with urllib.request.urlopen(url, timeout=30) as r:
                data = json.load(r)
        except HTTPError as e:
            raise RuntimeError("no pair" if e.code == 400 else f"HTTP {e.code}")
        except URLError as e:
            raise RuntimeError(f"net {e.reason}")
        if not data:
            break
        out += [(_day(k[0]), float(k[4])) for k in data]
        if len(data) < 1000:
            break
        cur = data[-1][0] + DAY_MS
    if not out:
        raise RuntimeError("empty")
    return out


# CoinGecko caps market_chart history at 365 days on the free and demo tiers and
# rejects a wider window with HTTP *401* + error_code 10012 — the same status a
# spent rate limit returns, so it reads as "rate-limited" unless the body is
# parsed. That matters because a first-time backfill asks for start_date
# (floor 2017-01-01, ~3500 days): the call fails, nothing is written, the symbol
# never enters coverage.json["backfilled"], and every later run repeats the same
# doomed request. A CoinGecko-only commodity could therefore never bootstrap.
# So: classify the range error and retry at the widest window the tier allows.
# A paid key keeps its full history — the retry only fires after a real refusal.
CG_MAX_FREE_DAYS = 365
CG_RANGE_ERROR_CODE = 10012

# CoinGecko meters by calls-per-minute (~30 on a demo key, fewer anonymously) and
# answers a burst with the same 401/429 as any other refusal. Every call in this
# module goes through one throttle, so the main loop, the discover search and the
# over-range retry cannot collectively outrun the limit — pacing in the caller
# cannot see the retry, and a half-rate-limited run leaves prices frozen.
CG_MIN_INTERVAL_KEY = 2.2
CG_MIN_INTERVAL_ANON = 2.5
_cg_last_call = 0.0


def _coingecko_throttle(api_key):
    global _cg_last_call
    wait = (CG_MIN_INTERVAL_KEY if api_key else CG_MIN_INTERVAL_ANON) - (time.monotonic() - _cg_last_call)
    if wait > 0:
        time.sleep(wait)
    _cg_last_call = time.monotonic()


def _coingecko_error(e):
    """Classify an HTTPError from CoinGecko. 'out-of-range' is retryable."""
    if e.code in (400, 401):
        try:
            body = json.loads(e.read())
            if body.get("error", {}).get("status", {}).get("error_code") == CG_RANGE_ERROR_CODE:
                return "out-of-range"
        except Exception:
            pass
    return "rate-limited" if e.code in (401, 429) else f"HTTP {e.code}"


def _coingecko_chart(coin_id, days, api_key):
    url = (f"https://api.coingecko.com/api/v3/coins/{coin_id}/market_chart"
           f"?vs_currency=usd&days={days}&interval=daily")
    headers = {"Accept": "application/json"}
    if api_key:
        headers["x-cg-demo-api-key"] = api_key
    _coingecko_throttle(api_key)
    try:
        with urllib.request.urlopen(urllib.request.Request(url, headers=headers), timeout=30) as r:
            return json.load(r)
    except HTTPError as e:
        raise RuntimeError(_coingecko_error(e))
    except URLError as e:
        raise RuntimeError(f"net {e.reason}")


def fetch_coingecko(coin_id, start_date, api_key=None):
    days = max(1, (datetime.date.today() - datetime.datetime.strptime(start_date, "%Y-%m-%d").date()).days + 1)
    try:
        data = _coingecko_chart(coin_id, days, api_key)
    except RuntimeError as e:
        if str(e) != "out-of-range" or days <= CG_MAX_FREE_DAYS:
            raise
        data = _coingecko_chart(coin_id, CG_MAX_FREE_DAYS, api_key)
    seen, out = set(), []
    for ts, price in data.get("prices", []):
        d = _day(ts)
        if d not in seen:
            seen.add(d)
            out.append((d, price))
    return sorted(out)


def fetch_yfinance(ticker, start_date):
    try:
        import yfinance as yf
    except ImportError:
        raise RuntimeError("yfinance unavailable")
    span = (datetime.date.today() - datetime.datetime.strptime(start_date, "%Y-%m-%d").date()).days
    data = yf.download(ticker, period="max" if span > 730 else "2y", progress=False, auto_adjust=False)
    if data is None or data.empty:
        raise RuntimeError("no data")
    close = data["Close"]
    if hasattr(close, "iloc") and getattr(close, "ndim", 1) == 2:
        close = close.iloc[:, 0]
    return [(ts.strftime("%Y-%m-%d"), float(p)) for ts, p in close.dropna().items()]


def fetch_source(source, value, start_date, cg_key):
    if source == "binance":
        return fetch_binance(value, start_date)
    if source == "coingecko":
        return fetch_coingecko(value, start_date, cg_key)
    if source == "yfinance":
        return fetch_yfinance(value, start_date)
    raise RuntimeError(f"unknown source '{source}'")


# --- io ----------------------------------------------------------------------
def load_existing(path):
    ex = {}
    if os.path.exists(path):
        with open(path) as f:
            for line in f:
                p = line.split()
                if len(p) >= 5 and p[0] == "P":
                    ex[p[1]] = p[3]
    return ex


def write_file(path, sym, prices, existing):
    merged = dict(existing)
    n = 0
    for d, price in prices:
        if d not in merged:
            n += 1
        merged[d] = _fmt(price)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        for d in sorted(merged):
            f.write(f"P {d} {sym} {merged[d]} USD\n")
    return n


def main():
    ctx = json.load(sys.stdin)
    cfg = ctx.get("config", {})
    secrets = ctx.get("secrets", {})
    sources_dir = ctx["sources_dir"]
    data_dir = ctx.get("data_dir") or os.path.join(os.path.dirname(sources_dir.rstrip("/")), ".data")

    quote = cfg.get("quote_pair", "USDT")
    start_floor = str(cfg.get("start_date", "2017-01-01"))[:10]
    refresh_days = int(cfg.get("refresh_days", 7))
    max_age = int(cfg.get("max_age_days", 3))
    force = bool(cfg.get("force_backfill", False))
    do_discover = bool(cfg.get("discover", True))
    discover_limit = int(cfg.get("discover_limit", 150))
    cg_key = secrets.get("coingecko_api_key") or None
    today = datetime.date.today().isoformat()

    commodities_path = os.path.join(data_dir, cfg.get("commodities_file", "commodities.txt"))
    commodities = parse_commodities(commodities_path)

    state_path = os.path.join(data_dir, "coverage.json")
    state = {}
    if os.path.exists(state_path):
        try:
            state = json.load(open(state_path))
        except Exception:
            state = {}
    backfilled = set(state.get("backfilled", []))
    skip_set = set(state.get("discover_skip", []))

    discovered, warnings = [], []
    if do_discover:
        generated_dir = os.path.join(os.path.dirname(sources_dir.rstrip("/")), "generated")
        discovered, dwarn = discover(commodities, commodities_path, sources_dir, generated_dir, quote, cg_key, skip_set, today, discover_limit, max_age)
        warnings.extend(dwarn)

    recent = _add_days(today, -refresh_days)
    written, did_backfill = [], []
    for sym, sources in commodities.items():
        if not sources:
            warnings.append(f"{sym}: no source declared")
            continue
        path = os.path.join(sources_dir, "_prices", f"{price_file_stem(sym)}.txt")
        is_backfill = force or sym not in backfilled
        fetch_start = start_floor if is_backfill else recent
        prices, used, tried = None, None, []
        for source, value in sources:
            try:
                p = fetch_source(source, value, fetch_start, cg_key)
                if _stale(p, max_age):
                    tried.append(f"{source}:{value}=stale")
                else:
                    prices, used = p, f"{source}:{value}"
                    break
            except Exception as e:
                tried.append(f"{source}:{value}={str(e)[:40]}")
        if prices:
            n = write_file(path, sym, prices, load_existing(path))
            backfilled.add(sym)
            if is_backfill:
                did_backfill.append(sym)
            if n > 0:
                written.append(f"{sym}<-{'backfill' if is_backfill else used} (+{n})")
        else:
            warnings.append(f"{sym}: {'; '.join(tried)}")
        time.sleep(0.1)

    try:
        os.makedirs(data_dir, exist_ok=True)
        json.dump({"backfilled": sorted(backfilled), "discover_skip": sorted(skip_set)},
                  open(state_path, "w"), indent=2)
    except Exception:
        pass

    print(json.dumps({
        "files_written": written,
        "backfilled": did_backfill,
        "discovered": discovered,
        "warnings": warnings,
    }, indent=2))
    if warnings and not written:
        sys.exit(1)
    elif warnings:
        sys.exit(2)


if __name__ == "__main__":
    main()
