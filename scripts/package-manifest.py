#!/usr/bin/env python3
"""Describe exact candidate artifacts; do not infer release acceptance from hashes."""
import argparse
import hashlib
import json
import pathlib
import plistlib
import subprocess
import tempfile


def digest(path):
    result = hashlib.sha256()
    with path.open('rb') as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b''):
            result.update(chunk)
    return result.hexdigest()


def create_manifest(app, dmg, revision, dirty):
    with (app / 'Contents/Info.plist').open('rb') as source:
        info = plistlib.load(source)
    executable = app / 'Contents/MacOS' / info['CFBundleExecutable']
    architectures = subprocess.check_output(['lipo', '-archs', str(executable)], text=True).strip().split()
    if not architectures:
        raise ValueError('Executable has no Mach-O architectures')
    return {
        'schema_version': 1,
        'source_revision': revision,
        'source_dirty': dirty,
        'app_version': info['CFBundleShortVersionString'],
        'minimum_macos': info['LSMinimumSystemVersion'],
        'architectures': architectures,
        'binary_sha256': digest(executable),
        'dmg_filename': dmg.name,
        'dmg_sha256': digest(dmg),
        'acceptance': 'Candidate for acceptance testing; signing, notarization, hardware, Gatekeeper, and microphone permission checks are not established by this manifest.',
    }


def write_manifest(output_path, manifest):
    output_path.parent.mkdir(parents=True, exist_ok=True)
    temporary = None
    try:
        with tempfile.NamedTemporaryFile(mode='w', dir=output_path.parent, prefix='.manifest-', delete=False) as output:
            temporary = pathlib.Path(output.name)
            json.dump(manifest, output, indent=2)
            output.write('\n')
        temporary.replace(output_path)
    finally:
        if temporary is not None:
            temporary.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('app', type=pathlib.Path)
    parser.add_argument('dmg', type=pathlib.Path)
    parser.add_argument('output', type=pathlib.Path)
    parser.add_argument('--revision', required=True, help='Commit captured before the build')
    parser.add_argument('--dirty', action='store_true', help='Build included uncommitted source changes')
    args = parser.parse_args()
    manifest = create_manifest(args.app, args.dmg, args.revision, args.dirty)
    write_manifest(args.output, manifest)
    print(args.output)


if __name__ == '__main__':
    main()
