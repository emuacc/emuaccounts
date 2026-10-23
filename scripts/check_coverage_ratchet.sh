#!/usr/bin/env bash
# Coverage ratchet: re-measures Rust + TS line coverage and compares it with
# the committed floor in .coverage-baseline.json.
#
#   scripts/check_coverage_ratchet.sh            check only (what the gates run)
#   scripts/check_coverage_ratchet.sh --update   raise the floor: write + stage
#
# Rules:
#   - A metric at 100% must stay at 100%.
#   - Otherwise the measurement must not fall more than RATCHET_TOLERANCE_PP
#     (default 0.25 percentage points) below the floor — the tolerance absorbs
#     measurement noise between toolchain versions and machines.
#   - Check mode NEVER writes: an improvement is reported with the command to
#     record it. Raising the floor is a deliberate commit made with --update
#     in a worktree, so no gate ever leaves a tracked file modified in
#     whatever tree happened to run it (decision-002, amended 2026-08-28).
#   - --update stores values floored to one decimal place, so the file only
#     changes when coverage has genuinely moved.
#
# SKIP_COVERAGE_RATCHET=1 skips the check entirely.
# RATCHET_MEASURED_JSON / RATCHET_BASELINE_FILE exist for the self-test.
set -euo pipefail

MODE=check
if [ "${1:-}" = "--update" ]; then
  MODE=update
elif [ -n "${1:-}" ]; then
  echo "usage: $0 [--update]" >&2
  exit 2
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

if [ "${SKIP_COVERAGE_RATCHET:-0}" = "1" ]; then
  echo "[ratchet] SKIP_COVERAGE_RATCHET=1 — skipping coverage check."
  exit 0
fi

BASELINE_FILE="${RATCHET_BASELINE_FILE:-$REPO_ROOT/.coverage-baseline.json}"

if [ -n "${RATCHET_MEASURED_JSON:-}" ]; then
  NEW_JSON="$RATCHET_MEASURED_JSON"
else
  echo "[ratchet] Measuring coverage (this takes ~30s on warm caches)..."
  NEW_JSON="$(./scripts/measure_coverage.sh)"
fi
echo "[ratchet] Measured: $NEW_JSON"

OLD_JSON="null"
[ -f "$BASELINE_FILE" ] && OLD_JSON="$(cat "$BASELINE_FILE")"

# The Python exits 0 (ok), 1 (regression) or 3 (--update wrote the file).
rc=0
RATCHET_OLD_JSON="$OLD_JSON" RATCHET_NEW_JSON="$NEW_JSON" RATCHET_MODE="$MODE" \
RATCHET_BASELINE_PATH="$BASELINE_FILE" RATCHET_TOLERANCE_PP="${RATCHET_TOLERANCE_PP:-0.25}" \
python3 - <<'PY' || rc=$?
import json, math, os, sys

old = json.loads(os.environ["RATCHET_OLD_JSON"]) or {}
new = json.loads(os.environ["RATCHET_NEW_JSON"])
mode = os.environ["RATCHET_MODE"]
path = os.environ["RATCHET_BASELINE_PATH"]
tol = float(os.environ["RATCHET_TOLERANCE_PP"])

def floor1(v):
    return math.floor(v * 10 + 1e-9) / 10

regressed, improved, unmeasured = [], [], []
proposed = dict(old)

for name in ("rust_lines", "ts_lines"):
    old_v, new_v = old.get(name), new.get(name)
    if new_v is None:
        unmeasured.append(name)
        continue
    if old_v is None:
        improved.append(f"{name}: no floor yet, measured {new_v:.2f}%")
        proposed[name] = floor1(new_v)
        continue
    if old_v >= 100.0:
        if new_v < 100.0:
            regressed.append(f"{name}: was 100.00%, now {new_v:.2f}% — must stay at 100%")
        continue
    if new_v < old_v - tol:
        regressed.append(f"{name}: dropped {old_v:.2f}% → {new_v:.2f}% (−{old_v - new_v:.2f}pp, tolerance {tol}pp)")
    elif floor1(new_v) > old_v:
        improved.append(f"{name}: {old_v:.2f}% → {new_v:.2f}% (+{new_v - old_v:.2f}pp)")
        proposed[name] = floor1(new_v)

if unmeasured:
    print(f"[ratchet] not measured (tool unavailable), skipped: {', '.join(unmeasured)}")

if regressed:
    print("[ratchet] FAIL: coverage regressed:")
    for r in regressed:
        print(f"  - {r}")
    print("\nFix by adding tests, or set SKIP_COVERAGE_RATCHET=1 to override (not recommended).")
    sys.exit(1)

if mode == "update":
    if proposed != old:
        with open(path, "w") as f:
            json.dump(proposed, f, indent=2)
            f.write("\n")
        print(f"[ratchet] floor raised and written to {path}:")
        for i in improved:
            print(f"  + {i}")
        sys.exit(3)  # tell the shell wrapper to stage it
    print("[ratchet] floor unchanged — nothing to write.")
    sys.exit(0)

if improved:
    print("[ratchet] OK — coverage improved:")
    for i in improved:
        print(f"  + {i}")
    print("  Raise the floor when you like: run `scripts/check_coverage_ratchet.sh --update` in your worktree and commit .coverage-baseline.json.")
else:
    print("[ratchet] OK — coverage held steady.")
PY
if [ "$rc" = 3 ]; then
  # --update wrote the file; stage it when it lives in this repository.
  case "$BASELINE_FILE" in
    "$REPO_ROOT"/*) git add "$BASELINE_FILE" && echo "[ratchet] $BASELINE_FILE staged — commit it." ;;
  esac
  exit 0
fi
exit "$rc"
