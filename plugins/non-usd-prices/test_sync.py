"""Unit tests for the non-usd-prices plugin. No network: fetch_yahoo is patched."""

import datetime
import io
import json
import os
import sys

import sync


def _prices(tmp_path, files):
    d = tmp_path / "sources" / "_prices"
    os.makedirs(d)
    for name, body in files.items():
        (d / name).write_text(body)
    return d


def _run(tmp_path, config, monkeypatch, capsys):
    ctx = {"config": config, "secrets": {},
           "sources_dir": str(tmp_path / "sources"), "data_dir": str(tmp_path / ".data")}
    monkeypatch.setattr(sys, "stdin", io.StringIO(json.dumps(ctx)))
    sync.main()
    return json.loads(capsys.readouterr().out)


# --- config parsing ----------------------------------------------------------
def test_parse_pairs_splits_on_the_first_equals_only():
    # A Yahoo FX ticker contains an '=' of its own; splitting on all of them
    # would turn GBPAUD=X into a truncated symbol.
    assert sync.parse_pairs("GBP=GBPAUD=X,VUKG=VUKG.L") == [("GBP", "GBPAUD=X"), ("VUKG", "VUKG.L")]
    assert sync.parse_pairs(" USD=AUD ") == [("USD", "AUD")]


def test_quote_currency_comes_from_the_ticker_for_fx_and_yahoo_for_equities():
    assert sync.quote_for("GBP", "GBPAUD=X", None) == "AUD"
    assert sync.quote_for("EUR", "EURUSD=X", "whatever") == "USD"
    assert sync.quote_for("VUKG", "VUKG.L", "GBP") == "GBP"


# --- inversion ---------------------------------------------------------------
def test_invert_series_reciprocates_the_usd_quoted_file(tmp_path):
    d = _prices(tmp_path, {"AUD.txt": "P 2026-09-13 AUD 0.717257 USD\n"})
    assert sync.invert_series(str(d), "AUD", 4) == {"2026-09-13": 1.3942}


def test_invert_skips_a_zero_rate_rather_than_dividing_by_it(tmp_path):
    d = _prices(tmp_path, {"AUD.txt": "P 2026-09-12 AUD 0 USD\nP 2026-09-13 AUD 0.717257 USD\n"})
    assert sync.invert_series(str(d), "AUD", 4) == {"2026-09-13": 1.3942}


# --- forward fill ------------------------------------------------------------
def test_forward_fill_carries_the_close_across_a_weekend():
    out = sync.forward_fill({"2026-09-11": 1.39, "2026-09-14": 1.40}, "2026-09-15")
    assert out == {"2026-09-11": 1.39, "2026-09-12": 1.39, "2026-09-13": 1.39,
                   "2026-09-14": 1.40, "2026-09-15": 1.40}


# --- the three merge invariants ----------------------------------------------
def test_never_prepends_before_the_files_first_row(tmp_path, monkeypatch, capsys):
    """AUD.txt reaches back to 2006 and USD.txt starts 2015. Inverting the whole
    of AUD.txt would invent a decade of USD history from a source unrelated to
    the rows already in the file."""
    _prices(tmp_path, {
        "AUD.txt": "P 2006-05-16 AUD 0.767 USD\nP 2015-12-31 AUD 0.731 USD\nP 2016-01-01 AUD 0.730 USD\n",
        "USD.txt": "P 2015-12-31 USD 1.3683 AUD\n",
    })
    out = _run(tmp_path, {"invert": "USD=AUD", "yahoo": "", "forward_fill": ""}, monkeypatch, capsys)
    assert out["warnings"] == []
    lines = (tmp_path / "sources" / "_prices" / "USD.txt").read_text().splitlines()
    assert not any(line.split()[1] < "2015-12-31" for line in lines)
    assert lines[0] == "P 2015-12-31 USD 1.3683 AUD"  # original row untouched
    assert lines[-1] == "P 2016-01-01 USD 1.3699 AUD"


def test_preserves_the_original_date_token(tmp_path, monkeypatch, capsys):
    """VUKG.txt spells dates `2019-05-14T00:00:00.000000Z`. Normalising those
    would rewrite every historical line while claiming to leave it alone."""
    _prices(tmp_path, {"VUKG.txt": "P 2019-05-14T00:00:00.000000Z VUKG 30.0000 GBP\n"})
    monkeypatch.setattr(sync, "fetch_yahoo", lambda t, p: ({"2026-09-11": 57.4}, "GBP"))
    _run(tmp_path, {"invert": "", "yahoo": "VUKG=VUKG.L", "forward_fill": ""}, monkeypatch, capsys)
    lines = (tmp_path / "sources" / "_prices" / "VUKG.txt").read_text().splitlines()
    assert lines[0] == "P 2019-05-14T00:00:00.000000Z VUKG 30.0000 GBP"
    assert lines[-1] == "P 2026-09-11 VUKG 57.4000 GBP"


