#!/usr/bin/env bash
# Runs the multi-device sync tests. Sync is compiled only with the `sync`
# cargo feature (docs/SYNC.md), so run_unit_tests.sh and run_feature_tests.sh
# never build or run it; the pre-push gate only type-checks it. Run this after
# changing the sync code or anything it uses.
set -euo pipefail
cd "$(dirname "$0")/.."

echo "Sync build: Rust unit + integration tests (cargo nextest --features sync)…"
cargo nextest run --manifest-path src-tauri/Cargo.toml --no-fail-fast --features sync \
  -E 'not binary(bdd)'

echo "Sync build: BDD scenarios in src-tauri/features/sync/ (cucumber-rs --features sync)…"
cargo test --manifest-path src-tauri/Cargo.toml --test bdd --features sync -- \
  --input '**/features/sync/*.feature'
