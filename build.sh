#!/bin/bash
# Build Sleep Switch.app (universal: Apple silicon + Intel), sign it, and
# install it to ~/Applications.
#
#   ./build.sh          build and install
#   ./build.sh --open   ...and launch it
#   ./build.sh --zip    ...and write target/Sleep-Switch-<version>.zip for a release
#
# Signs with the first "Apple Development" or "Developer ID Application"
# identity in the keychain (override with SIGNING_ID=...), ad hoc otherwise.
set -euo pipefail
cd "$(dirname "$0")"

APP="target/Sleep Switch.app"
DEST="$HOME/Applications/Sleep Switch.app"
VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)

targets=(aarch64-apple-darwin x86_64-apple-darwin)
bins=()
for t in "${targets[@]}"; do
    if rustup target list --installed | grep -qx "$t"; then
        cargo build --release --target "$t"
        bins+=("target/$t/release/sleep-switch")
    else
        echo "Skipping $t (rustup target add $t to include it)"
    fi
done

rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
lipo -create "${bins[@]}" -output "$APP/Contents/MacOS/sleep-switch"
sed "s/__VERSION__/$VERSION/" Info.plist > "$APP/Contents/Info.plist"
cp assets/AppIcon.icns "$APP/Contents/Resources/"

SIGNING_ID=${SIGNING_ID:-$(security find-identity -v -p codesigning \
    | grep -Eo '"(Developer ID Application|Apple Development): [^"]+"' | head -1 | tr -d '"')}
if [[ -n "$SIGNING_ID" ]]; then
    codesign --force --sign "$SIGNING_ID" --timestamp --options runtime "$APP"
else
    echo "No signing identity found; signing ad hoc."
    codesign --force --sign - "$APP"
fi
codesign -v "$APP"

# Quit a running copy before replacing it.
pkill -x sleep-switch 2>/dev/null && sleep 1 || true
mkdir -p "$HOME/Applications"
rm -rf "$DEST"
cp -R "$APP" "$DEST"
echo "Installed: $DEST ($(lipo -archs "$APP/Contents/MacOS/sleep-switch"))"

for arg in "$@"; do
    case "$arg" in
        --open) open "$DEST" ;;
        --zip)
            zip="target/Sleep-Switch-$VERSION.zip"
            rm -f "$zip"
            ditto -c -k --keepParent "$APP" "$zip"
            echo "Wrote $zip"
            ;;
    esac
done
