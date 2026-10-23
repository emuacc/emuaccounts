"""Unit tests for the crypto-spot-prices plugin. No network: binance_has and
fetch_source are patched."""

import datetime
import io
import json
import os
import sys

import sync

# The names the ledger grammar accepts that the old `[A-Za-z][A-Za-z0-9_]*`
# scan missed, plus the EXCHANGE:SYMBOL form (decision-001).
GRAMMAR_NAMES = ["BRK.B", "RAY-SOL", "ATOM.S", "xyz:WTIOIL"]


# --- price-file name encoder (must mirror ledger_parser::price_file_stem) ----
def test_price_file_stem_matches_rust_encoder():
    assert sync.price_file_stem("xyz:WTIOIL") == "xyz%3AWTIOIL"
    assert sync.price_file_stem("HYPERLIQUID:BTC") == "HYPERLIQUID%3ABTC"
    # The whole filename-unsafe set, uppercase hex of the ASCII code.
    assert sync.price_file_stem('a:/\\*?"<>|b') == "a%3A%2F%5C%2A%3F%22%3C%3E%7Cb"


def test_price_file_stem_leaves_existing_names_byte_identical():
    for name in ["BTC", "BRK.B", "RAY-SOL", "ATOM.S", "aEthUSDC", "BTC_v2", "比特币"]:
        assert sync.price_file_stem(name) == name


def test_commodity_from_price_stem_round_trips():
    assert sync.commodity_from_price_stem("xyz%3AWTIOIL") == "xyz:WTIOIL"
    for name in GRAMMAR_NAMES + ["BTC", "aEthUSDC", "比特币", 'a:/\\*?"<>|b']:
        assert sync.commodity_from_price_stem(sync.price_file_stem(name)) == name


# --- discovery regexes -------------------------------------------------------
def test_posting_regex_captures_every_grammar_commodity():
    for name in GRAMMAR_NAMES + ["BTC", "aEthUSDC", "比特币"]:
        m = sync.POSTING_RE.match(f"    assets:crypto:wallet:x  -1,234.50 {name}")
        assert m and m.group(2) == name, name
    # Trailing price / cost / comment tokens never leak into the commodity.
    m = sync.POSTING_RE.match("    assets:wallet 10.00 xyz:WTIOIL @ 94.09 USD  ; leg:abc")
    assert m.group(2) == "xyz:WTIOIL"
    # Header lines are not postings.
    assert sync.POSTING_RE.match('2026-01-15 * "Hyperliquid" "Buy" ; txn:1') is None


def test_ticker_re_accepts_grammar_names_rejects_ui_spelling():
    for name in GRAMMAR_NAMES:
        assert sync.TICKER_RE.match(name), name
    # Hyperliquid's UI-export spelling has a space and parens — never a commodity.
    assert not sync.TICKER_RE.match("WTIOIL (xyz)")


def test_is_candidate_probes_only_spot_looking_tickers():
    # Scanning sees everything; probing is reserved for names that can exist on
    # a spot market. Venue-scoped, dotted, dashed and LP names are left alone.
    assert sync.is_candidate("BTC")
    assert sync.is_candidate("aEthUSDC")
    for name in ("xyz:WTIOIL", "ATOM.S", "RAY-SOL", "BRK.B", "WTIOIL (xyz)", "BTC USDT",
                 "RAY-SOL-LP", "mPEPEСLАlМmpepeco"):
        assert not sync.is_candidate(name), name


def test_parse_commodities_keeps_exchange_symbol_names(tmp_path):
    path = tmp_path / "commodities.txt"
    sync.append_commodity(str(path), "xyz:WTIOIL", [("yfinance", "CL=F")], "2026-08-25")
    assert sync.parse_commodities(str(path)) == {"xyz:WTIOIL": [("yfinance", "CL=F")]}


# --- discover: already-priced set decodes file names -------------------------
def test_discover_treats_encoded_price_file_as_already_priced(tmp_path, monkeypatch):
    sources = tmp_path / "sources"
    generated = tmp_path / "generated" / "wallet"
    os.makedirs(sources / "_prices")
    os.makedirs(generated)
    (sources / "_prices" / "xyz%3AWTIOIL.txt").write_text("P 2026-01-15 xyz:WTIOIL 94.09 USD\n")
    (sources / "_prices" / "BTC.txt").write_text("P 2026-01-15 BTC 100000.00 USD\n")
    (generated / "ledger.transactions").write_text(
        "2026-01-15 * \"Hyperliquid\" \"Buy\" ; txn:1\n"
        "    assets:crypto:hyperliquid 10.00 xyz:WTIOIL\n"
        "    assets:crypto:hyperliquid -1.00 BTC\n"
        "    assets:crypto:kraken 5.00 SOL\n"
        "    assets:crypto:kraken 5.00 ATOM.S\n"
    )
    probed = []
    monkeypatch.setattr(sync, "binance_has", lambda pair, max_age: probed.append(pair) or False)

    commodities, skip = {}, set()
    added, warnings = sync.discover(
        commodities, str(tmp_path / "commodities.txt"), str(sources), str(tmp_path / "generated"),
        "USDT", None, skip, "2026-08-25", 150, 3,
    )
    # xyz:WTIOIL and BTC already have price files -> not re-discovered; the
    # unpriced SOL is probed; ATOM.S is unpriced but not spot-looking -> no call.
    assert probed == ["SOLUSDT"]
    assert added == []
    assert skip == {"SOL"}  # probed and rejected -> remembered; ATOM.S never probed
    assert len(warnings) == 1 and "SOL held (1 txns)" in warnings[0]


