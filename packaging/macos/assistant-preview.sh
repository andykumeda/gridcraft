#!/usr/bin/env bash
# Package an already-built assistant preview; never builds, installs or publishes it.
# Usage: assistant-preview.sh --binary PATH --version VERSION --build-sha COMMIT
#        [--arch aarch64|x86_64|universal] [--output-dir DIR]
# Outputs a ZIP and drag-to-Applications DMG, each with a checksum.
# MACOS_SIGN_IDENTITY defaults to ad-hoc (-). This script does not notarize.
set -euo pipefail

BINARY=""
PREVIEW_VERSION=""
BUILD_SHA=""
ARCH=aarch64
while [[ $# -gt 0 ]]; do
  case "$1" in
    --binary|--version|--build-sha|--arch|--output-dir)
      [[ $# -ge 2 ]] || { echo "error: $1 needs a value" >&2; exit 2; }
      case "$1" in
        --binary) BINARY="$2" ;;
        --version) PREVIEW_VERSION="$2" ;;
        --build-sha) BUILD_SHA="$2" ;;
        --arch) ARCH="$2" ;;
        --output-dir) DIST="$2" ;;
      esac
      shift 2 ;;
    -h|--help) sed -n '2,5p' "$0"; exit 0 ;;
    *) echo "unknown argument: $1" >&2; exit 2 ;;
  esac
done
[[ -f "$BINARY" && -x "$BINARY" ]] || { echo "error: supply an executable --binary" >&2; exit 2; }
[[ "$PREVIEW_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-+][A-Za-z0-9.-]+)?$ ]] || {
  echo "error: supply the binary's --version (for example 0.11.0)" >&2; exit 2;
}
[[ "$BUILD_SHA" =~ ^[0-9a-f]{40}$ ]] || { echo "error: --build-sha must be the binary's full source commit" >&2; exit 2; }
BINARY="$(cd "$(dirname "$BINARY")" && pwd)/$(basename "$BINARY")"
if ! BINARY_VERSION="$("$BINARY" --version)"; then
  echo "error: could not run the supplied binary's --version on this Mac" >&2; exit 2;
fi
if [[ "$BINARY_VERSION" != "GridCraft $PREVIEW_VERSION (${BUILD_SHA:0:8})" ]]; then
  echo "error: supplied version/commit do not match binary --version: $BINARY_VERSION" >&2; exit 2;
fi
case "$("$BINARY" --help)" in
  *--disable-updates*) ;;
  *) echo "error: binary must support --disable-updates for preview isolation" >&2; exit 2 ;;
esac
export GRIDCRAFT_VERSION="$PREVIEW_VERSION" GRIDCRAFT_BUILD_SHA="$BUILD_SHA"
DIST="${DIST:-$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)/dist/assistant-preview/${BUILD_SHA:0:8}}"
# shellcheck source=../env.sh
. "$(dirname "${BASH_SOURCE[0]}")/../env.sh"
HERE="$ROOT/packaging/macos"

ACTUAL_ARCH="$(lipo -archs "$BINARY")"
case "$ARCH:$ACTUAL_ARCH" in
  aarch64:arm64) MACHINE="Apple silicon (arm64); Intel Macs are not supported by this archive" ;;
  x86_64:x86_64) MACHINE="Intel (x86_64)" ;;
  'universal:arm64 x86_64'|'universal:x86_64 arm64') MACHINE="Apple silicon and Intel (universal)" ;;
  *) echo "error: requested $ARCH but binary contains $ACTUAL_ARCH" >&2; exit 2 ;;
esac
# Use the binary's deployment target, including the highest minimum of universal slices.
MIN_MACOS="$(xcrun vtool -show-build "$BINARY" | awk '
  $1 == "minos" {
    split($2, v, "."); n = v[1] * 1000000 + v[2] * 1000 + v[3];
    if (n > highest) { highest = n; version = $2 }
  }
  END { print version }
')"
[[ "$MIN_MACOS" =~ ^[0-9]+\.[0-9]+(\.[0-9]+)?$ ]] || {
  echo "error: could not determine the binary's minimum macOS version" >&2; exit 2;
}

NAME="gridcraft-assistant-preview-$VERSION-${BUILD_SHA:0:8}-macos-$ARCH"
ZIP="$DIST/$NAME.zip"
CHECKSUM="$ZIP.sha256"
README="$DIST/$NAME-INSTALL.md"
DMG="$DIST/$NAME.dmg"
DMG_CHECKSUM="$DMG.sha256"
for output in "$ZIP" "$CHECKSUM" "$README" "$DMG" "$DMG_CHECKSUM"; do
  [[ ! -e "$output" ]] || { echo "error: output already exists: $output" >&2; exit 2; }
