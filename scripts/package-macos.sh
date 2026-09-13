#!/bin/bash
set -euo pipefail
# Called only for a production release after its signing inputs are provided.
DESTROY_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$DESTROY_ROOT"
DESTROY_TARGET="${1:?Usage: scripts/package-macos.sh aarch64-apple-darwin|x86_64-apple-darwin}"
case "$DESTROY_TARGET" in aarch64-apple-darwin|x86_64-apple-darwin) ;; *) exit 2;; esac
: "${APPLE_TEAM_ID:?Set the owner Apple Developer team ID}"
: "${APPLE_NOTARY_PROFILE:?Store notarytool credentials in this Keychain profile first}"
DESTROY_CONFIG_DIR="$(mktemp -d -t destroy-dictation-release)"
DESTROY_CONFIG="$DESTROY_CONFIG_DIR/config.json"
trap 'rm -rf "$DESTROY_CONFIG_DIR"' EXIT
python3 scripts/release-config.py "$DESTROY_CONFIG"
(cd apps/desktop && npm run tauri -- build --target "$DESTROY_TARGET" --bundles app --config "$DESTROY_CONFIG")
DESTROY_BUILD_ROOT="${CARGO_TARGET_DIR:-$DESTROY_ROOT/target}"
case "$DESTROY_BUILD_ROOT" in /*) ;; *) DESTROY_BUILD_ROOT="$DESTROY_ROOT/$DESTROY_BUILD_ROOT";; esac
DESTROY_APP="$DESTROY_BUILD_ROOT/$DESTROY_TARGET/release/bundle/macos/Destroy Dictation.app"
codesign --verify --deep --strict --verbose=2 "$DESTROY_APP"
codesign -dv --verbose=4 "$DESTROY_APP" 2>&1 | /usr/bin/grep -F "TeamIdentifier=$APPLE_TEAM_ID"
codesign -dv --verbose=4 "$DESTROY_APP" 2>&1 | /usr/bin/grep -E 'flags=.*runtime'
mkdir -p artifacts
DESTROY_VERSION="$(python3 -c 'import json; print(json.load(open("apps/desktop/package.json"))["version"])')"
DESTROY_BASE="DestroyDictation_${DESTROY_VERSION}_${DESTROY_TARGET}"
# Every app has its own submission; use a fresh notarization submission for these exact bytes.
ditto -c -k --keepParent "$DESTROY_APP" "artifacts/$DESTROY_BASE-notary.zip"
xcrun notarytool submit "artifacts/$DESTROY_BASE-notary.zip" --keychain-profile "$APPLE_NOTARY_PROFILE" --wait
xcrun stapler staple "$DESTROY_APP"
xcrun stapler validate "$DESTROY_APP"
spctl --assess --type execute --verbose=2 "$DESTROY_APP"
DESTROY_DMG_ROOT="$(mktemp -d -t destroy-dmg)"
trap 'rm -rf "$DESTROY_CONFIG_DIR"; rm -rf "$DESTROY_DMG_ROOT"' EXIT
ditto "$DESTROY_APP" "$DESTROY_DMG_ROOT/Destroy Dictation.app"
ln -s /Applications "$DESTROY_DMG_ROOT/Applications"
hdiutil create -volname "Destroy Dictation" -srcfolder "$DESTROY_DMG_ROOT" -ov -format UDZO "artifacts/$DESTROY_BASE.dmg"
codesign --force --sign "$APPLE_SIGNING_IDENTITY" --timestamp "artifacts/$DESTROY_BASE.dmg"
xcrun notarytool submit "artifacts/$DESTROY_BASE.dmg" --keychain-profile "$APPLE_NOTARY_PROFILE" --wait
xcrun stapler staple "artifacts/$DESTROY_BASE.dmg"
xcrun stapler validate "artifacts/$DESTROY_BASE.dmg"
hdiutil verify "artifacts/$DESTROY_BASE.dmg"
# Package the stapled app, then sign this exact updater payload with the NEW updater key.
tar -czf "artifacts/$DESTROY_BASE.app.tar.gz" -C "$(dirname "$DESTROY_APP")" "Destroy Dictation.app"
(cd apps/desktop && npm run tauri -- signer sign "../../artifacts/$DESTROY_BASE.app.tar.gz")
(cd artifacts && shasum -a 256 "$DESTROY_BASE.dmg" "$DESTROY_BASE.app.tar.gz" > "$DESTROY_BASE.sha256")
rm "artifacts/$DESTROY_BASE-notary.zip"