# --- writing: encoded file name, true commodity on the P line ----------------
def test_write_file_keeps_true_commodity_on_p_line(tmp_path):
    path = tmp_path / "_prices" / sync.price_file_stem("xyz:WTIOIL")
    n = sync.write_file(str(path), "xyz:WTIOIL", [("2026-01-15", 94.09)], {})
    assert n == 1
    assert path.name == "xyz%3AWTIOIL"
    assert path.read_text() == "P 2026-01-15 xyz:WTIOIL 94.09 USD\n"


def test_main_writes_encoded_file_names(tmp_path, monkeypatch, capsys):
    sources = tmp_path / "sources"
    data = tmp_path / ".data"
    os.makedirs(sources)
    os.makedirs(data)
    (data / "commodities.txt").write_text(
        "commodity xyz:WTIOIL\n  yfinance CL=F\n"
        "commodity BTC\n  binance BTCUSDT\n"
    )
    today = datetime.date.today().isoformat()
    monkeypatch.setattr(sync, "fetch_source", lambda src, val, start, key: [(today, 94.09)])
    monkeypatch.setattr(sync.time, "sleep", lambda s: None)
    ctx = {"config": {"discover": False}, "secrets": {},
           "sources_dir": str(sources), "data_dir": str(data)}
    monkeypatch.setattr(sys, "stdin", io.StringIO(json.dumps(ctx)))

    sync.main()  # clean run -> no SystemExit
    out = json.loads(capsys.readouterr().out)
    assert out["warnings"] == []
    assert sorted(os.listdir(sources / "_prices")) == ["BTC.txt", "xyz%3AWTIOIL.txt"]
    assert (sources / "_prices" / "xyz%3AWTIOIL.txt").read_text() == f"P {today} xyz:WTIOIL 94.09 USD\n"
    assert (sources / "_prices" / "BTC.txt").read_text() == f"P {today} BTC 94.09 USD\n"
    # Second run merges into the encoded file (load_existing reads the same path).
    monkeypatch.setattr(sys, "stdin", io.StringIO(json.dumps(ctx)))
    monkeypatch.setattr(sync, "fetch_source", lambda src, val, start, key: [(today, 95.00)])
    sync.main()
    capsys.readouterr()
    assert (sources / "_prices" / "xyz%3AWTIOIL.txt").read_text() == f"P {today} xyz:WTIOIL 95.00 USD\n"


# --- coingecko: the 365-day cap is a 401, not a 400 --------------------------
class _FakeResponse:
    def __init__(self, payload):
        self._payload = payload

    def read(self):
        return json.dumps(self._payload).encode()

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        return False


def _range_error():
    """CoinGecko's real over-range reply: 401 carrying error_code 10012."""
    body = {"error": {"status": {"error_code": 10012,
                                 "error_message": "Your request exceeds the allowed time range."}}}
    return sync.HTTPError("http://cg", 401, "Unauthorized", {},
                          io.BytesIO(json.dumps(body).encode()))


def _plain_401():
    """A spent rate limit: same status, no error_code."""
    return sync.HTTPError("http://cg", 401, "Unauthorized", {}, io.BytesIO(b"{}"))


def _patch_urlopen(monkeypatch, handler):
    """Fake the HTTP layer; record the `days` of each call and every throttle
    sleep (recorded, never actually slept, so the suite stays fast)."""
    calls, slept = [], []

    def fake(req, timeout=None):
        days = int(req.full_url.split("days=")[1].split("&")[0])
        calls.append(days)
        return handler(days)

    monkeypatch.setattr(sync.urllib.request, "urlopen", fake)
    monkeypatch.setattr(sync.time, "sleep", lambda s: slept.append(s))
    monkeypatch.setattr(sync, "_cg_last_call", 0.0)
    return calls, slept


