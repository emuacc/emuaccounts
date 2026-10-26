#!/usr/bin/env bash
# Fail when a macOS release bundle would not open on a user's Mac.
#
# Gatekeeper refuses a downloaded (quarantined) app whose bundle signature
# does not verify with "EmuAcc is damaged and can't be opened", and offers no
# Open Anyway. With no signing identity the Tauri bundler leaves only the Rust
# linker's ad-hoc signature on each binary: no sealed resources, Info.plist
# not bound, and `codesign --verify --deep --strict` fails with "code has no
# resources but signature indicates they must be present" (TASK-135).
# `bundle.macOS.signingIdentity: "-"` in src-tauri/tauri.conf.json makes the
# bundler ad-hoc sign the whole bundle instead.
#
# The signature does not cover file modes, so the check also fails a bundle
# whose Contents/MacOS files are not executable, like the 0.1.2
# EmuAcc.app.zip that was zipped after an upload/download-artifact round trip.
#
# Usage: scripts/check_macos_bundle.sh <App.app | file.dmg | file.zip>...
# A .dmg is attached read-only and its top-level .app checked in place. A
# .zip is expanded twice: with ditto, the way Archive Utility (Finder,
# Safari) expands it, and with unzip, which leaves the AppleDouble ._ files a
# zip made without `ditto --norsrc` carries inside the signed bundle.
# macOS only.
# Exit 0 = every bundle passes, 1 = at least one fails, 2 = usage error.
set -euo pipefail

usage() { echo "usage: $0 <App.app | file.dmg | file.zip>..." >&2; exit 2; }
[ $# -ge 1 ] || usage

tmp="$(mktemp -d "${TMPDIR:-/tmp}/macos-bundle.XXXXXX")"
mounts=()
cleanup() {
  for mnt in ${mounts[@]+"${mounts[@]}"}; do hdiutil detach -quiet "$mnt" || true; done
  rm -rf "$tmp"
}
trap cleanup EXIT

check_app() {  # <label> <App.app>: returns 1 when the bundle fails
  local label="$1" app="$2" failed=0 not_exec out
  if [ ! -d "$app/Contents/MacOS" ]; then
    echo "$label: no Contents/MacOS, not an app bundle" >&2
    return 1
  fi
  not_exec="$(find "$app/Contents/MacOS" -type f ! -perm -u+x | sed "s|^$app/||" | sort)"
  if [ -n "$not_exec" ]; then
    echo "$label: files in Contents/MacOS are not executable:" >&2
    echo "$not_exec" | sed 's/^/  /' >&2
    failed=1
  fi
  if ! out="$(codesign --verify --deep --strict "$app" 2>&1)"; then
    echo "$label: codesign --verify --deep --strict failed:" >&2
    echo "$out" | sed 's/^/  /' >&2
    failed=1
  fi
  [ "$failed" = 1 ] || echo "$label: signature valid, binaries executable"
  return "$failed"
}

check_apps_in() {  # <label> <dir> [suffix]: checks each top-level .app in <dir>
  local label="$1" dir="$2" suffix="${3:-}" app found=0 failed=0
  for app in "$dir"/*.app; do
    [ -d "$app" ] || continue
    found=1
    check_app "$label/$(basename "$app")$suffix" "$app" || failed=1
  done
  if [ "$found" = 0 ]; then
    echo "$label$suffix: no .app inside" >&2
    return 1
  fi
  return "$failed"
}

status=0
n=0
for target in "$@"; do
  n=$((n + 1))
  case "$target" in
    *.app|*.app/)
      [ -d "$target" ] || usage
      check_app "${target%/}" "${target%/}" || status=1
      ;;
    *.dmg)
      [ -f "$target" ] || usage
      mnt="$tmp/mnt$n"
      mkdir -p "$mnt"
      hdiutil attach -quiet -nobrowse -readonly -noautoopen -mountpoint "$mnt" "$target" \
        || { echo "$target: hdiutil attach failed" >&2; exit 2; }
      mounts+=("$mnt")
      check_apps_in "$target" "$mnt" || status=1
      ;;
    *.zip)
      [ -f "$target" ] || usage
      ditto -x -k "$target" "$tmp/ditto$n" || { echo "$target: ditto could not expand it" >&2; exit 2; }
      check_apps_in "$target" "$tmp/ditto$n" " (ditto -x -k)" || status=1
      unzip -q "$target" -d "$tmp/unzip$n" || { echo "$target: unzip could not expand it" >&2; exit 2; }
      check_apps_in "$target" "$tmp/unzip$n" " (unzip)" || status=1
      ;;
    *) usage ;;
  esac
done
exit "$status"
