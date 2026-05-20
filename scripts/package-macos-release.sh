#!/usr/bin/env bash
set -euo pipefail

ROOT="${ROOT:-$(git rev-parse --show-toplevel)}"
DIST="$ROOT/dist"
UPLOADS="$DIST/upload"
RAW_DIR="$DIST/alacritty-config-ui"
APP_NAME="Alacritty Config UI"
APP_DIR="$DIST/$APP_NAME.app"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"
ICONSET_DIR="$DIST/alacritty-config-ui.iconset"
ICON_PNG="$ROOT/assets/app-icon.png"
ICNS_PATH="$RESOURCES_DIR/alacritty-config-ui.icns"
BINARY_PATH="$ROOT/target/release/alacritty-config-ui"
BUNDLE_ID="io.github.dev2019zheng.alacritty-config-ui"
TAR_PATH="$UPLOADS/alacritty-config-ui-macos.tar.gz"
SIGNED_DMG_PATH="$UPLOADS/alacritty-config-ui-macos.dmg"
UNSIGNED_DMG_PATH="$UPLOADS/alacritty-config-ui-macos-unsigned.dmg"

APPLE_SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY:-}"
APPLE_NOTARY_APPLE_ID="${APPLE_NOTARY_APPLE_ID:-}"
APPLE_NOTARY_TEAM_ID="${APPLE_NOTARY_TEAM_ID:-}"
APPLE_NOTARY_APP_SPECIFIC_PASSWORD="${APPLE_NOTARY_APP_SPECIFIC_PASSWORD:-}"

VERSION="${VERSION:-$(python3 - "$ROOT/Cargo.toml" <<'PY'
import sys
import tomllib
with open(sys.argv[1], 'rb') as fh:
    print(tomllib.load(fh)['package']['version'])
PY
)}"

mkdir -p "$UPLOADS"
rm -rf "$RAW_DIR" "$APP_DIR" "$ICONSET_DIR" "$DIST/dmg-root"
mkdir -p "$RAW_DIR" "$MACOS_DIR" "$RESOURCES_DIR"

if [[ ! -f "$BINARY_PATH" ]]; then
  echo "missing release binary at $BINARY_PATH" >&2
  exit 1
fi

if [[ ! -f "$ICON_PNG" ]]; then
  echo "missing icon asset at $ICON_PNG" >&2
  exit 1
fi

copy_raw_payload() {
  local target_dir="$1"
  mkdir -p "$target_dir"
  cp "$BINARY_PATH" "$target_dir/alacritty-config-ui"
  chmod +x "$target_dir/alacritty-config-ui"
  cp -R "$ROOT/vendor/alacritty-theme/themes" "$target_dir/themes"
  cp "$ROOT/LICENSE-APACHE" "$target_dir/LICENSE-APACHE"
  cp "$ROOT/LICENSE-MIT" "$target_dir/LICENSE-MIT"
}

copy_app_payload() {
  cp "$BINARY_PATH" "$MACOS_DIR/alacritty-config-ui"
  chmod +x "$MACOS_DIR/alacritty-config-ui"
  cp -R "$ROOT/vendor/alacritty-theme/themes" "$RESOURCES_DIR/themes"
  cp "$ROOT/LICENSE-APACHE" "$RESOURCES_DIR/LICENSE-APACHE"
  cp "$ROOT/LICENSE-MIT" "$RESOURCES_DIR/LICENSE-MIT"
}

render_icns() {
  mkdir -p "$ICONSET_DIR"
  local spec
  for spec in \
    "16 16x16" \
    "32 16x16@2x" \
    "32 32x32" \
    "64 32x32@2x" \
    "128 128x128" \
    "256 128x128@2x" \
    "256 256x256" \
    "512 256x256@2x" \
    "512 512x512" \
    "1024 512x512@2x"; do
    set -- $spec
    sips -z "$1" "$1" "$ICON_PNG" --out "$ICONSET_DIR/icon_$2.png" >/dev/null
  done
  iconutil -c icns "$ICONSET_DIR" -o "$ICNS_PATH"
}

write_info_plist() {
  cat > "$CONTENTS_DIR/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>$APP_NAME</string>
    <key>CFBundleExecutable</key>
    <string>alacritty-config-ui</string>
    <key>CFBundleIconFile</key>
    <string>alacritty-config-ui.icns</string>
    <key>CFBundleIdentifier</key>
    <string>$BUNDLE_ID</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>$APP_NAME</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>$VERSION</string>
    <key>CFBundleVersion</key>
    <string>$VERSION</string>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.developer-tools</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
  </dict>
</plist>
EOF
}

sha256_file() {
  shasum -a 256 "$1" > "$1.sha256"
}

sign_if_configured() {
  if [[ -z "$APPLE_SIGNING_IDENTITY" ]]; then
    echo "signing=disabled"
    return 1
  fi

  codesign --force --options runtime --sign "$APPLE_SIGNING_IDENTITY" "$MACOS_DIR/alacritty-config-ui"
  codesign --force --options runtime --sign "$APPLE_SIGNING_IDENTITY" "$APP_DIR"
  codesign --verify --deep --strict --verbose=2 "$APP_DIR"
  echo "signing=enabled"
}

notarize_if_configured() {
  local dmg_path="$1"
  if [[ -z "$APPLE_SIGNING_IDENTITY" || -z "$APPLE_NOTARY_APPLE_ID" || -z "$APPLE_NOTARY_TEAM_ID" || -z "$APPLE_NOTARY_APP_SPECIFIC_PASSWORD" ]]; then
    echo "notarization=disabled"
    return 1
  fi

  codesign --force --sign "$APPLE_SIGNING_IDENTITY" "$dmg_path"
  xcrun notarytool submit "$dmg_path" \
    --apple-id "$APPLE_NOTARY_APPLE_ID" \
    --password "$APPLE_NOTARY_APP_SPECIFIC_PASSWORD" \
    --team-id "$APPLE_NOTARY_TEAM_ID" \
    --wait
  xcrun stapler staple "$dmg_path"
  echo "notarization=enabled"
}

copy_raw_payload "$RAW_DIR"
copy_app_payload
render_icns
write_info_plist

COPYFILE_DISABLE=1 tar -czf "$TAR_PATH" -C "$DIST" "$(basename "$RAW_DIR")"
sha256_file "$TAR_PATH"

DMG_ROOT="$DIST/dmg-root"
mkdir -p "$DMG_ROOT"
cp -R "$APP_DIR" "$DMG_ROOT/$APP_NAME.app"

DMG_PATH="$UNSIGNED_DMG_PATH"
if sign_if_configured; then
  if [[ -n "$APPLE_NOTARY_APPLE_ID" && -n "$APPLE_NOTARY_TEAM_ID" && -n "$APPLE_NOTARY_APP_SPECIFIC_PASSWORD" ]]; then
    DMG_PATH="$SIGNED_DMG_PATH"
  fi
fi

COPYFILE_DISABLE=1 hdiutil create -volname "$APP_NAME" -srcfolder "$DMG_ROOT" -ov -format UDZO "$DMG_PATH" >/dev/null
notarize_if_configured "$DMG_PATH" || true
sha256_file "$DMG_PATH"

printf 'created_artifact=%s\n' "$TAR_PATH"
printf 'created_artifact=%s\n' "$DMG_PATH"