def test_coingecko_over_range_retries_at_the_tier_cap(monkeypatch):
    """A full backfill (~3500 days) must not die on the free/demo 365-day cap.

    Regression: the 401 was read as "rate-limited", the source was dropped, the
    symbol never reached coverage.json["backfilled"], and every later run
    repeated the identical doomed request — freezing the price forever.
    """
    day_ms = 1_767_225_600_000  # 2026-01-01

    def handler(days):
        if days > sync.CG_MAX_FREE_DAYS:
            raise _range_error()
        return _FakeResponse({"prices": [[day_ms, 1.25], [day_ms + sync.DAY_MS, 1.30]]})

    calls, _ = _patch_urlopen(monkeypatch, handler)
    out = sync.fetch_coingecko("mango-markets", "2017-01-01", "demo-key")

    assert [p for _, p in out] == [1.25, 1.30]
    assert len(calls) == 2 and calls[0] > sync.CG_MAX_FREE_DAYS
    assert calls[1] == sync.CG_MAX_FREE_DAYS


def test_coingecko_real_rate_limit_still_raises_without_retry(monkeypatch):
    """A 401 with no error_code is a genuine refusal — retrying would just burn
    another call against the same limit."""
    calls, _ = _patch_urlopen(monkeypatch, lambda days: (_ for _ in ()).throw(_plain_401()))
    try:
        sync.fetch_coingecko("maker", "2017-01-01", None)
    except RuntimeError as e:
        assert str(e) == "rate-limited"
    else:
        raise AssertionError("expected RuntimeError")
    assert len(calls) == 1


def test_coingecko_over_range_inside_cap_is_not_retried(monkeypatch):
    """An incremental refresh already sits inside the cap; a range error there is
    not something a narrower window can fix, so it must surface, not loop."""
    calls, _ = _patch_urlopen(monkeypatch, lambda days: (_ for _ in ()).throw(_range_error()))
    start = sync._add_days(datetime.date.today().isoformat(), -7)
    try:
        sync.fetch_coingecko("helium", start, "demo-key")
    except RuntimeError as e:
        assert str(e) == "out-of-range"
    else:
        raise AssertionError("expected RuntimeError")
    assert len(calls) == 1


def test_coingecko_backfill_reaches_write_file_after_retry(tmp_path, monkeypatch, capsys):
    """End to end: a declared CoinGecko-only commodity now bootstraps, gets a
    price file, and is recorded as backfilled so later runs go incremental."""
    sources, data = tmp_path / "sources", tmp_path / ".data"
    os.makedirs(sources)
    os.makedirs(data)
    (data / "commodities.txt").write_text("commodity MNGO\n  coingecko mango-markets\n")
    today_ms = int(datetime.datetime.now(datetime.timezone.utc).timestamp() * 1000)

    def handler(days):
        if days > sync.CG_MAX_FREE_DAYS:
            raise _range_error()
        return _FakeResponse({"prices": [[today_ms, 0.032032]]})

    _patch_urlopen(monkeypatch, handler)
    monkeypatch.setattr(sync.time, "sleep", lambda s: None)
    ctx = {"config": {"discover": False}, "secrets": {"coingecko_api_key": "demo-key"},
           "sources_dir": str(sources), "data_dir": str(data)}
    monkeypatch.setattr(sys, "stdin", io.StringIO(json.dumps(ctx)))

    sync.main()
    out = json.loads(capsys.readouterr().out)
    assert out["warnings"] == []
    assert out["backfilled"] == ["MNGO"]
    assert "MNGO" in json.load(open(data / "coverage.json"))["backfilled"]
    # 0.032032 -> 4dp by _fmt's adaptive precision (0.01 <= p < 1).
    assert (sources / "_prices" / "MNGO.txt").read_text().endswith("MNGO 0.0320 USD\n")


def test_coingecko_calls_are_throttled_even_with_an_api_key(monkeypatch):
    """Regression: pacing fired only when there was NO api key, so a demo key —
    metered at ~30 calls/min — was hammered flat out and most of a run came back
    rate-limited, leaving those prices frozen. Having a key made it worse."""
    _, slept = _patch_urlopen(monkeypatch, lambda days: _FakeResponse({"prices": []}))
    for coin in ("mango-markets", "lifinity", "maker"):
        sync.fetch_coingecko(coin, "2026-09-01", "demo-key")
    # First call is free, each subsequent one waits out the interval.
    assert len(slept) == 2
    assert all(0 < s <= sync.CG_MIN_INTERVAL_KEY for s in slept)


def test_coingecko_throttle_covers_the_over_range_retry(monkeypatch):
    """The retry is invisible to any pacing in the caller, so it must be paced
    where the call is actually made — otherwise a backfill fires two rapid calls
    per commodity and outruns the limit it is trying to respect."""
    def handler(days):
        if days > sync.CG_MAX_FREE_DAYS:
            raise _range_error()
        return _FakeResponse({"prices": []})

    calls, slept = _patch_urlopen(monkeypatch, handler)
    sync.fetch_coingecko("mango-markets", "2017-01-01", "demo-key")
    assert len(calls) == 2
    assert len(slept) == 1 and 0 < slept[0] <= sync.CG_MIN_INTERVAL_KEY
