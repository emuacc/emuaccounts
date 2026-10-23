#!/usr/bin/env bash
# Self-test for scripts/check_coverage_ratchet.sh: drives the comparison with
# RATCHET_MEASURED_JSON and a temporary baseline file, so nothing is measured
# and nothing in the repository is touched.
set -euo pipefail
unset GIT_DIR GIT_WORK_TREE GIT_INDEX_FILE GIT_COMMON_DIR GIT_PREFIX

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ratchet="$here/../check_coverage_ratchet.sh"
tmp="$(mktemp -d "${TMPDIR:-/tmp}/ratchet-test.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT
baseline="$tmp/baseline.json"
fails=0

run() { # <measured-json> [--update] → sets $out $rc
  local measured="$1"; shift
  set +e
  out="$(RATCHET_MEASURED_JSON="$measured" RATCHET_BASELINE_FILE="$baseline" "$ratchet" "$@" 2>&1)"
  rc=$?
  set -e
}
expect_rc() { # <label> <expected rc>
  if [ "$rc" = "$2" ]; then echo "ok    $1"; else echo "FAIL  $1: expected rc $2, got $rc"; echo "$out" | sed 's/^/      /'; fails=$((fails+1)); fi
}
expect_out() { # <label> <grep pattern>
  if printf '%s' "$out" | grep -Eq -e "$2"; then echo "ok    $1"; else echo "FAIL  $1: output lacks /$2/"; echo "$out" | sed 's/^/      /'; fails=$((fails+1)); fi
}
expect_file() { # <label> <expected content or 'absent'>
  if [ "$2" = absent ]; then
    if [ ! -f "$baseline" ]; then echo "ok    $1"; else echo "FAIL  $1: baseline exists: $(cat "$baseline")"; fails=$((fails+1)); fi
  elif [ "$(cat "$baseline" 2>/dev/null)" = "$2" ]; then
    echo "ok    $1"
  else
    echo "FAIL  $1: got $(cat "$baseline" 2>/dev/null)"; fails=$((fails+1))
  fi
}

echo "--- check mode, no floor yet ---"
run '{"rust_lines": 67.04, "ts_lines": 80.19}'
expect_rc  "check without baseline passes" 0
expect_out "…and points at --update" '--update'
expect_file "…and writes nothing" absent

echo "--- update mode establishes a floored floor ---"
run '{"rust_lines": 67.04, "ts_lines": 80.19}' --update
expect_rc  "update without baseline passes" 0
expect_file "floor stored to one decimal, floored" "$(printf '{\n  "rust_lines": 67.0,\n  "ts_lines": 80.1\n}')"

echo "--- check mode against the floor ---"
run '{"rust_lines": 67.04, "ts_lines": 80.19}'
expect_rc  "same measurement holds steady" 0
expect_out "…says steady" 'held steady'
run '{"rust_lines": 66.9, "ts_lines": 80.1}'
expect_rc  "dip within 0.25pp tolerance passes" 0
run '{"rust_lines": 66.7, "ts_lines": 80.1}'
expect_rc  "drop beyond tolerance fails" 1
expect_out "…names the metric" 'rust_lines: dropped'
run '{"rust_lines": 68.3, "ts_lines": 80.1}'
expect_rc  "improvement passes" 0
expect_out "…reports it with the update command" 'coverage improved.*|--update'
expect_file "…and still writes nothing" "$(printf '{\n  "rust_lines": 67.0,\n  "ts_lines": 80.1\n}')"
run '{"rust_lines": null, "ts_lines": 80.1}'
expect_rc  "unmeasured metric is skipped" 0
expect_out "…and reported" 'not measured'
run '{"rust_lines": 67.01, "ts_lines": 80.1}'
expect_out "noise above the floor is not an 'improvement'" 'held steady'

echo "--- tolerance override ---"
set +e; out="$(RATCHET_TOLERANCE_PP=0 RATCHET_MEASURED_JSON='{"rust_lines": 66.95, "ts_lines": 80.1}' RATCHET_BASELINE_FILE="$baseline" "$ratchet" 2>&1)"; rc=$?; set -e
expect_rc  "tolerance 0 rejects any dip" 1

echo "--- update mode raises only what moved ---"
run '{"rust_lines": 68.37, "ts_lines": 80.15}' --update
expect_rc  "update with improvement passes" 0
expect_file "rust floor raised (floored), ts unchanged" "$(printf '{\n  "rust_lines": 68.3,\n  "ts_lines": 80.1\n}')"
run '{"rust_lines": 68.3, "ts_lines": 80.1}' --update
expect_rc  "update with nothing to raise passes" 0
expect_out "…and says so" 'unchanged'

echo "--- the 100% rule ---"
printf '{\n  "rust_lines": 100.0,\n  "ts_lines": 80.1\n}\n' > "$baseline"
run '{"rust_lines": 99.9, "ts_lines": 80.1}'
expect_rc  "a 100% metric must stay at 100%" 1

echo "--- skip switch ---"
set +e; out="$(SKIP_COVERAGE_RATCHET=1 RATCHET_BASELINE_FILE="$baseline" "$ratchet" 2>&1)"; rc=$?; set -e
expect_rc  "SKIP_COVERAGE_RATCHET=1 skips" 0

if [ "$fails" -eq 0 ]; then echo "coverage ratchet: all checks passed"; else echo "coverage ratchet: $fails check(s) failed" >&2; exit 1; fi
