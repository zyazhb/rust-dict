#!/usr/bin/env bash
# Build Eng Dict as a macOS .app bundle and wrap it in a .dmg installer.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
STAGING="$DIST/dmg-staging"
APP_NAME="Eng Dict"
BUNDLE_ID="dev.eng-dict.app"
BINARY_NAME="dict-app"
ICON_PNG="$ROOT/assets/app-icon.png"

prepare_square_icon() {
  local src_png="$1"
  local out_png="$2"

  if command -v magick >/dev/null 2>&1; then
    magick "$src_png" \
      -gravity center -crop 1:1 +repage \
      -fuzz 12% -trim +repage \
      -resize 1120x1120 \
      -gravity center -background none -extent 1024x1024 \
      "$out_png"
    return
  fi

  local width height size
  width="$(sips -g pixelWidth "$src_png" | awk '/pixelWidth/{print $2}')"
  height="$(sips -g pixelHeight "$src_png" | awk '/pixelHeight/{print $2}')"
  if [[ "$width" -eq "$height" ]]; then
    cp "$src_png" "$out_png"
  else
    if [[ "$width" -lt "$height" ]]; then
      size="$width"
    else
      size="$height"
    fi
    sips -c "$size" "$size" "$src_png" --out "$out_png" >/dev/null
  fi
  sips -z 1024 1024 "$out_png" >/dev/null
}

make_icns() {
  local src_png="$1"
  local out_icns="$2"
  local work iconset square_png

  work="$(mktemp -d)"
  iconset="$work/AppIcon.iconset"
  square_png="$work/icon-square.png"
  mkdir -p "$iconset"

  prepare_square_icon "$src_png" "$square_png"

  sips -z 16 16 "$square_png" --out "$iconset/icon_16x16.png" >/dev/null
  sips -z 32 32 "$square_png" --out "$iconset/icon_16x16@2x.png" >/dev/null
  sips -z 32 32 "$square_png" --out "$iconset/icon_32x32.png" >/dev/null
  sips -z 64 64 "$square_png" --out "$iconset/icon_32x32@2x.png" >/dev/null
  sips -z 128 128 "$square_png" --out "$iconset/icon_128x128.png" >/dev/null
  sips -z 256 256 "$square_png" --out "$iconset/icon_128x128@2x.png" >/dev/null
  sips -z 256 256 "$square_png" --out "$iconset/icon_256x256.png" >/dev/null
  sips -z 512 512 "$square_png" --out "$iconset/icon_256x256@2x.png" >/dev/null
  sips -z 512 512 "$square_png" --out "$iconset/icon_512x512.png" >/dev/null
  cp "$square_png" "$iconset/icon_512x512@2x.png"

  iconutil -c icns "$iconset" -o "$out_icns"
  rm -rf "$work"
}

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "error: this script must run on macOS" >&2
  exit 1
fi

VERSION="$(
  sed -n 's/^version = "\(.*\)"/\1/p' "$ROOT/Cargo.toml" | head -1
)"
if [[ -z "$VERSION" ]]; then
  VERSION="0.0.0"
fi

ARCH="$(uname -m)"
DMG_NAME="eng-dict-${VERSION}-macos-${ARCH}.dmg"
APP="$STAGING/${APP_NAME}.app"
CONTENTS="$APP/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"

if [[ ! -f "$ICON_PNG" ]]; then
  echo "error: app icon not found at $ICON_PNG" >&2
  exit 1
fi

cd "$ROOT"

"$ROOT/scripts/prepare-dict.sh"

echo "Building release binary with bundled-dict feature..."
cargo build --release -p dict-app --features bundled-dict

echo "Creating app bundle..."
rm -rf "$STAGING"
mkdir -p "$MACOS" "$RESOURCES"

cp "$ROOT/target/release/$BINARY_NAME" "$MACOS/$BINARY_NAME"
chmod +x "$MACOS/$BINARY_NAME"

echo "Generating app icon..."
make_icns "$ICON_PNG" "$RESOURCES/AppIcon.icns"

cat > "$CONTENTS/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDevelopmentRegion</key>
  <string>en</string>
  <key>CFBundleExecutable</key>
  <string>${BINARY_NAME}</string>
  <key>CFBundleIdentifier</key>
  <string>${BUNDLE_ID}</string>
  <key>CFBundleInfoDictionaryVersion</key>
  <string>6.0</string>
  <key>CFBundleName</key>
  <string>${APP_NAME}</string>
  <key>CFBundleDisplayName</key>
  <string>${APP_NAME}</string>
  <key>CFBundleIconFile</key>
  <string>AppIcon</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleShortVersionString</key>
  <string>${VERSION}</string>
  <key>CFBundleVersion</key>
  <string>${VERSION}</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSPrincipalClass</key>
  <string>NSApplication</string>
</dict>
</plist>
EOF

if command -v codesign >/dev/null 2>&1; then
  echo "Applying ad-hoc code signature..."
  codesign --force --deep --sign - "$APP"
fi

mkdir -p "$DIST"
ln -sf /Applications "$STAGING/Applications"

DMG_PATH="$DIST/$DMG_NAME"
rm -f "$DMG_PATH"

echo "Creating disk image..."
hdiutil create \
  -volname "$APP_NAME" \
  -srcfolder "$STAGING" \
  -ov \
  -format UDZO \
  "$DMG_PATH" >/dev/null

rm -rf "$STAGING"

ls -lh "$DMG_PATH"
echo ""
echo "macOS disk image: $DMG_PATH"
echo "Open the DMG and drag \"${APP_NAME}\" into Applications to install."
