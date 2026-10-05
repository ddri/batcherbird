#!/usr/bin/env bash
set -euo pipefail

# Determine script and project directories
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

APP_NAME="Batcherbird"
VERSION="$(grep -m1 '^version' crates/batcherbird-vizia/Cargo.toml | cut -d '"' -f2)"
# Stage inside the output directory so installation uses same-filesystem renames.
DIST_DIR="${BATCHERBIRD_DIST_DIR-${ROOT_DIR}/dist}"
if [[ -z "${DIST_DIR}" ]]; then
    echo "Output directory must not be empty." >&2
    exit 1
fi
mkdir -p "${DIST_DIR}"
DIST_DIR="$(cd "${DIST_DIR}" && pwd -P)"
case "${ROOT_DIR}/" in
    "${DIST_DIR}/"*) echo "Output directory must not be the repository or an ancestor." >&2; exit 1 ;;
esac
if [[ "${DIST_DIR}" == "/" || "${DIST_DIR}" == "${HOME}" ]]; then
    echo "Output directory must not be a filesystem or home root." >&2
    exit 1
fi
TARGET_DIR="${CARGO_TARGET_DIR:-${ROOT_DIR}/target}"
if [[ "${TARGET_DIR}" != /* ]]; then TARGET_DIR="${ROOT_DIR}/${TARGET_DIR}"; fi
FINAL_APP="${DIST_DIR}/${APP_NAME}.app"
FINAL_DMG="${DIST_DIR}/${APP_NAME}.dmg"
# Refuse ambiguous output types rather than overwriting an unrelated object.
if [[ -L "${FINAL_APP}" || ( -e "${FINAL_APP}" && ! -d "${FINAL_APP}" )
    || -L "${FINAL_DMG}" || ( -e "${FINAL_DMG}" && ! -f "${FINAL_DMG}" ) ]]; then
    echo "Existing package outputs have unexpected types or are symbolic links." >&2
    exit 1
fi

STAGING_DIR="$(mktemp -d "${DIST_DIR}/.batcherbird-package.XXXXXX")"
APP_DIR="${STAGING_DIR}/${APP_NAME}.app"
CONTENTS_DIR="${APP_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"
DMG_STAGING="${STAGING_DIR}/dmg_staging"
DMG_OUTPUT="${STAGING_DIR}/${APP_NAME}.dmg"
APP_INSTALLED=false
DMG_INSTALLED=false
COMMITTED=false

cleanup() {
    local status=$?
    trap - EXIT
    # Only remove the unique staging directory created above and outputs installed
    # by this invocation. Never remove the user's output directory.
    case "${STAGING_DIR}" in "${DIST_DIR}/.batcherbird-package."*) ;; *) exit 1 ;; esac
    if [[ "${COMMITTED}" != true ]]; then
        if [[ "${APP_INSTALLED}" == true ]]; then rm -rf "${FINAL_APP}"; fi
        if [[ "${DMG_INSTALLED}" == true ]]; then rm -f "${FINAL_DMG}"; fi
        if [[ -e "${STAGING_DIR}/previous.app" ]]; then mv "${STAGING_DIR}/previous.app" "${FINAL_APP}"; fi
        if [[ -e "${STAGING_DIR}/previous.dmg" ]]; then mv "${STAGING_DIR}/previous.dmg" "${FINAL_DMG}"; fi
    fi
    rm -rf "${STAGING_DIR}"
    exit "${status}"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

echo "==> Building ${APP_NAME} v${VERSION} (release mode)..."
cargo build --release -p batcherbird-vizia

echo "==> Assembling ${APP_NAME}.app bundle..."
mkdir -p "${MACOS_DIR}" "${RESOURCES_DIR}"
cp "${TARGET_DIR}/release/batcherbird-vizia" "${MACOS_DIR}/${APP_NAME}"
chmod +x "${MACOS_DIR}/${APP_NAME}"

# Match the bundle's minimum OS to the executable, including the highest minimum
# among slices if a universal binary is packaged. Older Intel binaries use
# LC_VERSION_MIN_MACOSX; current builds use LC_BUILD_VERSION.
MIN_MACOS_VERSION="11.0"
if command -v otool >/dev/null 2>&1; then
    MACH_MIN="$(otool -l "${MACOS_DIR}/${APP_NAME}" 2>/dev/null | awk '
        $1 == "cmd" { load_command = $2 }
        (load_command == "LC_BUILD_VERSION" && $1 == "minos") ||
        (load_command == "LC_VERSION_MIN_MACOSX" && $1 == "version") {
            split($2, candidate, "."); split(selected, current, ".")
            greater = (selected == "")
            for (i = 1; i <= 3; i++) {
                if (candidate[i] + 0 > current[i] + 0) { greater = 1; break }
                if (candidate[i] + 0 < current[i] + 0) { break }
            }
            if (greater) selected = $2
        }
        END { print selected }
    ')" || MACH_MIN=""
    if [[ "${MACH_MIN}" =~ ^[0-9]+(\.[0-9]+){1,2}$ ]]; then
        MIN_MACOS_VERSION="${MACH_MIN}"
    else
        echo "==> Could not inspect Mach-O minimum; using macOS ${MIN_MACOS_VERSION} fallback"
    fi
fi

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
    <string>${MIN_MACOS_VERSION}</string>
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

    if [[ ! -s "${DMG_OUTPUT}" ]]; then
        echo "DMG tool did not produce a nonempty image." >&2
        exit 1
    fi
    echo "==> DMG staging complete"
else
    echo "==> Skipped DMG generation (hdiutil not found, non-macOS environment)"
fi

# Keep previous artifacts until both replacements are ready, and roll back if an
# installation step fails. Unrelated files in the output directory remain intact.
if [[ -e "${FINAL_APP}" ]]; then mv "${FINAL_APP}" "${STAGING_DIR}/previous.app"; fi
if [[ -f "${DMG_OUTPUT}" && -e "${FINAL_DMG}" ]]; then mv "${FINAL_DMG}" "${STAGING_DIR}/previous.dmg"; fi
APP_INSTALLED=true
mv "${APP_DIR}" "${FINAL_APP}"
if [[ -f "${DMG_OUTPUT}" ]]; then
    DMG_INSTALLED=true
    mv "${DMG_OUTPUT}" "${FINAL_DMG}"
fi
COMMITTED=true

echo "==> Created ${FINAL_APP}"
if [[ "${DMG_INSTALLED}" == true ]]; then echo "==> Created DMG: ${FINAL_DMG}"; fi
echo "==> Done!"
