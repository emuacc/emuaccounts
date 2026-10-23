#!/usr/bin/env bash
# Launch EmuAcc in dev mode against a vault.
#
#   scripts/run_debug.sh                 # vault = current_root from the app config
#   scripts/run_debug.sh <vault-root>    # explicit vault (must contain sources/)
#   scripts/run_debug.sh --fresh [...]   # clear the pipeline cache first
#
# The app runs the pipeline itself at startup, and its cache re-transforms
# only the sources that changed. The cache doesn't hash the Rust code, so after
# changing transform or matching code bump MATCHER_VERSION or pass --fresh.
#
# EMUACC_INSTANCE (env) suffixes the window title so several vaults are
# distinguishable in the OS window list. A per-vault wrapper is a one-line
# script that exports it and execs this one with the vault:
#   EMUACC_INSTANCE=work exec scripts/run_debug.sh "$@" ~/vault
# scripts/run_fresh.sh launches a throwaway profile that starts on the vault picker.
#
# An explicit vault is applied through EMUACC_SOURCES_DIR/EMUACC_GENERATED_DIR
# (env > config), so the app config's current_root is left untouched — the
# sidebar's "Data Folder" label still shows the configured root, not this one.
set -euo pipefail

APP_DATA="$HOME/Library/Application Support/com.cog32.emuacccovid"
CONFIG_FILE="$APP_DATA/config.json"

FRESH=0
VAULT=""
for arg in "$@"; do
  case "$arg" in
    --fresh) FRESH=1 ;;
    -*) echo "error: unknown option $arg" >&2; exit 2 ;;
    *)
      if [ -n "$VAULT" ]; then
        echo "error: more than one vault root given ($VAULT, $arg)" >&2
        exit 2
      fi
      VAULT="$arg"
      ;;
  esac
done

if [ -n "$VAULT" ]; then
  ROOT="$VAULT"
  if [ ! -d "$ROOT/sources" ]; then
    echo "error: $ROOT is not a vault (no sources/ directory)" >&2
    exit 1
  fi
  echo "Vault (explicit): $ROOT${EMUACC_INSTANCE:+  [instance: $EMUACC_INSTANCE]}"
elif [ -f "$CONFIG_FILE" ]; then
  # Read current_root from the app config
  ROOT=$(python3 -c "import json,sys; c=json.load(open(sys.argv[1])); print(c.get('current_root',''))" "$CONFIG_FILE" 2>/dev/null || echo "")
  echo "Vault (config current_root): ${ROOT:-<none>}"
else
  ROOT=""
fi

if [ -n "$ROOT" ]; then
  export EMUACC_SOURCES_DIR="$ROOT/sources"
  export EMUACC_GENERATED_DIR="$ROOT/generated"
else
  export EMUACC_SOURCES_DIR="$APP_DATA/sources"
  export EMUACC_GENERATED_DIR="$APP_DATA/generated"
fi

echo "EMUACC_SOURCES_DIR=$EMUACC_SOURCES_DIR"
echo "EMUACC_GENERATED_DIR=$EMUACC_GENERATED_DIR"

cd "$(dirname "$0")/.."

# Deterministic "Add Plugins" catalog in dev (bundled resource_dir layouts vary).
export EMUACC_PLUGIN_CATALOG="$PWD/plugins"

# Kill previous dev server and Tauri instances
lsof -ti:1420 2>/dev/null | xargs kill 2>/dev/null || true
pkill -f 'emuacc-covid' 2>/dev/null || true

if [ -n "$ROOT" ] && [ -d "plugins" ]; then
  echo "Installing plugins to $ROOT/plugins..."
  mkdir -p "$ROOT/plugins"
  rsync -a --delete \
    --exclude='.data/' \
    --exclude='__pycache__/' \
    --exclude='*.pyc' \
    --exclude='.pytest_cache/' \
    --exclude='.ruff_cache/' \
    --exclude='wallets.json' \
    --exclude='accounts.json' \
    --exclude='.gitignore' \
    plugins/ "$ROOT/plugins/"

  # Don't run plugins here — they have side effects (write to sources/, hit
  # external APIs). uv resolves PEP 723 deps lazily on first plugin click,
  # which is fast enough (~150ms with the hardlinked wheel cache).
  # Surface a clear warning if uv is missing so PEP-723 plugins won't fail
  # mysteriously the first time the user clicks Run.
  if ! command -v uv >/dev/null 2>&1; then
    echo "  warning: \`uv\` not on PATH — plugins with PEP 723 deps will fail."
    echo "           Install via: brew install uv  (or  curl -LsSf https://astral.sh/uv/install.sh | sh)"
  fi
fi

# --fresh: drop the derived caches so every source is re-transformed at app
# startup. ui-state.json stays: it's the vault's persisted nav state, not a
# build artifact.
if [ "$FRESH" = 1 ]; then
  echo "Clearing pipeline cache (--fresh; preserving ui-state.json)..."
  if [ -d "$EMUACC_GENERATED_DIR/.cache" ]; then
    find "$EMUACC_GENERATED_DIR/.cache" -mindepth 1 -maxdepth 1 ! -name 'ui-state.json' \
      -exec rm -rf {} +
  fi
  rm -f "$EMUACC_GENERATED_DIR/build-cache.json"
fi

npm run tauri:dev
