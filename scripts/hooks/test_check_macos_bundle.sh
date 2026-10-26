#!/usr/bin/env bash
# Self-test for scripts/check_macos_bundle.sh: a bundle the bundler sealed,
# the linker-signed-only bundle 0.1.2 shipped, binaries that lost their exec
# bits, the same bundles inside a .dmg and a .zip, a zip carrying AppleDouble
# files, several targets at once, and the usage errors. Needs codesign,
# hdiutil, ditto and cc, so it skips off macOS.
set -euo pipefail

if [ "$(uname -s)" != Darwin ] || ! command -v cc >/dev/null; then
  echo "check_macos_bundle self-test: skipped (needs macOS with cc)"
  exit 0
fi

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
check="$here/../check_macos_bundle.sh"

tmp="$(mktemp -d "${TMPDIR:-/tmp}/macos-bundle-test.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

fail() { echo "FAIL: $*" >&2; exit 1; }

printf 'int main(void) { return 0; }\n' > "$tmp/main.c"
cc -o "$tmp/main" "$tmp/main.c"

# <dir>/Fake.app with a main binary and a sidecar, each carrying only the
# linker's ad-hoc signature, as the bundler leaves them without an identity.
linker_signed_app() {
  local app="$1/Fake.app"
  mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
  cat > "$app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleExecutable</key><string>fake</string>
<key>CFBundleIdentifier</key><string>com.example.fake</string>
<key>CFBundlePackageType</key><string>APPL</string>
</dict></plist>
PLIST
  cp "$tmp/main" "$app/Contents/MacOS/fake"
  cp "$tmp/main" "$app/Contents/MacOS/fake-cli"
  echo data > "$app/Contents/Resources/data.txt"
}
sealed_app() {  # <dir>: the bundle as signingIdentity "-" leaves it
  linker_signed_app "$1"
  codesign --force --deep -s - "$1/Fake.app" 2>/dev/null
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

# 1. A bundle the bundler sealed passes.
sealed_app "$tmp/sealed"
run 0 "$tmp/sealed/Fake.app"
expect_in "Fake.app: signature valid, binaries executable"

# 2. Linker signatures alone are the 0.1.2 "damaged" app.
linker_signed_app "$tmp/linker"
run 1 "$tmp/linker/Fake.app"
expect_in "codesign --verify --deep --strict failed"
expect_in "code has no resources but signature indicates they must be present"

# 3. Lost exec bits fail even though the signature still verifies.
sealed_app "$tmp/modes"
chmod 644 "$tmp/modes/Fake.app/Contents/MacOS/fake-cli"
run 1 "$tmp/modes/Fake.app"
expect_in "not executable"
expect_in "Contents/MacOS/fake-cli"
expect_not_in "Contents/MacOS/fake"$'\n'
expect_not_in "codesign --verify"

# 4. A .dmg is attached and the .app inside it checked.
hdiutil create -quiet -fs HFS+ -volname Fake -srcfolder "$tmp/sealed" "$tmp/Sealed.dmg"
run 0 "$tmp/Sealed.dmg"
expect_in "Sealed.dmg/Fake.app: signature valid"
hdiutil create -quiet -fs HFS+ -volname Fake -srcfolder "$tmp/linker" "$tmp/Linker.dmg"
run 1 "$tmp/Linker.dmg"
expect_in "Linker.dmg/Fake.app: codesign --verify --deep --strict failed"

# 5. A zip made like the release job's (ditto --norsrc) expands to a valid
#    bundle with Archive Utility and with unzip; a zip of the copy that lost
#    its exec bits does not.
ditto -c -k --norsrc --keepParent "$tmp/sealed/Fake.app" "$tmp/Sealed.app.zip"
run 0 "$tmp/Sealed.app.zip"
expect_in "Sealed.app.zip/Fake.app (ditto -x -k): signature valid"
expect_in "Sealed.app.zip/Fake.app (unzip): signature valid"
ditto -c -k --norsrc --keepParent "$tmp/modes/Fake.app" "$tmp/Modes.app.zip"
run 1 "$tmp/Modes.app.zip"
expect_in "Fake.app (ditto -x -k): files in Contents/MacOS are not executable"
expect_in "Contents/MacOS/fake-cli"

# 6. Without --norsrc an extended attribute travels as an AppleDouble ._
#    file, which unzip leaves inside the signed bundle.
sealed_app "$tmp/xattr"
xattr -w com.example.test 1 "$tmp/xattr/Fake.app/Contents/MacOS/fake"
ditto -c -k --keepParent "$tmp/xattr/Fake.app" "$tmp/Xattr.app.zip"
run 1 "$tmp/Xattr.app.zip"
expect_in "Xattr.app.zip/Fake.app (ditto -x -k): signature valid"
expect_in "Xattr.app.zip/Fake.app (unzip): files in Contents/MacOS are not executable"
expect_in "Contents/MacOS/._fake"

# 7. Every target is checked; one bad one fails the run.
run 1 "$tmp/Sealed.dmg" "$tmp/linker/Fake.app"
expect_in "Sealed.dmg/Fake.app: signature valid"
expect_in "code has no resources"

# 8. A .dmg or .zip with no .app in it fails.
mkdir -p "$tmp/empty"
echo readme > "$tmp/empty/README.txt"
ditto -c -k "$tmp/empty" "$tmp/Empty.zip"
run 1 "$tmp/Empty.zip"
expect_in "Empty.zip (ditto -x -k): no .app inside"

# 9. Usage errors: no target, a missing path, an unknown kind of file, a
#    .dmg or .zip that cannot be opened.
run 2
expect_in "usage:"
run 2 "$tmp/missing.dmg"
expect_in "usage:"
run 2 "$tmp/main.c"
expect_in "usage:"
echo "not a disk image" > "$tmp/Broken.dmg"
run 2 "$tmp/Broken.dmg"
expect_in "hdiutil attach failed"
echo "not a zip" > "$tmp/Broken.zip"
run 2 "$tmp/Broken.zip"
expect_in "ditto could not expand it"

echo "check_macos_bundle self-test: OK"
