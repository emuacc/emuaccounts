#!/usr/bin/env bash
# Self-test for scripts/release_notes.py: a version's section with and
# without --out, a version with no section, an empty section, a missing
# changelog.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
notes="$here/../release_notes.py"

tmp="$(mktemp -d "${TMPDIR:-/tmp}/release-notes-test.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }

cat > "$tmp/CHANGELOG.md" <<'EOF'
# Changelog

Words about the file that belong to no release.

## 0.1.10 - 2026-12-01

Later.

## 0.1.1 - 2026-10-23

### New

- A feature. (TASK-1)

### Breaking changes

None.

## [0.1.0] - 2026-02-09

## 0.0.9 - 2026-01-01

The first build.
EOF
expected="$(printf '### New\n\n- A feature. (TASK-1)\n\n### Breaking changes\n\nNone.')"

# 1. The section body: heading left out, subheadings kept, ends at the next
#    release heading; 0.1.1 does not match 0.1.10.
out="$(python3 "$notes" 0.1.1 --changelog "$tmp/CHANGELOG.md")" || fail "0.1.1 exited $?"
[ "$out" = "$expected" ] || fail "0.1.1 body unexpected: $out"

# 2. --out writes the same text; a leading v is dropped.
python3 "$notes" v0.1.1 --changelog "$tmp/CHANGELOG.md" --out "$tmp/notes.md" || fail "--out exited $?"
[ "$(cat "$tmp/notes.md")" = "$expected" ] || fail "--out content unexpected: $(cat "$tmp/notes.md")"

expect_refused() {  # <version> <message part>: exit 3, the message, nothing written
  set +e
  out="$(python3 "$notes" "$1" --changelog "$tmp/CHANGELOG.md" --out "$tmp/refused.md" 2>&1)"
  rc=$?
  set -e
  [ "$rc" -eq 3 ] || fail "$1 exited $rc, expected 3: $out"
  case "$out" in *"$2"*) ;; *) fail "$1 message unexpected: $out" ;; esac
  [ ! -e "$tmp/refused.md" ] || fail "$1 wrote --out although refused"
}

# 3. No section for the version.
expect_refused 0.2.0 "no section for 0.2.0"

# 4. A heading with nothing under it is no release notes either.
expect_refused 0.1.0 "no section for 0.1.0"

# 5. No changelog at all.
set +e
out="$(python3 "$notes" 0.1.1 --changelog "$tmp/missing.md" 2>&1)"
rc=$?
set -e
[ "$rc" -eq 3 ] || fail "missing changelog exited $rc, expected 3: $out"
case "$out" in *"does not exist"*) ;; *) fail "missing changelog message unexpected: $out" ;; esac

echo "release_notes self-test: OK"
