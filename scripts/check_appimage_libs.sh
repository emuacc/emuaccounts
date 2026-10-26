#!/usr/bin/env bash
# Fail when an AppImage bundles libraries that must come from the host's
# graphics stack.
#
# AppRun puts the AppImage's usr/lib ahead of the host's libraries, so the
# host's Mesa EGL links against a bundled libwayland-client. Ubuntu 22.04's
# 1.20 lacks symbols a current Mesa needs (Mesa 26: wl_fixes_interface), the
# Mesa EGL vendor fails to load, and WebKit aborts at startup with "Could not
# create default EGL display: EGL_BAD_PARAMETER" (TASK-134,
# tauri-apps/tauri#15976). The GL/EGL/GBM/DRM libraries belong to the host's
# driver for the same reason. The linuxdeploy that @tauri-apps/cli 2.12 pins
# leaves these out; the other libwayland libraries it still bundles load fine.
#
# Usage: scripts/check_appimage_libs.sh <file.AppImage | AppDir>...
# An AppImage is extracted with its own --appimage-extract, so it has to run
# on this machine (same CPU architecture); an AppDir is read as it is.
# Exit 0 = none bundled, 1 = at least one bundled, 2 = usage error.
set -euo pipefail

usage() { echo "usage: $0 <file.AppImage | AppDir>..." >&2; exit 2; }
[ $# -ge 1 ] || usage

tmp="$(mktemp -d "${TMPDIR:-/tmp}/appimage-libs.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

status=0
for target in "$@"; do
  if [ -d "$target" ]; then
    appdir="$target"
  elif [ -f "$target" ]; then
    image="$(cd "$(dirname "$target")" && pwd)/$(basename "$target")"
    appdir="$tmp/$(basename "$target").d/squashfs-root"
    mkdir -p "$(dirname "$appdir")"
    (cd "$(dirname "$appdir")" && "$image" --appimage-extract >/dev/null) \
      || { echo "$target: --appimage-extract failed" >&2; exit 2; }
  else
    usage
  fi

  bundled="$(find "$appdir" \( -type f -o -type l \) \( \
      -name 'libwayland-client.so*' \
      -o -name 'libEGL*.so*' -o -name 'libGL.so*' -o -name 'libGLX*.so*' \
      -o -name 'libGLdispatch.so*' -o -name 'libOpenGL.so*' -o -name 'libgbm.so*' \
      -o -name 'libdrm*.so*' \
    \) | sed "s|^$appdir/||" | sort)"
  if [ -n "$bundled" ]; then
    echo "$target bundles libraries that must come from the host:" >&2
    echo "$bundled" | sed 's/^/  /' >&2
    status=1
  else
    echo "$target: no host graphics libraries bundled"
  fi
done
exit "$status"
