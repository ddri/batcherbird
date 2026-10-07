#!/usr/bin/env bash
# Real plist/Mach-O inspection with controlled dynamic dependency output.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/batcherbird-audit-test.XXXXXX")"
trap 'rm -rf "${FIXTURE_DIR}"' EXIT
APP="${FIXTURE_DIR}/Moved App.app"
mkdir -p "${APP}/Contents/MacOS" "${FIXTURE_DIR}/bin"
cp /usr/bin/true "${APP}/Contents/MacOS/Batcherbird"
PLIST="${APP}/Contents/Info.plist"
cat > "${PLIST}" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleIdentifier</key><string>com.davidryan.batcherbird</string>
<key>CFBundleExecutable</key><string>Batcherbird</string>
<key>NSMicrophoneUsageDescription</key><string>Record audio input.</string>
<key>LSMinimumSystemVersion</key><string>11.0</string>
</dict></plist>
PLIST
cat > "${FIXTURE_DIR}/bin/otool" <<'TOOL'
#!/usr/bin/env bash
printf '%s:\n' "${!#}"
printf '\t%s (compatibility version 1.0.0, current version 1.0.0)\n' "${FIXTURE_DEPENDENCY:-/usr/lib/libSystem.B.dylib}"
TOOL
chmod +x "${FIXTURE_DIR}/bin/otool"
export PATH="${FIXTURE_DIR}/bin:${PATH}"
unset BATCHERBIRD_REQUIRE_NOTARIZED
bash "${ROOT_DIR}/scripts/verify-macos-package.sh" "${APP}" > "${FIXTURE_DIR}/log" 2>&1
for dependency in /opt/homebrew/lib/libfixture.dylib '@rpath/libfixture.dylib'; do
    if FIXTURE_DEPENDENCY="${dependency}" bash "${ROOT_DIR}/scripts/verify-macos-package.sh" "${APP}" > "${FIXTURE_DIR}/log" 2>&1; then
        echo "Expected rejection for developer-machine dependency ${dependency}" >&2; exit 1
    fi
done
/usr/libexec/PlistBuddy -c 'Delete :NSMicrophoneUsageDescription' "${PLIST}"
if bash "${ROOT_DIR}/scripts/verify-macos-package.sh" "${APP}" > "${FIXTURE_DIR}/log" 2>&1; then
    echo "Expected rejection for missing microphone purpose string." >&2; exit 1
fi
echo 'Standalone audit fixtures passed: moved bundle, system dependencies, developer-machine dylibs, and missing microphone purpose.'
