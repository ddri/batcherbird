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
if [[ "$1" == -l ]]; then
    printf 'cmd LC_BUILD_VERSION\n minos 11.0\n sdk 26.2\n'
    exit 0
fi
printf '%s:\n' "${!#}"
printf '\t%s (compatibility version 1.0.0, current version 1.0.0)\n' "${FIXTURE_DEPENDENCY:-/usr/lib/libSystem.B.dylib}"
TOOL
cat > "${FIXTURE_DIR}/bin/codesign" <<'TOOL'
#!/usr/bin/env bash
if [[ "${FIXTURE_INVALID_SIGNATURE:-0}" == 1 ]]; then exit 23; fi
TOOL
chmod +x "${FIXTURE_DIR}/bin/otool" "${FIXTURE_DIR}/bin/codesign"
export PATH="${FIXTURE_DIR}/bin:${PATH}"
unset BATCHERBIRD_REQUIRE_NOTARIZED
bash "${ROOT_DIR}/scripts/verify-macos-package.sh" "${APP}" > "${FIXTURE_DIR}/log" 2>&1
if FIXTURE_INVALID_SIGNATURE=1 bash "${ROOT_DIR}/scripts/verify-macos-package.sh" "${APP}" > "${FIXTURE_DIR}/log" 2>&1; then
    echo "Expected rejection for invalid bundle signature." >&2; exit 1
fi
for dependency in /opt/homebrew/lib/libfixture.dylib '@rpath/libfixture.dylib'; do
    if FIXTURE_DEPENDENCY="${dependency}" bash "${ROOT_DIR}/scripts/verify-macos-package.sh" "${APP}" > "${FIXTURE_DIR}/log" 2>&1; then
        echo "Expected rejection for developer-machine dependency ${dependency}" >&2; exit 1
    fi
done
/usr/libexec/PlistBuddy -c 'Set :LSMinimumSystemVersion 10.15' "${PLIST}"
if bash "${ROOT_DIR}/scripts/verify-macos-package.sh" "${APP}" > "${FIXTURE_DIR}/log" 2>&1; then
    echo "Expected rejection for understated minimum OS." >&2; exit 1
fi
/usr/libexec/PlistBuddy -c 'Set :LSMinimumSystemVersion 11.0' "${PLIST}"
/usr/libexec/PlistBuddy -c 'Delete :NSMicrophoneUsageDescription' "${PLIST}"
if bash "${ROOT_DIR}/scripts/verify-macos-package.sh" "${APP}" > "${FIXTURE_DIR}/log" 2>&1; then
    echo "Expected rejection for missing microphone purpose string." >&2; exit 1
fi
echo 'Standalone audit fixtures passed: moved bundle, system dependencies, developer-machine dylibs, invalid signature, OS mismatch, and missing microphone purpose.'
