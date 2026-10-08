# macOS installation and distribution

Local packages are suitable for developer acceptance testing. They are not
Developer ID signed or notarized unless those steps are explicitly configured.
The tag-based GitHub release workflow currently has no signing credentials; a
successful release upload does not turn that candidate into a notarized release.
Local packaging seals the complete app bundle with an ad hoc signature that needs
no account or credentials. This verifies bundle integrity without establishing
a trusted developer identity.

## Build and inspect a local candidate

```sh
./scripts/package-macos.sh
./scripts/verify-macos-package.sh dist/Batcherbird.app dist/Batcherbird.dmg
```

`BATCHERBIRD_DIST_DIR` selects a different output directory. Packaging replaces
only `Batcherbird.app` and `Batcherbird.dmg` after both are ready. Build, signing,
notarization, and installation failures preserve the previous artifacts and
unrelated files. The default package uses the build machine's architecture; it
makes no claim of universal Intel/Apple Silicon compatibility. The minimum OS in
the plist is derived from the executable.

The audit validates plist metadata, the microphone purpose string, executable
permissions, Mach-O format, complete bundle signature validity, system-only dynamic
library dependencies, and DMG
integrity. UI styling is embedded in the executable. Copy the bundle outside the
repository and run this audit again to check that the package needs no checkout
files. This inspection does not establish native launch, playback, recording,
or microphone permission behavior.

## Opt-in Developer ID signing and notarization

Use a Developer ID Application certificate already installed in your Keychain.
Set up a `notarytool` Keychain profile following Apple's documentation; do not
put Apple account passwords, certificates, or private keys in repository files.

```sh
BATCHERBIRD_SIGN_IDENTITY='Developer ID Application: Your Name (TEAMID)' \
BATCHERBIRD_NOTARY_PROFILE='batcherbird-notary' \
./scripts/package-macos.sh

BATCHERBIRD_REQUIRE_NOTARIZED=1 \
./scripts/verify-macos-package.sh dist/Batcherbird.app dist/Batcherbird.dmg
```

Specifying only the signing identity signs the app and DMG without uploading.
Specifying a notary profile explicitly enables uploads to Apple. Notarization
requires a signing identity; ad hoc identity `-` is rejected for notarization.
These opt-in operations use secure timestamps and need network access.

The app is signed with hardened runtime and the audio-input entitlement in
`scripts/macos-entitlements.plist`. No debugging, unsigned-memory, or disabled
library-validation exceptions are granted. The script verifies the app signature,
submits its ZIP to Apple, staples and validates its ticket, then constructs and
signs the DMG, submits the DMG, and staples and validates that ticket. Both the app
and image carry tickets for offline installation. Failures stop packaging before
replacing the previous candidate. Actual hardened-runtime audio capture and
Developer ID/Gatekeeper validation remain acceptance gates until exercised with
real credentials and hardware.

Apple's authoritative references:

- [Notarizing macOS software before distribution](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution)
- [Customizing the notarization workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow)
- [Resolving common notarization issues](https://developer.apple.com/documentation/security/resolving-common-notarization-issues)

## Installation acceptance still required

Use a fresh macOS account and the final downloaded DMG, retaining its normal
quarantine metadata. Open the image in Finder, drag the app into Applications,
launch it there, and confirm normal Gatekeeper and microphone permission prompts.
Check recording permission granted and denied, launch after a move/rename,
preferences and session recovery, playback, and uninstall/reinstall behavior.
Repeat on each advertised architecture and minimum supported OS. Record the
build SHA, OS/architecture, signing identity, and results. Do not remove
quarantine or disable Gatekeeper to claim this acceptance passed.

## Automated evidence (October 7, 2026)

Packaging fixtures passed unsigned success and build/DMG/install/sign/notary/
staple failures, preserving previous artifacts. They verify Developer ID signing is absent by
default and the app is stapled before DMG submission. These use fake tools and
prove transaction behavior, not Apple's acceptance of a signature.

A previously built local candidate copied to `/private/tmp` passed metadata,
system-dependency, and DMG integrity inspection. Native launch from a clean
account and real signing/notarization were not performed. CI now audits a copy of
each newly built package outside the checkout as well.

## Downloadable CI acceptance candidates

The **Package macOS App & DMG** CI job uploads a candidate artifact for each
successful pull-request or main build. On GitHub, open the Actions run and download
`batcherbird-macos-candidate-<source-revision>` from its Artifacts section. The
artifact is retained for 14 days and contains `Batcherbird.dmg` plus
`Batcherbird-manifest.json`. It is an ad hoc signed acceptance candidate, not a
published release or evidence that hardware/Gatekeeper tests passed.

The JSON records the exact checked-out commit (a PR run can use GitHub's merge
commit), whether uncommitted changes were included, application version, minimum
macOS, executable architectures, and SHA-256 hashes of the executable and DMG.
Check the DMG hash before testing and attach this manifest to acceptance results.
This makes a hardware report attributable to a specific candidate even when a
new CI build replaces it. CI requires a clean checkout before compilation.

For a local build, capture the revision **before** compilation, and use `--dirty`
if any tracked or untracked source changes are included:

```sh
candidate_revision="$(git rev-parse HEAD)"
./scripts/package-macos.sh
python3 scripts/package-manifest.py dist/Batcherbird.app dist/Batcherbird.dmg \
  dist/Batcherbird-manifest.json --revision "$candidate_revision" --dirty
```

Omit `--dirty` only for a clean-source build. The manifest describes supplied
artifacts; it cannot retrospectively prove which source was compiled. Preserve
the original manifest alongside results rather than replacing a dirty baseline
with a clean revision until that clean revision has actually been rebuilt.

## Fresh local package inspection (October 8, 2026)

A release package was built with Rust 1.99.0 from baseline commit
`5ef5ac2bbc86f811fc501734bd8c03fd2e33bd7d` plus the uncommitted October 8
acceptance/diagnostics changes. Its manifest correctly records `source_dirty: true`.
This is version 0.1.0, arm64 only, with minimum macOS 11.0 confirmed against the
executable's load commands.

The newly generated DMG passed checksum verification and was mounted read-only.
Its Applications shortcut targeted `/Applications`. The bundled app was copied
to a unique directory under `/private/tmp`, audited outside the checkout, and
the image was detached. The copied executable matched the packaged executable
hash. Plist metadata, microphone purpose, permissions, minimum OS, and
system-only dylib dependency checks passed.

- Executable SHA-256: `879174a325a07bfd8cf38a4d85fa7a70ea345ff39e16e3fa66462589197bbe1b`
- DMG SHA-256: `f8392655e5d693417dc01112114847b6c35666e6e9d8982edc0338cffef178f9`

The first inspection found the executable's linker signature was insufficient
for the app bundle: sealed resources were absent. Packaging now applies an ad hoc
signature after assembling the plist and resources, then verifies the complete
bundle with `codesign --verify --deep --strict`. Default local builds use no
Developer ID identity, hardened-runtime entitlement, timestamp service, or
notarization upload. Credentialed signing remains opt-in.

The candidate was rebuilt and audited after this fix; the final artifact hashes
are listed above. The final copied bundle passed complete signature verification,
including its Info.plist and resource seal; its executable matched the app
embedded in the read-only mounted DMG. The image was detached afterward. Neither the locally
generated DMG nor copied app had a quarantine attribute. This inspection cannot
validate Developer ID trust, downloaded-app Gatekeeper behavior, notarization,
microphone grants, or fresh-account installation. Native launch from this
outside-checkout copy was not verified. Those manual gates remain open.
