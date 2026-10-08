#!/usr/bin/env bash
# Exercise packaging transactions with fake build/DMG tools; no release compile.
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURE_DIR="$(mktemp -d "${TMPDIR:-/tmp}/batcherbird-package-test.XXXXXX")"
trap 'rm -rf "${FIXTURE_DIR}"' EXIT
mkdir -p "${FIXTURE_DIR}/bin" "${FIXTURE_DIR}/target" "${FIXTURE_DIR}/output/Batcherbird.app"
cat > "${FIXTURE_DIR}/bin/cargo" <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${FIXTURE_FAIL_BUILD:-0}" == 1 ]]; then exit 17; fi
mkdir -p "${CARGO_TARGET_DIR}/release"
printf 'fixture binary\n' > "${CARGO_TARGET_DIR}/release/batcherbird-vizia"
TOOL
cat > "${FIXTURE_DIR}/bin/hdiutil" <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${FIXTURE_FAIL_DMG:-0}" == 1 ]]; then exit 18; fi
if [[ "${FIXTURE_EMPTY_DMG:-0}" == 1 ]]; then exit 0; fi
printf 'fixture dmg\n' > "${!#}"
TOOL
cat > "${FIXTURE_DIR}/bin/mv" <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
if [[ "${FIXTURE_FAIL_INSTALL:-0}" == 1 && "$1" == */.batcherbird-package.*/Batcherbird.dmg ]]; then exit 19; fi
exec /bin/mv "$@"
TOOL
cat > "${FIXTURE_DIR}/bin/otool" <<'TOOL'
#!/usr/bin/env bash
if [[ "${FIXTURE_FAIL_OTOOL:-0}" == 1 ]]; then exit 1; fi
if [[ "${FIXTURE_LEGACY_OTOOL:-0}" == 1 ]]; then
    printf 'cmd LC_VERSION_MIN_MACOSX\n version 10.15.0\n sdk 13.1\n'
else
    printf 'cmd LC_BUILD_VERSION\n minos 13.2\n sdk 26.2\ncmd LC_BUILD_VERSION\n minos 12.0\n sdk 26.2\n'
fi
TOOL
chmod +x "${FIXTURE_DIR}/bin/cargo" "${FIXTURE_DIR}/bin/hdiutil" "${FIXTURE_DIR}/bin/mv" "${FIXTURE_DIR}/bin/otool"
cat > "${FIXTURE_DIR}/bin/codesign" <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
printf 'codesign %s\n' "$*" >> "${FIXTURE_COMMAND_LOG}"
if [[ "${FIXTURE_FAIL_SIGN:-0}" == 1 ]]; then exit 20; fi
TOOL
cat > "${FIXTURE_DIR}/bin/ditto" <<'TOOL'
#!/usr/bin/env bash
printf 'ditto %s\n' "$*" >> "${FIXTURE_COMMAND_LOG}"
printf 'fixture archive\n' > "${!#}"
TOOL
cat > "${FIXTURE_DIR}/bin/xcrun" <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
printf 'xcrun %s\n' "$*" >> "${FIXTURE_COMMAND_LOG}"
if [[ "${FIXTURE_FAIL_NOTARY:-0}" == 1 && "$1" == notarytool ]]; then exit 21; fi
if [[ "${FIXTURE_FAIL_STAPLE:-0}" == 1 && "$1" == stapler && "$2" == staple ]]; then exit 22; fi
TOOL
chmod +x "${FIXTURE_DIR}/bin/codesign" "${FIXTURE_DIR}/bin/ditto" "${FIXTURE_DIR}/bin/xcrun"
export FIXTURE_COMMAND_LOG="${FIXTURE_DIR}/commands.log"
# Isolate fixture runs from any real developer-account configuration.
unset BATCHERBIRD_SIGN_IDENTITY BATCHERBIRD_NOTARY_PROFILE
export PATH="${FIXTURE_DIR}/bin:${PATH}"
export CARGO_TARGET_DIR="${FIXTURE_DIR}/target"
export BATCHERBIRD_DIST_DIR="${FIXTURE_DIR}/output"
printf 'unrelated\n' > "${BATCHERBIRD_DIST_DIR}/notes.txt"
printf 'original app\n' > "${BATCHERBIRD_DIST_DIR}/Batcherbird.app/original.txt"
printf 'original dmg\n' > "${BATCHERBIRD_DIST_DIR}/Batcherbird.dmg"
assert_original() {
    [[ "$(cat "${BATCHERBIRD_DIST_DIR}/Batcherbird.app/original.txt")" == "original app" ]]
    [[ "$(cat "${BATCHERBIRD_DIST_DIR}/Batcherbird.dmg")" == "original dmg" ]]
    [[ "$(cat "${BATCHERBIRD_DIST_DIR}/notes.txt")" == "unrelated" ]]
    [[ -z "$(find "${BATCHERBIRD_DIST_DIR}" -maxdepth 1 -name '.batcherbird-package.*' -print)" ]]
}
for failure in FIXTURE_FAIL_BUILD FIXTURE_FAIL_DMG FIXTURE_EMPTY_DMG FIXTURE_FAIL_INSTALL FIXTURE_FAIL_SIGN; do
    if env "${failure}=1" bash "${ROOT_DIR}/scripts/package-macos.sh" > "${FIXTURE_DIR}/log" 2>&1; then
        echo "Expected packaging failure: ${failure}" >&2; exit 1
    fi
    assert_original
