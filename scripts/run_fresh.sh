#!/usr/bin/env bash
# Launch EmuAcc with a fresh, empty profile — no vault selected, so the app
# starts on the vault picker. HOME is pointed at a throwaway temp dir so the
# real profile (~/Library/Application Support/com.cog32.emuacccovid) is
# untouched, and any existing instance keeps running.
set -euo pipefail

cd "$(dirname "$0")/.."

BIN=src-tauri/target/debug/emuacc-covid
needs_build=0
if [ ! -x "$BIN" ]; then
  needs_build=1
elif grep -aq "http://localhost:1420" "$BIN"; then
  # Built without tauri/custom-protocol: the dev URL is baked in, so the app
  # would load from the tauri-dev server and share its localStorage origin.
  echo "Binary was built without tauri/custom-protocol — rebuilding."
  needs_build=1
fi
if [ "$needs_build" = 1 ]; then
  echo "Building app (frontend + debug binary)..."
  npm run build
  cargo build --manifest-path src-tauri/Cargo.toml --bin emuacc-covid \
    --features tauri/custom-protocol
fi

PROFILE=$(mktemp -d "${TMPDIR:-/tmp}/emuacc-fresh.XXXXXX")
trap 'rm -rf "$PROFILE"' EXIT
echo "Fresh profile: $PROFILE"

# EMUACC_SOURCES_DIR / EMUACC_GENERATED_DIR would bypass the vault picker —
# make sure they aren't inherited from the calling shell.
env -u EMUACC_SOURCES_DIR -u EMUACC_GENERATED_DIR \
  HOME="$PROFILE" EMUACC_INSTANCE=fresh "$BIN"
