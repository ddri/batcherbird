#!/usr/bin/env python3
"""Regression-test compiled SDK host preflight without installing a sampler.

Usage: python3 scripts/render-decent-vst3/test-cli.py /path/to/batcherbird_decent_vst3
"""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

HOST = Path(sys.argv.pop(1)).resolve() if len(sys.argv) > 1 else None


class HostPreflightTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="batcherbird-host-cli-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.sentinel = self.root / "existing.bin"
        self.sentinel.write_bytes(b"preserve this existing user file\x00\xff")
        self.probes = self.root / "probes.csv"
        self.probes.write_text("id,start_seconds,note,velocity,root,layer\n0,0,60,64,60,low\n")
        self.wav, self.trace = self.root / "new.wav", self.root / "new.csv"

    def reject(self, args, expected):
        before = {p: p.read_bytes() for p in self.root.iterdir() if p.is_file()}
        env = dict(os.environ, CFFIXED_USER_HOME=str(self.root / "isolated-user"))
        result = subprocess.run([str(HOST), str(self.root / "missing.vst3"), *map(str, args)],
                                env=env, text=True, capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        self.assertIn(expected, result.stderr)
        self.assertEqual({p: p.read_bytes() for p in self.root.iterdir() if p.is_file()}, before)

    def test_seed_existing_file(self):
        self.reject(["--seed", self.sentinel], "Refusing to overwrite state")

    def test_seed_extra_arguments_cannot_overwrite(self):
        for extra in ([], ["--nextafter"]):
            with self.subTest(extra=extra):
                self.reject(["--seed", self.sentinel, self.wav, self.trace, *extra],
                            "--seed mode requires exactly")

    def test_incomplete_render_mode(self):
        self.reject(["state.bin", self.sentinel], "Rendering requires")

    def test_unknown_option(self):
        self.reject(["state.bin", self.probes, self.wav, self.trace, "--typo"], "Unknown diagnostic")

    def test_render_existing_destinations(self):
        for wav, trace in ((self.sentinel, self.trace), (self.wav, self.sentinel)):
            with self.subTest(wav=wav, trace=trace):
                self.reject(["state.bin", self.probes, wav, trace], "Refusing to overwrite render")

    def test_colliding_normalized_destinations(self):
        aliases = [self.wav, self.root / "unused" / ".." / "new.wav"]
        (self.root / "alias").symlink_to(self.root, target_is_directory=True)
        aliases.append(self.root / "alias" / "new.wav")
        for alias in aliases:
            with self.subTest(alias=alias):
                self.reject(["state.bin", self.probes, self.wav, alias], "must have distinct paths")

    def test_case_alias_exclusive_creation(self):
        check = self.root / "casecheck"
        check.write_text("probe")
        insensitive = (self.root / "CASECHECK").exists()
        check.unlink()
        if not insensitive:
            self.skipTest("Filesystem has case-sensitive names")
        self.reject(["state.bin", self.probes, self.wav, self.root / "NEW.WAV"],
                    "Cannot exclusively create output")

    def test_failed_plugin_load_cleans_owned_reservations(self):
        self.reject(["state.bin", self.probes, self.wav, self.trace], "missing.vst3")
        self.reject(["--seed", self.root / "new-state.bin"], "missing.vst3")

    def test_invalid_numeric_probes_before_plugin_load(self):
        cases = [("0junk", "60", "64"), ("0", "60junk", "64"), ("0", "60", "64junk"),
                 ("nan", "60", "64"), ("inf", "60", "64"), ("-1", "60", "64"),
                 ("3601", "60", "64"), ("0", "-1", "64"), ("0", "128", "64"),
                 ("0", "60", "0"), ("0", "60", "128")]
        for start, note, velocity in cases:
            with self.subTest(start=start, note=note, velocity=velocity):
                self.probes.write_text(f"header\n0,{start},{note},{velocity},60,low\n")
                self.reject(["state.bin", self.probes, self.wav, self.trace], "Invalid MIDI probe")

    def test_empty_probes(self):
        self.probes.write_text("header\n")
        self.reject(["state.bin", self.probes, self.wav, self.trace], "No probes")


if __name__ == "__main__":
    if HOST is None or not HOST.is_file():
        sys.exit("Pass the compiled SDK host executable as the first argument")
    unittest.main()
