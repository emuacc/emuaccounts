#!/usr/bin/env bash
# Self-test for stop_tauri_dev in .githooks/lib.sh: fake dev processes that
# belong to this tree must be stopped, fakes from a foreign tree must survive.
#
# Fakes are `sleep` processes with a rewritten argv[0]: the app binaries carry
# their fake path (matched by path), the launchers look like `npm run
# tauri:dev` (matched by cwd). Copies of /bin/sleep cannot be used — macOS
# kills platform binaries executed from anywhere but their own location.
set -euo pipefail
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR GIT_PREFIX
# The test owns this switch: an inherited SKIP_KILL_DEV=1 (e.g. from the
# environment of the git command that ran the hook) would make every check
# below fail for the wrong reason.
unset SKIP_KILL_DEV

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tree="$(cd "$here/../.." && pwd -P)"
# shellcheck source=../../.githooks/lib.sh
. "$tree/.githooks/lib.sh"

foreign="$(mktemp -d "${TMPDIR:-/tmp}/stop-tauri-dev-test.XXXXXX")"
foreign="$(cd "$foreign" && pwd -P)"
mkdir -p "$foreign/src-tauri/target/debug" "$tree/src-tauri/target/debug"
# This tree's fake keeps the real binary name as a prefix so the path matches
# exactly what the hook looks for, without touching a real build output.
ours_bin="$tree/src-tauri/target/debug/emuacc-covid.stoptest"
foreign_bin="$foreign/src-tauri/target/debug/emuacc-covid"
fake_app() { # <path>  — a script whose process shows its own path in argv[0]
  printf '#!/bin/bash\nexec -a "$0" /bin/sleep "$1"\n' > "$1"
  chmod +x "$1"
}
fake_app "$ours_bin"
fake_app "$foreign_bin"

pids=""
cleanup() {
  # shellcheck disable=SC2086
  [ -n "$pids" ] && kill $pids 2>/dev/null || true
  rm -f "$ours_bin"
  rm -rf "$foreign"
}
trap cleanup EXIT

"$foreign_bin" 300 & foreign_app=$!
"$ours_bin" 300 & our_app=$!
(cd "$foreign" && exec -a "npm run tauri:dev" /bin/sleep 300) & foreign_launcher=$!
(cd "$tree" && exec -a "npm run tauri:dev" /bin/sleep 300) & our_launcher=$!
pids="$foreign_app $our_app $foreign_launcher $our_launcher"
disown -a # keep bash from reporting the fakes as they are stopped
sleep 1

(cd "$tree" && stop_tauri_dev test)

alive() { kill -0 "$1" 2>/dev/null; }
fails=0
check() { # <label> <pid> <expect: alive|stopped>
  if [ "$3" = alive ]; then
    if alive "$2"; then echo "ok    $1 (still running)"; else echo "FAIL  $1 was stopped"; fails=$((fails+1)); fi
  else
    if alive "$2"; then echo "FAIL  $1 is still running"; fails=$((fails+1)); else echo "ok    $1 (stopped)"; fi
  fi
}
check "foreign tree's app binary"        "$foreign_app"      alive
check "foreign tree's tauri dev launcher" "$foreign_launcher" alive
check "this tree's app binary"           "$our_app"          stopped
check "this tree's tauri dev launcher"   "$our_launcher"     stopped

echo "--- SKIP_KILL_DEV=1 ---"
if [ "$fails" -eq 0 ]; then
  "$ours_bin" 300 & extra=$!
  disown
  pids="$pids $extra"
  sleep 1
  (cd "$tree" && SKIP_KILL_DEV=1 stop_tauri_dev test)
  check "this tree's app survives with SKIP_KILL_DEV=1" "$extra" alive
fi

if [ "$fails" -eq 0 ]; then echo "stop_tauri_dev: all checks passed"; else echo "stop_tauri_dev: $fails check(s) failed" >&2; exit 1; fi
