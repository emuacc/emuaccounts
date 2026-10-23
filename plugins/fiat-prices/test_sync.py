"""Unit tests for the fiat-prices plugin. No network: fetch_fx_close is patched."""

import io
import json
import os
import sys

import sync


# --- price-file name encoder (must mirror ledger_parser::price_file_stem) ----
def test_price_file_stem_matches_rust_encoder():
    assert sync.price_file_stem("xyz:WTIOIL") == "xyz%3AWTIOIL"
    assert sync.price_file_stem("HYPERLIQUID:BTC") == "HYPERLIQUID%3ABTC"
    assert sync.price_file_stem('a:/\\*?"<>|b') == "a%3A%2F%5C%2A%3F%22%3C%3E%7Cb"


def test_price_file_stem_leaves_existing_names_byte_identical():
    for name in ["AUD", "BTC", "BRK.B", "RAY-SOL", "ATOM.S", "aEthUSDC", "比特币"]:
        assert sync.price_file_stem(name) == name


def test_commodity_from_price_stem_round_trips():
    assert sync.commodity_from_price_stem("xyz%3AWTIOIL") == "xyz:WTIOIL"
    for name in ["BRK.B", "RAY-SOL", "ATOM.S", "xyz:WTIOIL", "AUD", 'a:/\\*?"<>|b']:
        assert sync.commodity_from_price_stem(sync.price_file_stem(name)) == name


# --- writing ------------------------------------------------------------------
def test_write_prices_merges_and_keeps_currency_on_p_line(tmp_path):
    fp = tmp_path / "AUD.txt"
    n = sync.write_prices(str(fp), "AUD", [("2026-01-16", 0.654321)], {"2026-01-15": "0.650000"})
    assert n == 1
    assert fp.read_text().splitlines() == [
        "P 2026-01-15 AUD 0.650000 USD",
        "P 2026-01-16 AUD 0.654321 USD",
    ]


def test_main_builds_file_names_with_the_shared_encoder(tmp_path, monkeypatch, capsys):
    """Currency codes never contain unsafe characters; this pins the wiring so
    the fiat plugin names files exactly like the other price plugins."""
    sources = tmp_path / "sources"
    os.makedirs(sources / "_prices")
    (sources / "_prices" / "AUD.txt").write_text("P 2026-01-15 AUD 0.650000 USD\n")
    monkeypatch.setattr(sync, "fetch_fx_close", lambda currency, period: [("2026-01-16", 0.66)])
    ctx = {"config": {"currencies": "AUD,USD,xyz:WTIOIL"}, "sources_dir": str(sources)}
    monkeypatch.setattr(sys, "stdin", io.StringIO(json.dumps(ctx)))

    sync.main()  # clean run -> no SystemExit
    out = json.loads(capsys.readouterr().out)
    assert out["warnings"] == []
    assert out["files_written"] == ["_prices/AUD.txt", "_prices/XYZ%3AWTIOIL.txt"]
    assert sorted(os.listdir(sources / "_prices")) == ["AUD.txt", "XYZ%3AWTIOIL.txt"]
    # Existing AUD file merged (read via the same encoded path); P line keeps the true name.
    assert (sources / "_prices" / "AUD.txt").read_text().splitlines() == [
        "P 2026-01-15 AUD 0.650000 USD",
        "P 2026-01-16 AUD 0.660000 USD",
    ]
    assert (sources / "_prices" / "XYZ%3AWTIOIL.txt").read_text() == "P 2026-01-16 XYZ:WTIOIL 0.660000 USD\n"
