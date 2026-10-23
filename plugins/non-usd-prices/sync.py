#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# dependencies = ["yfinance>=0.2"]
# ///
"""Daily refresh for the price files quoted in the base currency, not USD.

Every other price plugin quotes in USD. `PriceGraph` converts to the base
currency by a direct price or **at most one hop** (`convert_to_base`,
src-tauri/src/ledger_parser.rs), so a vault whose base is not USD needs a few
files that deliberately break the USD convention. With the defaults (AUD base):

    USD.txt   USD quoted in AUD   the leg EVERY usd-quoted commodity hops through
    GBP.txt   GBP quoted in AUD   direct, so GBP cash needs no USD bridge
    VUKG.txt  VUKG quoted in GBP  VUKG -> GBP -> AUD is one hop; via USD it would be two

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

# --- price-file names --------------------------------------------------------
# MUST stay identical to `price_file_stem` in src-tauri/src/ledger_parser.rs.
# Plugins are copied into vaults standalone, so there is no shared module: every
# price plugin carries the same copy.
_FILENAME_UNSAFE = ':/\\*?"<>|'


def price_file_stem(commodity):
    return "".join("%%%02X" % ord(c) if c in _FILENAME_UNSAFE else c for c in commodity)


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
    """{YYYY-MM-DD: (date_token, price_text, quote)}.

    The date TOKEN is kept as written, not just the day: VUKG.txt spells its
    dates `2019-05-14T00:00:00.000000Z`, and normalising those would rewrite
    every historical line while claiming to leave it alone. Price is kept as
    text for the same reason — an untouched row must round-trip byte-identically.
    """
    out = {}
    if os.path.exists(path):
        with open(path) as f:
            for line in f:
                p = line.split()
                if len(p) >= 5 and p[0] == "P":
                    out[p[1][:10]] = (p[1], p[3], p[4])
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


def merge(existing, fresh, quote, cutoff, decimals):
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
        if date in merged and merged[date][1:] == (text, quote):
            continue
        token = merged[date][0] if date in merged else date
        merged[date] = (token, text, quote)
        changed += 1
    return merged, changed


def write_file(path, sym, merged):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f:
        for date in sorted(merged):
            token, price, quote = merged[date]
            f.write(f"P {token} {sym} {price} {quote}\n")


def main():
    ctx = json.load(sys.stdin)
    cfg = ctx.get("config", {})
    sources_dir = ctx["sources_dir"]
    prices_dir = os.path.join(sources_dir, "_prices")

    decimals = int(cfg.get("decimals", 4))
    refresh_days = int(cfg.get("refresh_days", 7))
    period = cfg.get("period", "max")
    fill = {s.strip() for s in str(cfg.get("forward_fill", "USD,GBP")).split(",") if s.strip()}
    today = datetime.date.today()
    cutoff = (today - datetime.timedelta(days=refresh_days)).isoformat()

    written, warnings = [], []

    def emit(sym, series, quote):
        path = os.path.join(prices_dir, price_file_stem(sym) + ".txt")
        existing = load_existing(path)
        if sym in fill:
            series = forward_fill(series, today.isoformat())
        merged, changed = merge(existing, series, quote, cutoff, decimals)
        if changed:
            write_file(path, sym, merged)
            written.append(f"{sym} ({quote}) +{changed} -> {max(merged)}")

    for sym, quote_sym in parse_pairs(str(cfg.get("invert", "USD=AUD"))):
        try:
            emit(sym, invert_series(prices_dir, quote_sym, decimals), quote_sym)
        except Exception as e:
            warnings.append(f"{sym}: invert {quote_sym} failed: {e}")

    for sym, ticker in parse_pairs(str(cfg.get("yahoo", "GBP=GBPAUD=X,VUKG=VUKG.L"))):
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
