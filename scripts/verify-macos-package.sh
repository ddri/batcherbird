#!/usr/bin/env bash
# Inspect a copied bundle without launching UI or touching app preferences.
set -euo pipefail
APP_DIR="${1:?Usage: verify-macos-package.sh /path/to/Batcherbird.app [image.dmg]}"
PLIST="${APP_DIR}/Contents/Info.plist"
plutil -lint "${PLIST}"
plist_value() { /usr/libexec/PlistBuddy -c "Print :$1" "${PLIST}"; }
[[ "$(plist_value CFBundlePackageType)" == APPL ]]
[[ "$(plist_value CFBundleIdentifier)" == com.davidryan.batcherbird ]]
MICROPHONE_PURPOSE="$(plist_value NSMicrophoneUsageDescription)"
MINIMUM_OS="$(plist_value LSMinimumSystemVersion)"
[[ -n "${MICROPHONE_PURPOSE}" ]]
[[ "${MINIMUM_OS}" =~ ^[0-9]+(\.[0-9]+){1,2}$ ]]
EXECUTABLE="${APP_DIR}/Contents/MacOS/$(plist_value CFBundleExecutable)"
[[ -x "${EXECUTABLE}" ]]
file "${EXECUTABLE}" | grep -q 'Mach-O'
# The app statically embeds its UI stylesheet. Reject developer-machine dylibs;
# this application currently has no bundled dynamic frameworks or @rpath entries.
DEPENDENCIES="$(otool -L "${EXECUTABLE}")"
printf '%s\n' "${DEPENDENCIES}" | awk '/^[[:space:]]/ {
    if ($1 !~ /^\/System\/Library\// && $1 !~ /^\/usr\/lib\//) {
        print "Non-system dependency: " $1 > "/dev/stderr"; failed = 1
    }
} END { exit failed }'
if [[ -n "${2:-}" ]]; then hdiutil verify "$2"; fi
if [[ "${BATCHERBIRD_REQUIRE_NOTARIZED:-0}" == 1 ]]; then
    codesign --verify --deep --strict "${APP_DIR}"
    xcrun stapler validate "${APP_DIR}"
    spctl --assess --type execute --verbose=2 "${APP_DIR}"
    if [[ -n "${2:-}" ]]; then
        xcrun stapler validate "$2"
        spctl --assess --type open --context context:primary-signature --verbose=2 "$2"
    fi
fi
echo "Package metadata and standalone dependency audit passed. Native launch and microphone permission still require acceptance testing."
