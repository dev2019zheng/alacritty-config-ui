#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ROOT="${ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"
DIST="$ROOT/dist"
UPLOADS="$DIST/upload"
RAW_DIR="$DIST/alacritty-config-ui"
TARGET_DIR="$ROOT/target/release"
BUNDLE_DIR="$TARGET_DIR/bundle"
TAURI_CONFIG="$ROOT/src-tauri/tauri.conf.json"
ASSET_PREFIX="${RELEASE_ASSET_PREFIX:-alacritty-config-ui}"
APPIMAGE_PATH="$UPLOADS/$ASSET_PREFIX-linux.AppImage"
DEB_PATH="$UPLOADS/$ASSET_PREFIX-linux.deb"
TAR_PATH="$UPLOADS/$ASSET_PREFIX-linux.tar.gz"
RESOURCE_DIR_NAME="$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1], encoding="utf-8"))["productName"])' "$TAURI_CONFIG")"

sha256_file() {
  shasum -a 256 "$1" > "$1.sha256"
}

require_path() {
  local path="$1"
  local description="$2"
  if [[ ! -e "$path" ]]; then
    echo "missing $description at $path" >&2
    exit 1
  fi
}

assert_tar_contains() {
  local archive="$1"
  local entry="$2"
  if ! tar -tzf "$archive" "$entry" >/dev/null 2>&1; then
    echo "missing $entry in $archive" >&2
    exit 1
  fi
}

copy_portable_payload() {
  cp "$TARGET_DIR/alacritty-config-ui" "$RAW_DIR/alacritty-config-ui"
  chmod +x "$RAW_DIR/alacritty-config-ui"
  cp -R "$ROOT/vendor/alacritty-theme/themes" "$RAW_DIR/themes"
  cp "$ROOT/LICENSE-APACHE" "$RAW_DIR/LICENSE-APACHE"
  cp "$ROOT/LICENSE-MIT" "$RAW_DIR/LICENSE-MIT"
}

verify_appimage_contents() {
  local appimage="$1"
  local temp_dir
  temp_dir="$(mktemp -d)"
  chmod +x "$appimage"
  (
    cd "$temp_dir"
    "$appimage" --appimage-extract >/dev/null
  )

  local resource_root="$temp_dir/squashfs-root/usr/lib/$RESOURCE_DIR_NAME"
  require_path "$resource_root/themes" "bundled themes in AppImage"
  require_path "$resource_root/LICENSE-APACHE" "LICENSE-APACHE in AppImage"
  require_path "$resource_root/LICENSE-MIT" "LICENSE-MIT in AppImage"
  rm -rf "$temp_dir"
}

verify_deb_contents() {
  local deb="$1"
  local temp_dir
  temp_dir="$(mktemp -d)"
  dpkg-deb -x "$deb" "$temp_dir"

  local resource_root="$temp_dir/usr/lib/$RESOURCE_DIR_NAME"
  require_path "$resource_root/themes" "bundled themes in deb"
  require_path "$resource_root/LICENSE-APACHE" "LICENSE-APACHE in deb"
  require_path "$resource_root/LICENSE-MIT" "LICENSE-MIT in deb"
  rm -rf "$temp_dir"
}

rm -rf "$DIST"
mkdir -p "$UPLOADS" "$RAW_DIR"

npm run --prefix "$ROOT" tauri build -- --bundles appimage,deb

APPIMAGE_SOURCE="$(find "$BUNDLE_DIR/appimage" -maxdepth 1 -type f -name '*.AppImage' -print -quit)"
DEB_SOURCE="$(find "$BUNDLE_DIR/deb" -maxdepth 1 -type f -name '*.deb' -print -quit)"

if [[ -z "$APPIMAGE_SOURCE" ]]; then
  echo "missing AppImage output in $BUNDLE_DIR/appimage" >&2
  exit 1
fi
if [[ -z "$DEB_SOURCE" ]]; then
  echo "missing deb output in $BUNDLE_DIR/deb" >&2
  exit 1
fi

verify_appimage_contents "$APPIMAGE_SOURCE"
verify_deb_contents "$DEB_SOURCE"

copy_portable_payload
COPYFILE_DISABLE=1 tar -czf "$TAR_PATH" -C "$DIST" "$(basename "$RAW_DIR")"
assert_tar_contains "$TAR_PATH" "$(basename "$RAW_DIR")/alacritty-config-ui"
assert_tar_contains "$TAR_PATH" "$(basename "$RAW_DIR")/themes"
assert_tar_contains "$TAR_PATH" "$(basename "$RAW_DIR")/LICENSE-APACHE"
assert_tar_contains "$TAR_PATH" "$(basename "$RAW_DIR")/LICENSE-MIT"
sha256_file "$TAR_PATH"

cp "$APPIMAGE_SOURCE" "$APPIMAGE_PATH"
cp "$DEB_SOURCE" "$DEB_PATH"
sha256_file "$APPIMAGE_PATH"
sha256_file "$DEB_PATH"

printf 'created_artifact=%s\n' "$APPIMAGE_PATH"
printf 'created_artifact=%s\n' "$DEB_PATH"
printf 'created_artifact=%s\n' "$TAR_PATH"