done
for failure in FIXTURE_FAIL_SIGN FIXTURE_FAIL_NOTARY FIXTURE_FAIL_STAPLE; do
    if env "${failure}=1" BATCHERBIRD_SIGN_IDENTITY='Developer ID Application: Fixture' \
        BATCHERBIRD_NOTARY_PROFILE=fixture bash "${ROOT_DIR}/scripts/package-macos.sh" > "${FIXTURE_DIR}/log" 2>&1; then
        echo "Expected signing/notarization failure: ${failure}" >&2; exit 1
    fi
    assert_original
done
if BATCHERBIRD_NOTARY_PROFILE=fixture bash "${ROOT_DIR}/scripts/package-macos.sh" > "${FIXTURE_DIR}/log" 2>&1; then
    echo "Expected rejection for notarization without signing." >&2; exit 1
fi
assert_original
if BATCHERBIRD_SIGN_IDENTITY=- BATCHERBIRD_NOTARY_PROFILE=fixture \
    bash "${ROOT_DIR}/scripts/package-macos.sh" > "${FIXTURE_DIR}/log" 2>&1; then
    echo "Expected rejection for ad hoc notarization." >&2; exit 1
fi
assert_original
: > "${FIXTURE_COMMAND_LOG}"
bash "${ROOT_DIR}/scripts/package-macos.sh" > "${FIXTURE_DIR}/log" 2>&1
[[ "$(cat "${BATCHERBIRD_DIST_DIR}/Batcherbird.app/Contents/MacOS/Batcherbird")" == "fixture binary" ]]
[[ "$(cat "${BATCHERBIRD_DIST_DIR}/Batcherbird.dmg")" == "fixture dmg" ]]
cmp "${ROOT_DIR}/LICENSE" "${BATCHERBIRD_DIST_DIR}/Batcherbird.app/Contents/Resources/licenses/Batcherbird-AGPL.txt"
cmp "${ROOT_DIR}/vendor/vizia_core/LICENSE" "${BATCHERBIRD_DIST_DIR}/Batcherbird.app/Contents/Resources/licenses/vizia_core-MIT.txt"
[[ "$(cat "${BATCHERBIRD_DIST_DIR}/notes.txt")" == "unrelated" ]]
[[ ! -e "${BATCHERBIRD_DIST_DIR}/Batcherbird.app/original.txt" ]]
[[ -z "$(find "${BATCHERBIRD_DIST_DIR}" -maxdepth 1 -name '.batcherbird-package.*' -print)" ]]
# Local signatures seal resources without accounts, hardened runtime, or uploads.
[[ "$(grep -c '^codesign ' "${FIXTURE_COMMAND_LOG}")" == 2 ]]
grep -q -- '--force --sign - --timestamp=none' "${FIXTURE_COMMAND_LOG}"
grep -q -- '--verify --deep --strict' "${FIXTURE_COMMAND_LOG}"
if grep -qE 'xcrun|ditto|--options runtime|--entitlements|--timestamp ' "${FIXTURE_COMMAND_LOG}"; then
    echo "Local packaging unexpectedly invoked developer-account signing or upload steps." >&2; exit 1
fi
BATCHERBIRD_SIGN_IDENTITY='Developer ID Application: Fixture' BATCHERBIRD_NOTARY_PROFILE=fixture \
    bash "${ROOT_DIR}/scripts/package-macos.sh" > "${FIXTURE_DIR}/log" 2>&1
[[ "$(grep -c 'notarytool submit' "${FIXTURE_COMMAND_LOG}")" == 2 ]]
[[ "$(grep -c 'stapler staple' "${FIXTURE_COMMAND_LOG}")" == 2 ]]
grep -q -- '--options runtime --timestamp --entitlements' "${FIXTURE_COMMAND_LOG}"
# App ticket must be attached before DMG signing/submission begins.
APP_STAPLE_LINE="$(grep -n 'stapler staple .*Batcherbird.app' "${FIXTURE_COMMAND_LOG}" | cut -d: -f1)"
DMG_SUBMIT_LINE="$(grep -n 'notarytool submit .*Batcherbird.dmg' "${FIXTURE_COMMAND_LOG}" | cut -d: -f1)"
[[ "${APP_STAPLE_LINE}" -lt "${DMG_SUBMIT_LINE}" ]]
minimum_version() {
    awk '/<key>LSMinimumSystemVersion/ { getline; gsub(/.*<string>|<\/string>.*/, ""); print }' "${BATCHERBIRD_DIST_DIR}/Batcherbird.app/Contents/Info.plist"
}
[[ "$(minimum_version)" == "13.2" ]]
FIXTURE_LEGACY_OTOOL=1 bash "${ROOT_DIR}/scripts/package-macos.sh" > "${FIXTURE_DIR}/log" 2>&1
[[ "$(minimum_version)" == "10.15.0" ]]
FIXTURE_FAIL_OTOOL=1 bash "${ROOT_DIR}/scripts/package-macos.sh" > "${FIXTURE_DIR}/log" 2>&1
[[ "$(minimum_version)" == "11.0" ]]
for invalid_output in / "${ROOT_DIR}" ""; do
    if BATCHERBIRD_DIST_DIR="${invalid_output}" bash "${ROOT_DIR}/scripts/package-macos.sh" > "${FIXTURE_DIR}/log" 2>&1; then
        echo "Expected rejection for output root: ${invalid_output}" >&2; exit 1
    fi
done
echo "Packaging fixtures passed: success, build/DMG/installation failures, preservation, root guards, Mach-O minimum OS metadata, opt-in signing/notarization, and ticket order."