done
WORK="$(mktemp -d "${TMPDIR:-/tmp}/gridcraft-assistant-package.XXXXXX")"
DMG_MOUNT=""
cleanup() {
  if [[ -n "$DMG_MOUNT" ]] && ! hdiutil detach "$DMG_MOUNT" >/dev/null 2>&1; then
    echo "error: could not detach $DMG_MOUNT; retained $WORK for cleanup" >&2
    return
  fi
  rm -rf "$WORK"
}
trap cleanup EXIT
PACKAGE="$WORK/$NAME"
APP="$PACKAGE/GridCraft Assistant Preview.app"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"
cp "$BINARY" "$APP/Contents/MacOS/GridCraft-native"
cp "$ROOT/assets/app-icon/gridcraft.icns" "$APP/Contents/Resources/GridCraft.icns"
copy_docs "$APP/Contents/Resources"

cat >"$APP/Contents/MacOS/GridCraft" <<'LAUNCHER'
#!/bin/sh
set -eu
if [ -z "${GRIDCRAFT_PROFILE_DIR:-}" ]; then
  : "${HOME:?A home directory is required for the assistant preview profile}"
  GRIDCRAFT_PROFILE_DIR="$HOME/Library/Application Support/GridCraft Assistant Preview"
fi
export GRIDCRAFT_PROFILE_DIR
# Finder has a minimal PATH. Preserve explicit choices and add standard Codex install locations.
PATH="${PATH:-/usr/bin:/bin:/usr/sbin:/sbin}:/opt/homebrew/bin:/usr/local/bin${HOME:+:$HOME/.local/bin}"
export PATH
PREVIEW_BIN_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
exec "$PREVIEW_BIN_DIR/GridCraft-native" --disable-updates "$@"
LAUNCHER
chmod 755 "$APP/Contents/MacOS/GridCraft" "$APP/Contents/MacOS/GridCraft-native"

SHORT_VERSION="${VERSION%%[-+]*}"
sed -e "s/@VERSION@/$VERSION/g" -e "s/@SHORT_VERSION@/$SHORT_VERSION/g" \
  -e "s/@BUILD_SHA@/$BUILD_SHA/g" "$HERE/Info.plist.in" >"$APP/Contents/Info.plist"
for key in CFBundleDisplayName CFBundleName; do
  plutil -replace "$key" -string 'GridCraft Assistant Preview' "$APP/Contents/Info.plist"
done
plutil -replace CFBundleIdentifier -string ai.storyteller.gridcraft.assistant-preview "$APP/Contents/Info.plist"
plutil -replace LSMinimumSystemVersion -string "$MIN_MACOS" "$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist"
printf 'APPL????' >"$APP/Contents/PkgInfo"

IDENTITY="${MACOS_SIGN_IDENTITY:--}"
SIGNING="Ad-hoc signed; not notarized or Developer ID signed. macOS may block a downloaded copy."
if [[ "$IDENTITY" != "-" ]]; then
  SIGNING="Signed with the packager's selected identity; not notarized. Check its certificate before distribution."
fi
sign() {
  local ts=(--timestamp) kc=()
  [[ "$IDENTITY" = "-" ]] && ts=(--timestamp=none)
  [[ -n "${MACOS_KEYCHAIN:-}" ]] && kc=(--keychain "$MACOS_KEYCHAIN")
  codesign --force --sign "$IDENTITY" ${kc[@]+"${kc[@]}"} "${ts[@]}" \
    --options runtime --entitlements "$HERE/entitlements.plist" "$@"
}
xattr -cr "$APP"
sign "$APP/Contents/MacOS/GridCraft-native"
sign "$APP"
codesign --verify --strict --deep --verbose=2 "$APP"

cat >"$PACKAGE/INSTALL.md" <<INSTALL
# GridCraft Assistant Preview $VERSION

Source commit: $BUILD_SHA (version and short commit verified against binary --version).
Architecture: $MACHINE. Requires macOS $MIN_MACOS or newer.
Signing: $SIGNING

1. Verify the downloaded DMG against its adjacent .sha256 file with
   shasum -a 256 -c $NAME.dmg.sha256 in the download directory.
   For the ZIP alternative, use $NAME.zip.sha256 instead.
