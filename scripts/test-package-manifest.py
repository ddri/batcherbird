#!/usr/bin/env python3
import importlib.util
import hashlib
import pathlib
import plistlib
import tempfile
import unittest
from unittest import mock

spec = importlib.util.spec_from_file_location('package_manifest', pathlib.Path(__file__).with_name('package-manifest.py'))
manifest_module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(manifest_module)


class ManifestTests(unittest.TestCase):
    def test_manifest_identifies_exact_artifacts_and_dirty_source(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            app = root / 'Moved app.app'
            executable = app / 'Contents/MacOS/Batcherbird'
            executable.parent.mkdir(parents=True)
            executable.write_bytes(b'first binary')
            with (app / 'Contents/Info.plist').open('wb') as target:
                plistlib.dump({'CFBundleExecutable': 'Batcherbird', 'CFBundleShortVersionString': '0.1.0', 'LSMinimumSystemVersion': '11.0'}, target)
            dmg = root / 'Batcherbird.dmg'
            dmg.write_bytes(b'first dmg')
            with mock.patch.object(manifest_module.subprocess, 'check_output', return_value='arm64 x86_64\n'):
                first = manifest_module.create_manifest(app, dmg, 'abc123', True)
                self.assertEqual(first['source_revision'], 'abc123')
                self.assertTrue(first['source_dirty'])
                self.assertEqual(first['architectures'], ['arm64', 'x86_64'])
                self.assertEqual(first['binary_sha256'], hashlib.sha256(b'first binary').hexdigest())
                self.assertEqual(first['dmg_sha256'], hashlib.sha256(b'first dmg').hexdigest())
                dmg.write_bytes(b'changed dmg')
                second = manifest_module.create_manifest(app, dmg, 'abc123', False)
                self.assertNotEqual(first['dmg_sha256'], second['dmg_sha256'])
                self.assertEqual(first['binary_sha256'], second['binary_sha256'])
                self.assertFalse(second['source_dirty'])

    def test_partial_write_preserves_existing_manifest_and_removes_temporary(self):
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            output = root / 'manifest.json'
            output.write_text('previous evidence')

            def fail_after_write(_manifest, target, **_kwargs):
                target.write('partial new evidence')
                raise OSError('disk full')

            with mock.patch.object(manifest_module.json, 'dump', side_effect=fail_after_write):
                with self.assertRaises(OSError):
                    manifest_module.write_manifest(output, {'revision': 'new'})
            self.assertEqual(output.read_text(), 'previous evidence')
            self.assertEqual(list(root.glob('.manifest-*')), [])

    def test_missing_image_fails_instead_of_producing_incomplete_evidence(self):
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaises(FileNotFoundError):
                manifest_module.create_manifest(pathlib.Path(directory) / 'missing.app', pathlib.Path(directory) / 'missing.dmg', 'abc123', False)


if __name__ == '__main__':
    unittest.main()
