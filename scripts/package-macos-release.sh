#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="${ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"
DIST="$ROOT/dist"
UPLOADS="$DIST/upload"
RAW_DIR="$DIST/alacritty-config-ui"
TARGET_DIR="$ROOT/target/release"
BUNDLE_DIR="$TARGET_DIR/bundle"
APP_NAME="Alacritty Config UI"
APP_BUNDLE="$BUNDLE_DIR/macos/$APP_NAME.app"
ICON_ICNS="$ROOT/assets/app-icon.icns"
TAR_PATH="$UPLOADS/alacritty-config-ui-macos.tar.gz"

APPLE_CERTIFICATE_VALUE="${APPLE_CERTIFICATE:-${APPLE_CERTIFICATE_P12_BASE64:-}}"
APPLE_CERTIFICATE_PASSWORD_VALUE="${APPLE_CERTIFICATE_PASSWORD:-}"
APPLE_SIGNING_IDENTITY_VALUE="${APPLE_SIGNING_IDENTITY:-}"
APPLE_ID_VALUE="${APPLE_ID:-${APPLE_NOTARY_APPLE_ID:-}}"
APPLE_PASSWORD_VALUE="${APPLE_PASSWORD:-${APPLE_NOTARY_APP_SPECIFIC_PASSWORD:-}}"
APPLE_TEAM_ID_VALUE="${APPLE_TEAM_ID:-${APPLE_NOTARY_TEAM_ID:-}}"
APPLE_API_KEY_VALUE="${APPLE_API_KEY:-}"
APPLE_API_ISSUER_VALUE="${APPLE_API_ISSUER:-}"
APPLE_API_KEY_PATH_VALUE="${APPLE_API_KEY_PATH:-}"

sha256_file() {
  shasum -a 256 "$1" > "$1.sha256"
}

export_if_set() {
  local name="$1"
  local value="$2"

  if [[ -n "$value" ]]; then
    export "$name=$value"
  else
    unset "$name"
  fi
}

signing_configured() {
  [[ -n "$APPLE_CERTIFICATE_VALUE" || -n "$APPLE_SIGNING_IDENTITY_VALUE" ]]
}

notarization_configured() {
  if [[ -n "$APPLE_ID_VALUE" && -n "$APPLE_PASSWORD_VALUE" && -n "$APPLE_TEAM_ID_VALUE" ]]; then
    return 0
  fi

  if [[ -n "$APPLE_API_KEY_VALUE" && -n "$APPLE_API_ISSUER_VALUE" && -n "$APPLE_API_KEY_PATH_VALUE" ]]; then
    return 0
  fi

  return 1
}

copy_raw_payload() {
  local resources_dir="$APP_BUNDLE/Contents/Resources"
  local macos_dir="$APP_BUNDLE/Contents/MacOS"

  cp "$macos_dir/alacritty-config-ui" "$RAW_DIR/alacritty-config-ui"
  chmod +x "$RAW_DIR/alacritty-config-ui"
  cp -R "$resources_dir/themes" "$RAW_DIR/themes"
  cp "$resources_dir/LICENSE-APACHE" "$RAW_DIR/LICENSE-APACHE"
  cp "$resources_dir/LICENSE-MIT" "$RAW_DIR/LICENSE-MIT"
}

mkdir -p "$ROOT/assets"
if [[ ! -f "$ICON_ICNS" ]]; then
  echo "missing bundle icon at $ICON_ICNS; run python3 scripts/generate_app_icon.py first" >&2
  exit 1
fi

rm -rf "$DIST" "$BUNDLE_DIR"
mkdir -p "$UPLOADS" "$RAW_DIR"

export_if_set APPLE_CERTIFICATE "$APPLE_CERTIFICATE_VALUE"
export_if_set APPLE_CERTIFICATE_PASSWORD "$APPLE_CERTIFICATE_PASSWORD_VALUE"
export_if_set APPLE_SIGNING_IDENTITY "$APPLE_SIGNING_IDENTITY_VALUE"
export_if_set APPLE_ID "$APPLE_ID_VALUE"
export_if_set APPLE_PASSWORD "$APPLE_PASSWORD_VALUE"
export_if_set APPLE_TEAM_ID "$APPLE_TEAM_ID_VALUE"
export_if_set APPLE_API_KEY "$APPLE_API_KEY_VALUE"
export_if_set APPLE_API_ISSUER "$APPLE_API_ISSUER_VALUE"
export_if_set APPLE_API_KEY_PATH "$APPLE_API_KEY_PATH_VALUE"

npm run --prefix "$ROOT" tauri build

if [[ ! -d "$APP_BUNDLE" ]]; then
  echo "missing app bundle at $APP_BUNDLE" >&2
  exit 1
fi

if [[ ! -d "$APP_BUNDLE/Contents/Resources/themes" ]]; then
  echo "missing bundled themes in $APP_BUNDLE/Contents/Resources/themes" >&2
  exit 1
fi

if [[ ! -f "$APP_BUNDLE/Contents/Resources/LICENSE-APACHE" || ! -f "$APP_BUNDLE/Contents/Resources/LICENSE-MIT" ]]; then
  echo "missing bundled license files in $APP_BUNDLE/Contents/Resources" >&2
  exit 1
fi

copy_raw_payload
COPYFILE_DISABLE=1 tar -czf "$TAR_PATH" -C "$DIST" "$(basename "$RAW_DIR")"
sha256_file "$TAR_PATH"

DMG_SOURCE="$(find "$BUNDLE_DIR/dmg" -maxdepth 1 -type f -name '*.dmg' | head -n 1)"
if [[ -z "$DMG_SOURCE" ]]; then
  echo "missing dmg output in $BUNDLE_DIR/dmg" >&2
  exit 1
fi

if signing_configured && notarization_configured; then
  DMG_PATH="$UPLOADS/alacritty-config-ui-macos.dmg"
elif signing_configured; then
  DMG_PATH="$UPLOADS/alacritty-config-ui-macos-signed-unnotarized.dmg"
else
  DMG_PATH="$UPLOADS/alacritty-config-ui-macos-unsigned.dmg"
fi

cp "$DMG_SOURCE" "$DMG_PATH"
sha256_file "$DMG_PATH"

printf 'created_artifact=%s\n' "$TAR_PATH"
printf 'created_artifact=%s\n' "$DMG_PATH"
