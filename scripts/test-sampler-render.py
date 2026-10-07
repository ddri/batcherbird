#!/usr/bin/env python3
"""Regression checks for the audio checker, not sampler compatibility evidence."""
import importlib.util
import math
from pathlib import Path
import struct
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("checker", Path(__file__).with_name("check-sampler-render.py"))
checker = importlib.util.module_from_spec(spec)
spec.loader.exec_module(checker)


class RenderCheckerTests(unittest.TestCase):
    def check_signal(self, *, pitch=60, root=60, upper=False, gain=-0.35):
        rate = 8000
        frequency = 440 * 2 ** ((pitch - 69) / 12)
        marker = {60: 0.1, 63: 0.2, 66: 0.3}[root]
        signal = [0.2 * (math.sin(2 * math.pi * frequency * i / rate)
                        + marker * math.sin(6 * math.pi * frequency * i / rate)
                        + (0.5 if upper else 0) * math.sin(4 * math.pi * frequency * i / rate))
                  for i in range(rate)]
        probe = dict(start_seconds="0", note="60", velocity="64", root="60", layer="low")
        return checker.measure(signal, [gain * x for x in signal], rate, probe)["errors"]

    def test_correct_signal_passes(self):
        self.assertEqual(self.check_signal(), [])

    def test_wrong_pitch_is_rejected(self):
        self.assertTrue(self.check_signal(pitch=61))

    def test_wrong_root_is_rejected(self):
        self.assertIn("wrong sampled root or overlapping different roots", self.check_signal(root=63))

    def test_wrong_layer_is_rejected(self):
        self.assertIn("wrong velocity layer or overlapping different layers", self.check_signal(upper=True))

    def test_channel_swap_or_mono_is_rejected(self):
        self.assertIn("stereo channel identity changed", self.check_signal(gain=1.0))

    def test_float_wav_decoding_and_nonfinite_rejection(self):
        def wav(value, incomplete_frame=False):
            fmt = struct.pack("<HHIIHH", 3, 2, 48000, 384000, 8, 32)
            data = struct.pack("<ff", value, -0.35 * value)
            if incomplete_frame:
                data = data[:4]
            body = b"WAVEfmt " + struct.pack("<I", len(fmt)) + fmt + b"data" + struct.pack("<I", len(data)) + data
            return b"RIFF" + struct.pack("<I", len(body)) + body
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder) / "render.wav"
            path.write_bytes(wav(0.25))
            rate, left, right = checker.read_audio(path)
            self.assertEqual(rate, 48000)
            self.assertEqual(left, [0.25])
            self.assertAlmostEqual(right[0], -0.0875)
            path.write_bytes(wav(float("nan")))
            with self.assertRaisesRegex(ValueError, "non-finite"):
                checker.read_audio(path)
            path.write_bytes(wav(0.25, incomplete_frame=True))
            with self.assertRaisesRegex(ValueError, "incomplete stereo frame"):
                checker.read_audio(path)

    def test_missing_late_probe_is_rejected(self):
        probe = dict(start_seconds="88", note="67", velocity="127", root="silent", layer="silent")
        self.assertEqual(checker.measure([0.0], [0.0], 48000, probe)["errors"],
                         ["render is shorter than this probe"])

    def test_outside_key_sounding_only_in_right_channel_is_rejected(self):
        probe = dict(start_seconds="0", note="59", velocity="127", root="silent", layer="silent")
        result = checker.measure([0.0] * 8000, [0.25] * 8000, 8000, probe)
        self.assertEqual(result["errors"], ["out-of-range key sounded"])
        self.assertEqual(result["rms"], 0.0)
        self.assertEqual(result["right_rms"], 0.25)

    def test_truncated_right_channel_is_rejected_for_silent_and_active_keys(self):
        for layer, root in [("silent", "silent"), ("low", "60")]:
            with self.subTest(layer=layer):
                probe = dict(start_seconds="0", note="60", velocity="64", root=root, layer=layer)
                self.assertEqual(checker.measure([0.0] * 8000, [0.0] * 1600, 8000, probe)["errors"],
                                 ["stereo channels have different lengths"])

    def test_nonfinite_audio_in_either_channel_is_rejected_before_silence_check(self):
        probe = dict(start_seconds="0", note="59", velocity="127", root="silent", layer="silent")
        for channel in (0, 1):
            for invalid in (float("nan"), float("inf")):
                with self.subTest(channel=channel, invalid=invalid):
                    channels = [[0.0] * 8000, [0.0] * 8000]
                    channels[channel][1600] = invalid
                    self.assertEqual(checker.measure(*channels, 8000, probe)["errors"],
                                     ["render contains non-finite audio"])


if __name__ == "__main__":
    unittest.main()