2. Open the DMG and drag **GridCraft Assistant Preview.app** to **Applications**.
   Alternatively, extract the ZIP and move that app to **/Applications**.
   Replace only the preview; keep the main **GridCraft.app** separate.
3. Open the preview. Keep your existing **GridCraft.app**; this is a separate app.
   If macOS blocks it, follow [Apple's per-app approval instructions](https://support.apple.com/en-us/102445)
   after verifying its source and checksum. Do not disable Gatekeeper globally.
4. Open **Assistant** and configure **File → Options → Assistant**. One selected
   model handles chat and workbook tools; no model or credentials are included.

Install [Codex CLI](https://developers.openai.com/codex/cli) and sign in through this
preview's isolated Craft connection on each Mac. Install [Ollama for macOS](https://ollama.com/download/mac)
separately and download the desired local model there if used.
The launcher discovers Codex on PATH, including standard Homebrew locations;
CRAFT_CODEX_BINARY can explicitly select another executable. Settings, chats,
credentials and downloaded models are never copied from the packaging machine.

Document preferences and recovery use
~/Library/Application Support/GridCraft Assistant Preview.
GRIDCRAFT_PROFILE_DIR can override that location. Assistant connections and chats
use the destination user's shared Craft AI store; CRAFT_AI_CONFIG_DIR can override
it. Requested workbook context and chat history reach the selected provider.

The preview launcher disables the standard GridCraft updater, including manual
Check for Updates. Preview updates arrive as new preview DMGs. Save your documents
and quit the preview before replacing it. The separate main GridCraft app keeps
its normal updater. Neither installation method copies your settings or credentials.

This archive contains the supplied binary, original app icon and project licences.
Optional font licences must accompany binaries built with CRAFT_FONTS_DIR. Packaging
and signature checks do not establish inference correctness or destination-Mac compatibility.
INSTALL

ditto -c -k --keepParent "$PACKAGE" "$WORK/$NAME.zip"
# Verify the delivered bytes after extraction, not just the staging bundle.
ditto -x -k "$WORK/$NAME.zip" "$WORK/check"
codesign --verify --strict --deep --verbose=2 "$WORK/check/$NAME/GridCraft Assistant Preview.app"
# Keep the signed app metadata unchanged while staging the DMG.
DMG_STAGE="$WORK/dmg"
DMG_BUILD="$WORK/$NAME.dmg"
mkdir -p "$DMG_STAGE"
ditto "$APP" "$DMG_STAGE/GridCraft Assistant Preview.app"
ln -s /Applications "$DMG_STAGE/Applications"
cp "$PACKAGE/INSTALL.md" "$DMG_STAGE/INSTALL.md"
hdiutil create -srcfolder "$DMG_STAGE" -volname "GridCraft Assistant Preview" \
  -fs HFS+ -format UDZO -imagekey zlib-level=9 "$DMG_BUILD"
DMG_MOUNT="$WORK/dmg-check"
mkdir -p "$DMG_MOUNT"
hdiutil attach -readonly -nobrowse -noautoopen -mountpoint "$DMG_MOUNT" "$DMG_BUILD" >/dev/null
codesign --verify --strict --deep --verbose=2 "$DMG_MOUNT/GridCraft Assistant Preview.app"
ditto "$DMG_MOUNT/GridCraft Assistant Preview.app" "$WORK/dmg-copy/GridCraft Assistant Preview.app"
codesign --verify --strict --deep --verbose=2 "$WORK/dmg-copy/GridCraft Assistant Preview.app"
hdiutil detach "$DMG_MOUNT" >/dev/null
DMG_MOUNT=""
# Publish only after both package formats have passed extraction/mount verification.
cp "$PACKAGE/INSTALL.md" "$README"
cp "$WORK/$NAME.zip" "$ZIP"
cp "$DMG_BUILD" "$DMG"
printf '%s  %s\n' "$(sha256 "$ZIP")" "$NAME.zip" >"$CHECKSUM"
echo "==> packaged $ZIP ($ACTUAL_ARCH; $SIGNING)"
printf '%s  %s\n' "$(sha256 "$DMG")" "$NAME.dmg" >"$DMG_CHECKSUM"
echo "==> packaged $DMG (bundle $SIGNING)"
ls -lh "$ZIP" "$CHECKSUM" "$DMG" "$DMG_CHECKSUM" "$README"
