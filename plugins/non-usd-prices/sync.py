#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["yfinance>=0.2"]
# ///
"""Daily refresh for the price files quoted in the base currency, not USD.

Every other price plugin quotes in USD. `PriceGraph` converts to the base
currency by a direct price or **at most one hop** (`convert_to_base`,
src-tauri/src/ledger_parser.rs), so a vault whose base is not USD needs a few
files that deliberately break the USD convention. For an AUD base:

    USD.txt   USD quoted in AUD   the leg EVERY usd-quoted commodity hops through
    GBP.txt   GBP quoted in AUD   only for a holding quoted in GBP: a London listing
                                  converts X -> GBP -> AUD in one hop, via USD in two

The defaults write only USD.txt. A leg that serves a holding of your own goes in
`yahoo` (`GBP=GBPAUD=X`, plus the listing itself if no other plugin prices it).
fiat-prices writes GBP.txt quoted in USD by default, so take GBP out of its
`currencies` before adding it here: a price file holds one quote currency, and a
file already holding another is left untouched and reported (quote_conflict).

Files like these tend to be maintained by hand and to go stale silently: a
nearest-prior lookup carries a stale rate forward forever rather than dropping
it, so every base-currency figure in the app drifts without an error.

The inverted files are derived, not fetched: `fiat-prices` keeps AUD.txt as
AUD-quoted-in-USD, and `invert` turns it into USD.txt. That makes ordering matter
only cosmetically — if this runs before `fiat-prices` on a given day the newest
row lands one run later, and `refresh_days` rewrites it anyway.

WHAT IT WILL NOT DO: rows older than `refresh_days` are never touched. A
hand-built history may come from a source that no longer exists and cannot be
reproduced; overwriting it would silently rewrite years of valuations. Only the
recent window is rewritten, which is what corrects a provisional close captured
on the day of a run (observed in every file this was validated against: each
historical row matched the recipe exactly, only the final row differed).

Input (stdin JSON): {config, secrets, sources_dir, data_dir}
Output: JSON {files_written, warnings}.
"""
import datetime
import json
import os
import re
import sys

# The runner passes only .data/config.json, so these are what runs without one.
# They MUST match the [config] defaults in plugin.toml, which the Plugins view
# shows (test_script_defaults_are_the_manifest_defaults). They name no holding.
DEFAULTS = {
    "invert": "USD=AUD",
    "yahoo": "",
    "forward_fill": "USD",
    "refresh_days": 7,
    "period": "max",
    "decimals": 4,
}

# --- price-file names --------------------------------------------------------
# MUST stay identical to `price_file_stem` in src-tauri/src/ledger_parser.rs.
# Plugins are copied into vaults standalone, so there is no shared module: every
# price plugin carries the same copy.
_FILENAME_UNSAFE = ':/\\*?"<>|'


def price_file_stem(commodity):
    return "".join("%%%02X" % ord(c) if c in _FILENAME_UNSAFE else c for c in commodity)


# --- one quote currency per file ---------------------------------------------
# A price plugin reads a file back by date and rewrites every line under the one
# quote currency it writes. A row already quoted in another currency would be
# relabelled (GBP 1.9053 AUD becomes GBP 1.9053 USD), a silent revaluation, so a
# file holding any is left untouched and the conflict is reported instead.
#
# This MUST stay identical in every price plugin: like price_file_stem it is
# copied, not shared, because plugins are installed into vaults standalone.
def quote_conflict(path, quote):
    """A warning when a P row in `path` is quoted in anything but `quote`, else None."""
    others = set()
    if os.path.exists(path):
        with open(path) as f:
            for line in f:
                p = line.split()
                if len(p) >= 5 and p[0] == "P" and p[4] != quote:
                    others.add(p[4])
    if not others:
        return None
    return (f"_prices/{os.path.basename(path)} left untouched: it holds prices quoted in "
            f"{', '.join(sorted(others))} and this plugin writes {quote}")


FX_TICKER_RE = re.compile(r"^([A-Za-z]{3})([A-Za-z]{3})=X$")


def parse_pairs(spec):
    """"A=B, C=D" -> [(A, B), (C, D)]. Splits on the FIRST '=' so Yahoo tickers
    that contain one (GBPAUD=X) survive."""
    out = []
    for item in spec.split(","):
        item = item.strip()
        if not item:
            continue
        if "=" not in item:
            raise ValueError(f"expected SYMBOL=VALUE, got {item!r}")
        sym, value = item.split("=", 1)
        out.append((sym.strip(), value.strip()))
    return out


def load_existing(path):
    """{YYYY-MM-DD: (date_token, price_text)}. The quote is not kept: a file is
    only rewritten when every row already carries the quote it is written in.

    The date TOKEN is kept as written, not just the day: a stock-prices file
    spells its dates `2019-05-14T00:00:00.000000Z`, and normalising those would
    rewrite every historical line while claiming to leave it alone. Price is
    kept as text for the same reason — an untouched row must round-trip
    byte-identically.
    """
    out = {}
    if os.path.exists(path):
        with open(path) as f:
            for line in f:
                p = line.split()
                if len(p) >= 5 and p[0] == "P":
                    out[p[1][:10]] = (p[1], p[3])
    return out


