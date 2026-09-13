#!/usr/bin/env bash
set -euo pipefail

# Determine script and project directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

APP_NAME="Batcherbird"
VERSION="$(grep -m1 '^version' crates/batcherbird-vizia/Cargo.toml | cut -d '"' -f2)"
DIST_DIR="${ROOT_DIR}/dist"
APP_DIR="${DIST_DIR}/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"
DMG_STAGING="${DIST_DIR}/dmg_staging"
DMG_NAME="${APP_NAME}.dmg"
DMG_OUTPUT="${DIST_DIR}/${DMG_NAME}"

echo "==> Building ${APP_NAME} v${VERSION} (release mode)..."
cargo build --release -p batcherbird-vizia

echo "==> Assembling ${APP_NAME}.app bundle..."
rm -rf "${DIST_DIR}"
mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}"

# Copy binary to bundle
cp "${ROOT_DIR}/target/release/batcherbird-vizia" "${MACOS_DIR}/${APP_NAME}"
chmod +x "${MACOS_DIR}/${APP_NAME}"

# Generate Info.plist
cat <<EOF > "${CONTENTS_DIR}/Info.plist"
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>com.davidryan.batcherbird</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleSignature</key>
    <string>????</string>
    <key>CFBundleExecutable</key>
    <string>${APP_NAME}</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSMicrophoneUsageDescription</key>
    <string>Batcherbird requires microphone and audio input access to record audio from your audio interface and hardware synthesizers.</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
</dict>
</plist>
EOF

echo "==> Created ${APP_DIR}"

# If running on macOS with hdiutil, package DMG installer
if command -v hdiutil >/dev/null 2>&1; then
    echo "==> Packaging DMG with hdiutil..."
    mkdir -p "${DMG_STAGING}"
    cp -R "${APP_DIR}" "${DMG_STAGING}/"
    ln -s /Applications "${DMG_STAGING}/Applications"

    hdiutil create \
        -volname "${APP_NAME}" \
        -srcfolder "${DMG_STAGING}" \
        -ov \
        -format UDZO \
        "${DMG_OUTPUT}"

    rm -rf "${DMG_STAGING}"
    echo "==> Created DMG: ${DMG_OUTPUT}"
else
    echo "==> Skipped DMG generation (hdiutil not found, non-macOS environment)"
fi

echo "==> Done!"
