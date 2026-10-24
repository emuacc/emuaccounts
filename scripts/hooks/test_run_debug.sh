#!/usr/bin/env bash
# Self-test for scripts/run_debug.sh: a plain launch keeps the vault's pipeline
# cache, --fresh clears it (keeping ui-state.json), a per-vault wrapper passes
# --fresh through, and no launch runs a separate pipeline build first — the app
# runs the pipeline itself at startup.
#
# npm, cargo, lsof, pkill and rsync are stubs on PATH that only record their
# arguments, so nothing is built, launched, synced or killed (the real launcher
# stops any running emuaccounts).
#
# The per-vault wrappers (scripts/run_*_debug.sh) are one person's launchers
# and are not published, so the wrapper section covers whichever exist and
# names none: the root a wrapper launches by default is read from
# run_debug.sh's own "not a vault" error rather than written here.
set -euo pipefail
unset EMUACC_VAULT_DIR # a wrapper may honour it; it is set only where tested

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tree="$(cd "$here/../.." && pwd -P)"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/run-debug-test.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$tmp/bin"
for cmd in npm cargo lsof pkill rsync; do
  printf '#!/bin/sh\necho "%s $*" >> "%s/calls"\n' "$cmd" "$tmp" > "$tmp/bin/$cmd"
  chmod +x "$tmp/bin/$cmd"
done

fail() { echo "FAIL: $*" >&2; exit 1; }

new_vault() { # <root> — a vault whose generated/ holds a warm pipeline cache
  rm -rf "$1" "$tmp/calls"
  mkdir -p "$1/sources" "$1/generated/.cache/transforms"
  echo '{}' > "$1/generated/.cache/ui-state.json"
  echo '{}' > "$1/generated/build-cache.json"
}

run() { # <script> [args...] — expects success; the caller's environment is kept
  local script="$1"; shift
  HOME="$tmp/home" PATH="$tmp/bin:$PATH" "$tree/scripts/$script" "$@" > "$tmp/out" 2>&1 \
    || { cat "$tmp/out" >&2; fail "$script $* exited non-zero"; }
}

expect_launch_only() { # the app is started and nothing is built beforehand
  grep -qx 'npm run tauri:dev' "$tmp/calls" || fail "$1: npm run tauri:dev was not run"
  if grep -q '^cargo' "$tmp/calls"; then
    fail "$1: ran cargo before launching: $(grep '^cargo' "$tmp/calls")"
  fi
}

wrapper_root() { # <wrapper> — the root it would launch now, which must not exist yet
  local out
  out="$(HOME="$tmp/home" PATH="$tmp/bin:$PATH" "$tree/scripts/$1" 2>&1 || true)"
  sed -n 's#^error: \(.*\) is not a vault (no sources/ directory)$#\1#p' <<< "$out"
}

vault="$tmp/home/vault"
cache="$vault/generated/.cache"

new_vault "$vault"
run run_debug.sh "$vault"
[ -d "$cache/transforms" ] || fail "plain launch cleared .cache/"
[ -f "$vault/generated/build-cache.json" ] || fail "plain launch deleted build-cache.json"
expect_launch_only "plain launch"

new_vault "$vault"
run run_debug.sh --fresh "$vault"
[ ! -e "$cache/transforms" ] || fail "--fresh kept .cache/transforms"
[ ! -e "$vault/generated/build-cache.json" ] || fail "--fresh kept build-cache.json"
[ -f "$cache/ui-state.json" ] || fail "--fresh deleted ui-state.json"
expect_launch_only "--fresh"

for bad in "--frsh $vault" "$vault $vault"; do
  # shellcheck disable=SC2086 # split into separate arguments on purpose
  if HOME="$tmp/home" PATH="$tmp/bin:$PATH" "$tree/scripts/run_debug.sh" $bad > "$tmp/out" 2>&1; then
    fail "run_debug.sh $bad succeeded; expected a usage error"
  fi
done

# Each per-vault wrapper must pass --fresh through to the root it launches by
# default; one that honours EMUACC_VAULT_DIR must clear the override's cache,
# not its default's. With no wrapper present the glob stays literal and the
# loop runs nothing.
override="$tmp/home/vault-elsewhere"
for path in "$tree"/scripts/run_*_debug.sh; do
  [ -x "$path" ] || continue
  wrapper="$(basename "$path")"
  default="$(wrapper_root "$wrapper")"
  [ -n "$default" ] || fail "could not read $wrapper's default vault root"
  new_vault "$default"
  run "$wrapper" --fresh
  [ ! -e "$default/generated/.cache/transforms" ] \
    || fail "$wrapper --fresh did not reach its default vault"
  expect_launch_only "$wrapper --fresh"

  rm -rf "$override"
  [ "$(EMUACC_VAULT_DIR="$override" wrapper_root "$wrapper")" = "$override" ] || continue
  new_vault "$default"
  new_vault "$override"
  EMUACC_VAULT_DIR="$override" run "$wrapper" --fresh
  [ ! -e "$override/generated/.cache/transforms" ] \
    || fail "$wrapper ignored EMUACC_VAULT_DIR: the override vault's cache survived --fresh"
  [ -f "$override/generated/.cache/ui-state.json" ] \
    || fail "$wrapper --fresh deleted ui-state.json in the EMUACC_VAULT_DIR vault"
  [ -d "$default/generated/.cache/transforms" ] \
    || fail "$wrapper ignored EMUACC_VAULT_DIR: the default vault was cleared instead"
  expect_launch_only "$wrapper with EMUACC_VAULT_DIR"
done

echo "run_debug self-test: OK"
