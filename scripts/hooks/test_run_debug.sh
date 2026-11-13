#!/usr/bin/env bash
# Self-test for scripts/run_debug.sh: a plain launch keeps the vault's pipeline
# cache, --fresh clears it (keeping ui-state.json), a launch with no vault finds
# current_root in this OS's app-data config.json, a per-vault wrapper passes
# --fresh through, and no launch runs a separate pipeline build first — the app
# runs the pipeline itself at startup.
#
# npm, cargo, lsof, pkill and rsync are stubs on PATH that only record their
# arguments, so nothing is built, launched, synced or killed (the real launcher
# stops any running emuaccounts).
#
# The per-vault wrappers (scripts/run_<instance>_debug.sh) are one person's
# launchers and are not published, so the wrapper section covers whichever
# exist and names none: the root a wrapper launches by default is read from
# run_debug.sh's own "not a vault" error rather than written here, and its
# override is EMUACC_<INSTANCE>_VAULT, named after the file.
set -euo pipefail
unset XDG_DATA_HOME # the Linux config.json is then under $HOME/.local/share

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

case "$(uname -s)" in
  Darwin) app_data="$tmp/home/Library/Application Support/com.emusoftware.emuaccounts" ;;
  Linux)  app_data="$tmp/home/.local/share/com.emusoftware.emuaccounts" ;;
  *)      app_data="" ;;
esac
if [ -n "$app_data" ]; then
  new_vault "$vault"
  mkdir -p "$app_data"
  printf '{"current_root": "%s"}\n' "$vault" > "$app_data/config.json"
  run run_debug.sh
  grep -qx "EMUACC_SOURCES_DIR=$vault/sources" "$tmp/out" \
    || fail "no-vault launch did not use current_root from $app_data/config.json"
  expect_launch_only "no-vault launch"
  rm -rf "$app_data"
fi

# Each per-vault wrapper must pass --fresh through to the root it launches by
# default, and its EMUACC_<INSTANCE>_VAULT must move that root, so --fresh
# clears the override's cache, not the default's. EMUACC_VAULT_DIR, which other
# tools point at whichever vault they work on, must not move it. With no
# wrapper present the glob stays literal and the loop runs nothing.
override="$tmp/home/vault-elsewhere"
for path in "$tree"/scripts/run_*_debug.sh; do
  [ -x "$path" ] || continue
  wrapper="$(basename "$path")"
  instance="${wrapper#run_}"
  instance="${instance%_debug.sh}"
  var="EMUACC_$(tr '[:lower:]' '[:upper:]' <<< "$instance")_VAULT"
  unset "$var"
  rm -rf "$override"
  default="$(wrapper_root "$wrapper")"
  [ -n "$default" ] || fail "could not read $wrapper's default vault root"
  [ "$(EMUACC_VAULT_DIR="$override" wrapper_root "$wrapper")" = "$default" ] \
    || fail "$wrapper reads EMUACC_VAULT_DIR; its override is $var"
  [ "$(export "$var=$override"; wrapper_root "$wrapper")" = "$override" ] \
    || fail "$wrapper ignores $var"

  new_vault "$default"
  run "$wrapper" --fresh
  [ ! -e "$default/generated/.cache/transforms" ] \
    || fail "$wrapper --fresh did not reach its default vault"
  expect_launch_only "$wrapper --fresh"

  new_vault "$default"
  new_vault "$override"
  export "$var=$override"
  run "$wrapper" --fresh
  unset "$var"
  [ ! -e "$override/generated/.cache/transforms" ] \
    || fail "$wrapper --fresh with $var: the override vault's cache survived"
  [ -f "$override/generated/.cache/ui-state.json" ] \
    || fail "$wrapper --fresh deleted ui-state.json in the $var vault"
  [ -d "$default/generated/.cache/transforms" ] \
    || fail "$wrapper --fresh with $var cleared the default vault instead"
  expect_launch_only "$wrapper with $var"
done

echo "run_debug self-test: OK"
