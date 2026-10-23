#!/usr/bin/env bash
# Start the LM Studio local server, then run the normal dev loop
# (`npm run tauri:dev`).
#
# The app's AI features talk to an OpenAI-compatible endpoint configured in
# Global Settings — LM Studio at http://localhost:1234/v1 by default. This
# wrapper makes sure the model server is up before the app starts, so AI
# suggestions work on first click instead of erroring with "Could not reach
# the AI server".

set -euo pipefail
cd "$(dirname "$0")/.."

MODELS_URL="http://localhost:1234/v1/models"
# `lms` is installed with LM Studio; GUI installs don't always put it on PATH.
LMS_BIN="$(command -v lms || echo "$HOME/.lmstudio/bin/lms")"

if curl -sf --max-time 2 "$MODELS_URL" > /dev/null; then
  echo "[dev+ai] LM Studio server already running on localhost:1234"
else
  if [ ! -x "$LMS_BIN" ]; then
    echo "[dev+ai] ERROR: lms CLI not found (looked on PATH and at ~/.lmstudio/bin/lms)." >&2
    echo "[dev+ai] Install LM Studio, or start its server manually, then re-run." >&2
    exit 1
  fi
  echo "[dev+ai] Starting LM Studio server…"
  "$LMS_BIN" server start
  for _ in $(seq 1 30); do
    curl -sf --max-time 2 "$MODELS_URL" > /dev/null && break
    sleep 1
  done
  if ! curl -sf --max-time 2 "$MODELS_URL" > /dev/null; then
    echo "[dev+ai] ERROR: LM Studio server did not come up on localhost:1234 within 30s" >&2
    exit 1
  fi
  echo "[dev+ai] LM Studio server is up."
fi

# Warn (don't block) when no model is available — the app's AI calls will fail
# until one is downloaded/loaded and its id is set in Global Settings.
if ! curl -sf --max-time 2 "$MODELS_URL" | grep -q '"id"'; then
  echo "[dev+ai] WARNING: LM Studio reports no models. Load one (lms ls; lms load <model>)" >&2
  echo "[dev+ai]          and set its id in Global Settings → AI assistant." >&2
fi

# Prebuild the Rust side from THIS bash process, not under npm: cargo spawned
# with node in its ancestry can freeze mid-codegen (Node 25 starves rustc's
# jobserver — see MEMORY.md). With the build warm, the npm script's own
# prebuild is an instant no-op.
echo "[dev+ai] Prebuilding Rust (dodges the node jobserver freeze)…"
cargo build --manifest-path src-tauri/Cargo.toml --no-default-features

exec npm run tauri:dev
