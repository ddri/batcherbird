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
for failure in FIXTURE_FAIL_BUILD FIXTURE_FAIL_DMG FIXTURE_EMPTY_DMG FIXTURE_FAIL_INSTALL; do
    if env "${failure}=1" bash "${ROOT_DIR}/scripts/package-macos.sh" > "${FIXTURE_DIR}/log" 2>&1; then
        echo "Expected packaging failure: ${failure}" >&2; exit 1
    fi
    assert_original
done
bash "${ROOT_DIR}/scripts/package-macos.sh" > "${FIXTURE_DIR}/log" 2>&1
[[ "$(cat "${BATCHERBIRD_DIST_DIR}/Batcherbird.app/Contents/MacOS/Batcherbird")" == "fixture binary" ]]
[[ "$(cat "${BATCHERBIRD_DIST_DIR}/Batcherbird.dmg")" == "fixture dmg" ]]
[[ "$(cat "${BATCHERBIRD_DIST_DIR}/notes.txt")" == "unrelated" ]]
[[ ! -e "${BATCHERBIRD_DIST_DIR}/Batcherbird.app/original.txt" ]]
[[ -z "$(find "${BATCHERBIRD_DIST_DIR}" -maxdepth 1 -name '.batcherbird-package.*' -print)" ]]
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
echo "Packaging fixtures passed: success, build/DMG/installation failures, preservation, root guards, and Mach-O minimum OS metadata."