def test_rewrites_the_recent_window_but_leaves_history_alone(tmp_path, monkeypatch, capsys):
    """A close captured on the day of a run is provisional; the venue settles it
    later. Recent rows must be re-fetched, older ones must not."""
    today = datetime.date.today()
    recent = (today - datetime.timedelta(days=2)).isoformat()
    old = (today - datetime.timedelta(days=60)).isoformat()
    _prices(tmp_path, {"GBP.txt": f"P {old} GBP 1.1111 AUD\nP {recent} GBP 2.2222 AUD\n"})
    monkeypatch.setattr(sync, "fetch_yahoo", lambda t, p: ({old: 9.9999, recent: 1.8893}, "AUD"))
    _run(tmp_path, {"invert": "", "yahoo": "GBP=GBPAUD=X", "forward_fill": ""}, monkeypatch, capsys)
    body = (tmp_path / "sources" / "_prices" / "GBP.txt").read_text()
    assert f"P {old} GBP 1.1111 AUD" in body      # outside the window: untouched
    assert f"P {recent} GBP 1.8893 AUD" in body   # inside: corrected
    assert "9.9999" not in body


def test_gap_inside_the_existing_range_is_filled(tmp_path, monkeypatch, capsys):
    _prices(tmp_path, {
        "AUD.txt": "P 2016-01-01 AUD 0.730 USD\nP 2016-01-02 AUD 0.729 USD\nP 2016-01-03 AUD 0.728 USD\n",
        "USD.txt": "P 2016-01-01 USD 1.3699 AUD\nP 2016-01-03 USD 1.3736 AUD\n",
    })
    _run(tmp_path, {"invert": "USD=AUD", "yahoo": "", "forward_fill": ""}, monkeypatch, capsys)
    body = (tmp_path / "sources" / "_prices" / "USD.txt").read_text()
    assert "P 2016-01-02 USD 1.3717 AUD" in body


class _FakeYFinance:
    """Just enough yfinance for fetch_yahoo: a Close frame and fast_info."""

    def __init__(self, currency, closes):
        self._currency = currency
        self._closes = closes

    def download(self, ticker, period=None, progress=None, auto_adjust=None):
        import pandas as pd

        idx = pd.to_datetime(list(self._closes))
        return pd.DataFrame({("Close", ticker): list(self._closes.values())}, index=idx)

    def Ticker(self, ticker):  # noqa: N802 — mirrors yfinance's own spelling
        import types

        return types.SimpleNamespace(fast_info={"currency": self._currency})


def test_pence_quoted_equity_is_converted_to_pounds(monkeypatch):
    """Most LSE tickers quote in GBp. Writing those unscaled would overstate the
    holding 100x — VUKG happens to quote in GBP, so this is easy to miss."""
    monkeypatch.setitem(sys.modules, "yfinance", _FakeYFinance("GBp", {"2026-09-11": 5740.0}))
    series, currency = sync.fetch_yahoo("SOMETHING.L", "max")
    assert currency == "GBP"
    assert series == {"2026-09-11": 57.4}


def test_pound_quoted_equity_is_left_alone(monkeypatch):
    monkeypatch.setitem(sys.modules, "yfinance", _FakeYFinance("GBP", {"2026-09-11": 57.4}))
    series, currency = sync.fetch_yahoo("VUKG.L", "max")
    assert currency == "GBP"
    assert series == {"2026-09-11": 57.4}


def test_a_failing_source_warns_without_touching_the_file(tmp_path, monkeypatch, capsys):
    _prices(tmp_path, {"GBP.txt": "P 2026-01-01 GBP 1.9000 AUD\n"})
    before = (tmp_path / "sources" / "_prices" / "GBP.txt").read_text()

    def boom(ticker, period):
        raise RuntimeError("no data")

    monkeypatch.setattr(sync, "fetch_yahoo", boom)
    try:
        out = _run(tmp_path, {"invert": "", "yahoo": "GBP=GBPAUD=X"}, monkeypatch, capsys)
    except SystemExit as e:            # nothing written + warnings -> exit 1
        assert e.code == 1
    else:
        assert out["warnings"]
    assert (tmp_path / "sources" / "_prices" / "GBP.txt").read_text() == before


def test_missing_invert_source_is_a_warning_not_a_crash(tmp_path, monkeypatch, capsys):
    _prices(tmp_path, {"USD.txt": "P 2016-01-01 USD 1.3699 AUD\n"})
    try:
        out = _run(tmp_path, {"invert": "USD=AUD", "yahoo": ""}, monkeypatch, capsys)
    except SystemExit as e:
        assert e.code == 1
    else:
        assert any("AUD.txt not found" in w for w in out["warnings"])


def test_price_file_stem_matches_the_rust_encoder():
    assert sync.price_file_stem("xyz:WTIOIL") == "xyz%3AWTIOIL"
    for name in ["USD", "GBP", "VUKG", "BRK.B"]:
        assert sync.price_file_stem(name) == name
