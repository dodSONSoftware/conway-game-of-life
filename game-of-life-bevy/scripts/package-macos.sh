#!/bin/bash
set -euo pipefail
cd "$(dirname "$0")/.."
version=$(<../VERSION)
cargo build --locked --release
app="target/release/Game of Life.app"
resources="$app/Contents/Resources"
mkdir -p "$app/Contents/MacOS" "$resources"
cp target/release/game-of-life-bevy "$app/Contents/MacOS/Game of Life"
ditto assets/patterns "$resources/patterns"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
icons="$work/GameOfLife.iconset"
mkdir "$icons"
for size in 16 32 128 256 512; do
    sips -z "$size" "$size" assets/app-icon.png --out "$icons/icon_${size}x${size}.png" >/dev/null
    double=$((size * 2))
    sips -z "$double" "$double" assets/app-icon.png --out "$icons/icon_${size}x${size}@2x.png" >/dev/null
done
iconutil -c icns "$icons" -o "$resources/GameOfLife.icns"
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>com.randeldodson.gameoflife</string>
<key>CFBundleName</key><string>Game of Life</string>
<key>CFBundleExecutable</key><string>Game of Life</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleIconFile</key><string>GameOfLife.icns</string>
<key>CFBundleShortVersionString</key><string>$version</string>
<key>CFBundleVersion</key><string>$version</string>
</dict></plist>
PLIST
plutil -lint "$app/Contents/Info.plist"
codesign --force --sign - "$app"
codesign --verify --strict "$app"
