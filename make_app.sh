#!/bin/bash
set -e

# Creates a macOS app bundle from the cargo-installed jid binary
# Usage: curl -sSL https://raw.githubusercontent.com/ehamiter/jid/main/make_app.sh | bash

APP_NAME="jid"
BUNDLE_ID="com.jid.app"
VERSION="0.1.0"

# Find the jid binary
JID_BIN=$(which jid 2>/dev/null || echo "")

if [ -z "$JID_BIN" ]; then
    echo "Error: jid not found. Install it first with: cargo install jotitdown"
    exit 1
fi

echo "Found jid at: $JID_BIN"
echo "Creating app bundle..."

APP_DIR="/Applications/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

# Remove existing app if present
if [ -d "$APP_DIR" ]; then
    echo "Removing existing app bundle..."
    rm -rf "$APP_DIR"
fi

mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

cp "$JID_BIN" "${MACOS_DIR}/${APP_NAME}"

# Download app icon
echo "Downloading app icon..."
curl -sSL "https://raw.githubusercontent.com/ehamiter/jid/main/assets/icons/AppIcon.icns" -o "${RESOURCES_DIR}/AppIcon.icns"

cat > "${CONTENTS_DIR}/Info.plist" << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
</dict>
</plist>
EOF

echo "Done! jid.app installed to /Applications/"
echo "You can now launch it from Spotlight or Finder."
