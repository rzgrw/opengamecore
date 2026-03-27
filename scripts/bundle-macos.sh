#!/bin/bash
set -euo pipefail

APP_NAME="OpenGameCore"
BUNDLE_ID="com.opengamecore.app"
VERSION="${1:-0.1.0}"

echo "Building release binaries..."
cargo build --release --workspace

echo "Creating app bundle..."
BUNDLE_DIR="target/release/${APP_NAME}.app"
rm -rf "$BUNDLE_DIR"

mkdir -p "$BUNDLE_DIR/Contents/MacOS"
mkdir -p "$BUNDLE_DIR/Contents/Resources"

# Copy binaries
cp "target/release/opengamecore-app" "$BUNDLE_DIR/Contents/MacOS/${APP_NAME}"
cp "target/release/ogc" "$BUNDLE_DIR/Contents/MacOS/ogc"

# Create Info.plist
cat > "$BUNDLE_DIR/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>12.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>
PLIST

# Create PkgInfo
echo -n "APPL????" > "$BUNDLE_DIR/Contents/PkgInfo"

echo "App bundle created at: $BUNDLE_DIR"

# Create DMG
echo "Creating DMG..."
DMG_NAME="OpenGameCore-${VERSION}-macOS"
DMG_DIR="target/release/dmg"
rm -rf "$DMG_DIR"
mkdir -p "$DMG_DIR"
cp -r "$BUNDLE_DIR" "$DMG_DIR/"

# Create a symlink to /Applications for drag-and-drop install
ln -s /Applications "$DMG_DIR/Applications"

hdiutil create -volname "$APP_NAME" \
    -srcfolder "$DMG_DIR" \
    -ov -format UDZO \
    "target/release/${DMG_NAME}.dmg"

echo "DMG created at: target/release/${DMG_NAME}.dmg"
