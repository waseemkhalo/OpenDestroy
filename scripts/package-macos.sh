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
DESTROY_BUILD_ROOT="${CARGO_TARGET_DIR:-$DESTROY_ROOT/target}"
case "$DESTROY_BUILD_ROOT" in /*) ;; *) DESTROY_BUILD_ROOT="$DESTROY_ROOT/$DESTROY_BUILD_ROOT";; esac
# Cargo runs from the desktop package; resolve relative overrides here so the
# build and subsequent packaging use the same repository-relative directory.
export CARGO_TARGET_DIR="$DESTROY_BUILD_ROOT"
(cd apps/desktop && npm run tauri -- build --target "$DESTROY_TARGET" --bundles app --config "$DESTROY_CONFIG")
DESTROY_APP="$DESTROY_BUILD_ROOT/$DESTROY_TARGET/release/bundle/macos/OpenDestroy.app"
codesign --verify --deep --strict --verbose=2 "$DESTROY_APP"
codesign -dv --verbose=4 "$DESTROY_APP" 2>&1 | /usr/bin/grep -Fx "TeamIdentifier=$APPLE_TEAM_ID"
codesign -dv --verbose=4 "$DESTROY_APP" 2>&1 | /usr/bin/grep -E 'flags=.*runtime'
mkdir -p artifacts
DESTROY_VERSION="$(python3 -c 'import json; print(json.load(open("apps/desktop/package.json"))["version"])')"
DESTROY_BASE="OpenDestroy_${DESTROY_VERSION}_${DESTROY_TARGET}"
# Every app has its own submission; use a fresh notarization submission for these exact bytes.
ditto -c -k --keepParent "$DESTROY_APP" "artifacts/$DESTROY_BASE-notary.zip"
xcrun notarytool submit "artifacts/$DESTROY_BASE-notary.zip" --keychain-profile "$APPLE_NOTARY_PROFILE" --wait
xcrun stapler staple "$DESTROY_APP"
xcrun stapler validate "$DESTROY_APP"
spctl --assess --type execute --verbose=2 "$DESTROY_APP"
DESTROY_DMG_ROOT="$(mktemp -d -t destroy-dmg)"
DESTROY_MOUNT="$DESTROY_DMG_ROOT/mount"
restore_stapled_app() {
  if [ -d "$DESTROY_MOUNT" ]; then
    hdiutil detach "$DESTROY_MOUNT" -force >/dev/null 2>&1 || true
  fi
  if [ -d "$DESTROY_DMG_ROOT/OpenDestroy.app" ]; then
    rm -rf "$DESTROY_APP"
    ditto "$DESTROY_DMG_ROOT/OpenDestroy.app" "$DESTROY_APP"
  fi
  rm -rf "$DESTROY_CONFIG_DIR" "$DESTROY_DMG_ROOT"
}
# hdiutil appends a format extension when -o does not already end in one, and
# some macOS versions still append .dmg to a path that already has it.
write_image() {
  local source="$1" format="$2" dest="$3"
  shift 3
  rm -f "$dest" "${dest}.dmg"
  hdiutil convert "$source" -format "$format" -o "$dest" "$@"
  if [ ! -f "$dest" ] && [ -f "${dest}.dmg" ]; then
    mv "${dest}.dmg" "$dest"
  fi
  if [ ! -f "$dest" ]; then
    echo "hdiutil did not create $dest" >&2
    exit 1
  fi
}
trap restore_stapled_app EXIT
ditto "$DESTROY_APP" "$DESTROY_DMG_ROOT/OpenDestroy.app"
# Archive the stapled app before the DMG bundle. A DMG-only Tauri run rebuilds
# DESTROY_APP unsigned, and merging that tree back would ship a bad update.
# COPYFILE_DISABLE keeps AppleDouble files out of the archive; those extra
# files are not part of the signed bundle and break the installed update.
COPYFILE_DISABLE=1 tar -czf "artifacts/$DESTROY_BASE.app.tar.gz" -C "$DESTROY_DMG_ROOT" "OpenDestroy.app"
# The bundler accepts either key contents or a path in TAURI_SIGNING_PRIVATE_KEY;
# the standalone signer requires its explicit file option for a path.
if [ -f "$TAURI_SIGNING_PRIVATE_KEY" ]; then
  (cd apps/desktop && env -u TAURI_SIGNING_PRIVATE_KEY npm run tauri -- signer sign --private-key-path "$TAURI_SIGNING_PRIVATE_KEY" "../../artifacts/$DESTROY_BASE.app.tar.gz")
else
  (cd apps/desktop && npm run tauri -- signer sign "../../artifacts/$DESTROY_BASE.app.tar.gz")
fi
# Let Tauri create the Finder drag-install window using the checked-in DMG
# background, window size, and icon positions from tauri.conf.json.
DESTROY_DMG_DIR="$DESTROY_BUILD_ROOT/$DESTROY_TARGET/release/bundle/dmg"
DESTROY_BUNDLED_DMG="$DESTROY_DMG_DIR/OpenDestroy_${DESTROY_VERSION}_${DESTROY_TARGET%%-*}.dmg"
rm -f "$DESTROY_BUNDLED_DMG"
(cd apps/desktop && npm run tauri -- bundle --target "$DESTROY_TARGET" --bundles dmg --no-sign)
if [ ! -f "$DESTROY_BUNDLED_DMG" ]; then
  echo "Tauri did not create the configured DMG for $DESTROY_TARGET" >&2
  exit 1
fi
rm -rf "$DESTROY_APP"
ditto "$DESTROY_DMG_ROOT/OpenDestroy.app" "$DESTROY_APP"
# A DMG-only bundle run rebuilds the .app (tauri-bundler dmg::bundle_project),
# so with --no-sign the styled image holds an unsigned copy that notarization
# rejects. Swap in the notarized, stapled app, keeping the Finder layout.
DESTROY_RW_DMG="$DESTROY_DMG_ROOT/OpenDestroy-rw.dmg"
DESTROY_APP_MB="$(du -sm "$DESTROY_DMG_ROOT/OpenDestroy.app" | cut -f1)"
write_image "$DESTROY_BUNDLED_DMG" UDRW "$DESTROY_RW_DMG"
hdiutil resize -size "$((DESTROY_APP_MB * 2 + 64))m" "$DESTROY_RW_DMG"
mkdir -p "$DESTROY_MOUNT"
hdiutil attach "$DESTROY_RW_DMG" -readwrite -nobrowse -noverify -noautoopen -mountpoint "$DESTROY_MOUNT"
rm -rf "$DESTROY_MOUNT/OpenDestroy.app"
ditto "$DESTROY_DMG_ROOT/OpenDestroy.app" "$DESTROY_MOUNT/OpenDestroy.app"
codesign --verify --deep --strict "$DESTROY_MOUNT/OpenDestroy.app"
xcrun stapler validate "$DESTROY_MOUNT/OpenDestroy.app"
hdiutil detach "$DESTROY_MOUNT"
rm -f "$DESTROY_BUNDLED_DMG"
write_image "$DESTROY_RW_DMG" UDZO "artifacts/$DESTROY_BASE.dmg" -imagekey zlib-level=9
codesign --force --sign "$APPLE_SIGNING_IDENTITY" --timestamp "artifacts/$DESTROY_BASE.dmg"
xcrun notarytool submit "artifacts/$DESTROY_BASE.dmg" --keychain-profile "$APPLE_NOTARY_PROFILE" --wait
xcrun stapler staple "artifacts/$DESTROY_BASE.dmg"
xcrun stapler validate "artifacts/$DESTROY_BASE.dmg"
hdiutil verify "artifacts/$DESTROY_BASE.dmg"
(cd artifacts && shasum -a 256 "$DESTROY_BASE.dmg" "$DESTROY_BASE.app.tar.gz" > "$DESTROY_BASE.sha256")
rm "artifacts/$DESTROY_BASE-notary.zip"
