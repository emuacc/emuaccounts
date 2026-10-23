#!/usr/bin/env bash
# Self-test for the scoping in .githooks/pre-push: a push whose commits touch
# only Backlog.md bookkeeping (.backlog/) skips the gate, a push carrying
# anything else runs it, and a push that only deletes refs still skips it.
# Runs the real hook against a throwaway repo with a bare "origin". The gate's
# scripts do not exist in that repo, so "the gate ran" shows as the push being
# rejected at its first step (unit tests).
set -euo pipefail

# When run from a git hook, git exports GIT_DIR / GIT_INDEX_FILE / GIT_WORK_TREE
# for the repository being committed. Every git call below must target the
# throwaway repos, never that one — drop the inherited environment first.
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR GIT_PREFIX GIT_OBJECT_DIRECTORY GIT_NAMESPACE
unset SKIP_KILL_DEV

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
hooks="$(cd "$here/../../.githooks" && pwd -P)"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/pre-push-test.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

# Only the pre-push hook is wired (plus the lib it sources): pre-commit would
# type-check the throwaway repo's .ts file and fail every commit here.
mkdir -p "$tmp/hooks"
ln -s "$hooks/pre-push" "$tmp/hooks/pre-push"
ln -s "$hooks/lib.sh" "$tmp/hooks/lib.sh"

repo="$tmp/repo"
git init -q -b main "$repo"
git init -q --bare "$tmp/origin.git"
g() { git -C "$repo" -c user.email=t@t -c user.name=t "$@"; }
commit() { # <path> <message>: write the file and commit it
  mkdir -p "$(dirname "$repo/$1")"
  printf '%s\n' "$2" > "$repo/$1"
  g add -- "$1"
  g commit -q -m "$2"
}
commit src/a.ts "init"
g remote add origin "$tmp/origin.git"
g push -q origin main                  # seeds origin/main before the hook is wired
g config core.hooksPath "$tmp/hooks"

fails=0
has() { [ -n "$(printf '%s\n' "$1" | grep -F -- "$2" || true)" ]; }
push() { # <label> <skip|gate|deletions> <git push args...>
  local label="$1" expect="$2" out status
  shift 2
  if out="$(g push "$@" 2>&1)"; then status=0; else status=$?; fi
  case "$expect" in
    skip)      [ "$status" -eq 0 ] && has "$out" 'Only Backlog.md bookkeeping' && { echo "ok    $label"; return; } ;;
    deletions) [ "$status" -eq 0 ] && has "$out" 'Only ref deletions' && { echo "ok    $label"; return; } ;;
    gate)      [ "$status" -ne 0 ] && has "$out" 'Unit tests' && { echo "ok    $label"; return; } ;;
  esac
  echo "FAIL  $label (expected $expect, exit $status):"
  printf '%s\n' "$out" | sed 's/^/      /'
  fails=$((fails + 1))
}

commit ".backlog/tasks/task-1 - Add a thing.md" "Create task TASK-1"
push "backlog-only push to main skips the gate" skip origin main

g switch -q -c chore/backlog/more
commit ".backlog/tasks/task-2 - Another thing.md" "Create task TASK-2"
push "backlog-only push of a new branch skips the gate" skip -u origin chore/backlog/more

commit src/b.ts "feat: code next to bookkeeping"
push "a code commit behind bookkeeping runs the gate" gate origin chore/backlog/more

push "deleting a ref skips the gate" deletions origin --delete chore/backlog/more

if [ "$fails" -ne 0 ]; then
  echo "pre-push self-test: $fails check(s) failed" >&2
  exit 1
fi
echo "pre-push self-test: all checks passed"
