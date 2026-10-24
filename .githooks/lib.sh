#!/usr/bin/env bash
# Shared helpers for .githooks/pre-commit and .githooks/pre-push.
# Sourced by the hooks (which git runs with cwd = repository root).
# Must stay compatible with macOS bash 3.2: no mapfile, no ${var,,}.

hook_fail() { # <hook> <message>
  echo "[$1] ERROR: $2" >&2
  exit 1
}

# Stop the dev processes that belong to THIS tree before Rust tests: a
# `tauri dev` started here shares src-tauri/target with cargo, so the tests
# would wait on its lock. Apps and launchers from any other tree — the primary
# checkout, another worktree, the isolated MCP target dir — are never touched:
# with one worktree per session that would stop another session's app
# (decision-004, TASK-050). The lib/integration/BDD tests never open the
# production app-data store (only the CLI bins do, and they are test = false),
# so nothing outside this tree needs stopping.
# App binaries are matched by this tree's target path; `tauri dev` launchers
# carry no path on their command line, so they are matched by their working
# directory. SKIP_KILL_DEV=1 disables this.
stop_tauri_dev() { # <hook>
  [ "${SKIP_KILL_DEV:-0}" = "1" ] && return 0
  local tree pid cwd pids=""
  tree="$(pwd -P)"
  for pid in $(pgrep -f "$tree/src-tauri/target/debug/emuaccounts" 2>/dev/null || true); do
    pids="$pids $pid"
  done
  if command -v lsof >/dev/null 2>&1; then
    for pid in $(pgrep -f 'npm run tauri:dev|node_modules/.bin/tauri dev' 2>/dev/null || true); do
      cwd="$(lsof -a -p "$pid" -d cwd -Fn 2>/dev/null | sed -n 's/^n//p' | head -1)"
      case "$cwd" in "$tree"|"$tree"/*) pids="$pids $pid" ;; esac
    done
  fi
  [ -n "${pids// /}" ] || return 0
  echo "[$1] Stopping this tree's 'tauri dev' (pids:$pids) to free target/ and app locks…"
  # shellcheck disable=SC2086
  kill $pids 2>/dev/null || true
  sleep 1
}

# ESLint complexity check on production .ts files. No-op when given none.
lint_ts_files() { # <hook> [files...]
  local hook="$1"
  shift
  [ $# -eq 0 ] && return 0
  echo "[$hook] ESLint complexity check on $# .ts file(s)"
  npx eslint --no-warn-ignored --quiet "$@" || hook_fail "$hook" "ESLint complexity check failed"
}

# Rust tests for a directory with its own Cargo.toml (the .covid layer, covid-t).
# They build into the directory's own target/ (never src-tauri/target), one per
# checkout. Sharing a target dir between two checkouts of the same crate makes
# cargo reuse the other checkout's test binary, which has that checkout's
# CARGO_MANIFEST_DIR compiled in, so this tree's files would go untested.
run_rust_tests() { # <hook> <dir>
  local hook="$1" dir="$2"
  if [ ! -f "$dir/Cargo.toml" ]; then
    echo "[$hook] Skipping Rust tests for $dir (no Cargo.toml)."
    return 0
  fi
  echo "[$hook] Rust tests in $dir"
  (cd "$dir" && env -u CARGO_TARGET_DIR cargo test --quiet) \
    || hook_fail "$hook" "Rust tests failed in $dir"
}

# Python tests for a directory, only when it carries its own .venv.
run_py_tests() { # <hook> <dir>
  local hook="$1" dir="$2"
  [ -d "$dir" ] || return 0
  if [ -d "$dir/.venv" ] && [ -x "$dir/.venv/bin/pytest" ]; then
    echo "[$hook] Python tests in $dir"
    (cd "$dir" && .venv/bin/pytest -q -W ignore::DeprecationWarning \
      -k 'not step_definitions and not placeholder') \
      || hook_fail "$hook" "Python tests failed in $dir"
  else
    echo "[$hook] Skipping Python tests for $dir (no .venv/bin/pytest)."
  fi
}
