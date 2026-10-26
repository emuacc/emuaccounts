#!/usr/bin/env bash
# Self-test for scripts/check_appimage_libs.sh: a clean AppDir, a bundled
# libwayland-client, a bundled symlink, an AppImage extracted with its own
# --appimage-extract, several targets at once, and the usage errors.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
check="$here/../check_appimage_libs.sh"

tmp="$(mktemp -d "${TMPDIR:-/tmp}/appimage-libs-test.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }

appdir() {  # <dir> <lib>...: an AppDir with these files in usr/lib
  local dir="$1"; shift
  mkdir -p "$dir/usr/lib"
  for lib in "$@"; do : > "$dir/usr/lib/$lib"; done
}

run() {  # <expected exit> <args>...: sets $out (stdout + stderr)
  local want="$1"; shift
  set +e
  out="$("$check" "$@" 2>&1)"
  local rc=$?
  set -e
  [ "$rc" -eq "$want" ] || fail "$* exited $rc, expected $want: $out"
}

expect_in() { case "$out" in *"$1"*) ;; *) fail "expected '$1' in: $out" ;; esac; }
expect_not_in() { case "$out" in *"$1"*) fail "unexpected '$1' in: $out" ;; esac; }

# 1. The app's own libraries (WebKit, GTK) are fine, and so are the other
#    libwayland libraries the fixed bundler still ships.
appdir "$tmp/clean" libwebkit2gtk-4.1.so.0 libgtk-3.so.0 libxkbcommon.so.0 \
  libwayland-cursor.so.0 libwayland-egl.so.1 libwayland-server.so.0
run 0 "$tmp/clean"
expect_in "no host graphics libraries bundled"

# 2. A bundled libwayland-client is the TASK-134 crash.
appdir "$tmp/wayland" libwebkit2gtk-4.1.so.0 libwayland-client.so.0
run 1 "$tmp/wayland"
expect_in "must come from the host"
expect_in "usr/lib/libwayland-client.so.0"
expect_not_in "libwebkit2gtk"

# 3. A symlink counts as bundled, and so does the driver stack.
appdir "$tmp/egl" libEGL.so.1.1.0 libgbm.so.1 libdrm_amdgpu.so.1
ln -s libEGL.so.1.1.0 "$tmp/egl/usr/lib/libEGL.so.1"
run 1 "$tmp/egl"
expect_in "usr/lib/libEGL.so.1"$'\n'
expect_in "usr/lib/libgbm.so.1"
expect_in "usr/lib/libdrm_amdgpu.so.1"

# 4. An AppImage file is extracted with its own --appimage-extract.
fake_appimage() {  # <path> <lib>...: an executable that extracts these libs
  local path="$1"; shift
  {
    echo '#!/usr/bin/env bash'
    echo '[ "$1" = --appimage-extract ] || exit 9'
    echo 'mkdir -p squashfs-root/usr/lib'
    for lib in "$@"; do echo ": > squashfs-root/usr/lib/$lib"; done
  } > "$path"
  chmod +x "$path"
}
fake_appimage "$tmp/Clean.AppImage" libwebkit2gtk-4.1.so.0
run 0 "$tmp/Clean.AppImage"
expect_in "Clean.AppImage: no host graphics libraries bundled"
fake_appimage "$tmp/Bad.AppImage" libwebkit2gtk-4.1.so.0 libOpenGL.so.0
run 1 "$tmp/Bad.AppImage"
expect_in "Bad.AppImage bundles libraries that must come from the host"
expect_in "usr/lib/libOpenGL.so.0"

# 5. Every target is checked; one bad one fails the run.
run 1 "$tmp/Clean.AppImage" "$tmp/wayland"
expect_in "Clean.AppImage: no host graphics libraries bundled"
expect_in "usr/lib/libwayland-client.so.0"

# 6. Usage errors: no target, a missing path, an AppImage that won't extract.
run 2
expect_in "usage:"
run 2 "$tmp/missing.AppImage"
expect_in "usage:"
printf '#!/usr/bin/env bash\nexit 1\n' > "$tmp/Broken.AppImage"
chmod +x "$tmp/Broken.AppImage"
run 2 "$tmp/Broken.AppImage"
expect_in "--appimage-extract failed"

echo "check_appimage_libs self-test: OK"