def forward_fill(series, upto):
    """Carry the last close onto every calendar day through `upto` — the FX
    convention in these files, so a weekend valuation finds a row."""
    if not series:
        return series
    out = dict(series)
    day = datetime.date.fromisoformat(min(series))
    end = datetime.date.fromisoformat(upto)
    last = None
    while day <= end:
        key = day.isoformat()
        if key in out:
            last = out[key]
        elif last is not None:
            out[key] = last
        day += datetime.timedelta(days=1)
    return out


def invert_series(prices_dir, quote_sym, decimals):
    """USD-quoted <quote_sym>.txt -> <quote_sym>-quoted reciprocal series."""
    src = os.path.join(prices_dir, price_file_stem(quote_sym) + ".txt")
    if not os.path.exists(src):
        raise RuntimeError(f"{quote_sym}.txt not found (fiat-prices should write it)")
    out = {}
    with open(src) as f:
        for line in f:
            p = line.split()
            if len(p) >= 5 and p[0] == "P" and float(p[3]) != 0:
                out[p[1][:10]] = round(1 / float(p[3]), decimals)
    if not out:
        raise RuntimeError(f"{quote_sym}.txt has no usable rows")
    return out


def fetch_yahoo(ticker, period):
    """(series, native_currency). GBp (pence) is converted to GBP."""
    import yfinance as yf

    hist = yf.download(ticker, period=period, progress=False, auto_adjust=False)
    if hist is None or hist.empty:
        raise RuntimeError("no data")
    close = hist["Close"]
    try:
        currency = yf.Ticker(ticker).fast_info.get("currency")
    except Exception:
        currency = None
    scale = 0.01 if currency == "GBp" else 1.0
    if currency == "GBp":
        currency = "GBP"
    series = {}
    for ts, value in zip(close.index, close.iloc[:, 0].values):
        if value == value:  # drop NaN holidays
            series[str(ts.date())] = float(value) * scale
    return series, currency


def quote_for(sym, ticker, currency):
    """FX tickers name their quote (GBPAUD=X -> AUD); equities report theirs."""
    m = FX_TICKER_RE.match(ticker)
    if m:
        return m.group(2).upper()
    if not currency:
        raise RuntimeError(f"{sym}: yahoo did not report a currency for {ticker}")
    return currency.upper()


def merge(existing, fresh, cutoff, decimals):
    """Rows before `cutoff` are kept verbatim; from `cutoff` on, fresh wins.
    Gaps inside the existing range are filled. Nothing is ever PREPENDED: a
    source that reaches further back than the file (AUD.txt starts 2006, USD.txt
    2015) would otherwise invent a decade of history from a different source
    than the rows already there.

    Returns (merged, changed) where changed counts rows added or altered."""
    merged = dict(existing)
    floor = min(existing) if existing else None
    changed = 0
    for date, price in sorted(fresh.items()):
        if floor and date < floor:
            continue
        text = f"{price:.{decimals}f}"
        if date < cutoff and date in merged:
            continue
        if date in merged and merged[date][1] == text:
            continue
        token = merged[date][0] if date in merged else date
        merged[date] = (token, text)
        changed += 1
    return merged, changed


def write_file(path, sym, merged, quote):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        for date in sorted(merged):
            token, price = merged[date]
            f.write(f"P {token} {sym} {price} {quote}\n")


def main():
    ctx = json.load(sys.stdin)
    cfg = {**DEFAULTS, **ctx.get("config", {})}
    sources_dir = ctx["sources_dir"]
    prices_dir = os.path.join(sources_dir, "_prices")

    decimals = int(cfg["decimals"])
    refresh_days = int(cfg["refresh_days"])
    period = cfg["period"]
    fill = {s.strip() for s in str(cfg["forward_fill"]).split(",") if s.strip()}
    today = datetime.date.today()
    cutoff = (today - datetime.timedelta(days=refresh_days)).isoformat()

    written, warnings = [], []

    def emit(sym, series, quote):
        path = os.path.join(prices_dir, price_file_stem(sym) + ".txt")
        existing = load_existing(path)
        if sym in fill:
            series = forward_fill(series, today.isoformat())
        merged, changed = merge(existing, series, cutoff, decimals)
        if not changed:
            return
        conflict = quote_conflict(path, quote)
        if conflict:
            warnings.append(f"{sym}: {conflict}")
            return
        write_file(path, sym, merged, quote)
        written.append(f"{sym} ({quote}) +{changed} -> {max(merged)}")

    for sym, quote_sym in parse_pairs(str(cfg["invert"])):
        try:
            emit(sym, invert_series(prices_dir, quote_sym, decimals), quote_sym)
        except Exception as e:
            warnings.append(f"{sym}: invert {quote_sym} failed: {e}")

    for sym, ticker in parse_pairs(str(cfg["yahoo"])):
        try:
            series, currency = fetch_yahoo(ticker, period)
            emit(sym, series, quote_for(sym, ticker, currency))
        except Exception as e:
            warnings.append(f"{sym}: yahoo {ticker} failed: {e}")

    print(json.dumps({"files_written": written, "warnings": warnings}, indent=2))
    if warnings and not written:
        sys.exit(1)


if __name__ == "__main__":
    main()
