# macOS installation and distribution

Local packages are suitable for developer acceptance testing. They are not
Developer ID signed or notarized unless those steps are explicitly configured.
The tag-based GitHub release workflow currently has no signing credentials; a
successful release upload does not turn that candidate into a notarized release.

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
permissions, Mach-O format, system-only dynamic library dependencies, and DMG
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
staple failures, preserving previous artifacts. They verify signing is absent by
default and the app is stapled before DMG submission. These use fake tools and
prove transaction behavior, not Apple's acceptance of a signature.

A previously built local candidate copied to `/private/tmp` passed metadata,
system-dependency, and DMG integrity inspection. Native launch from a clean
account and real signing/notarization were not performed. CI now audits a copy of
each newly built package outside the checkout as well.
