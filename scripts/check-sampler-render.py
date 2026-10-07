#!/usr/bin/env python3
"""Check an independent sampler's boundary-sweep render using stdlib only.

This deliberately measures rendered audio, not the exported preset text.
Expected timings are those of generate_sampler_acceptance's MIDI file.
"""
import argparse
import csv
import json
import math
import struct
import wave
from pathlib import Path


def read_audio(path):
    # Python's wave module rejects IEEE float, used by the optional float
    # sfizz renderer to avoid quantizing very low velocity probes to DC.
    raw = path.read_bytes()
    if raw[:4] != b"RIFF" or raw[8:12] != b"WAVE":
        raise ValueError("Expected RIFF WAVE file")
    offset, format_info, audio = 12, None, None
    while offset + 8 <= len(raw):
        name, size = struct.unpack_from("<4sI", raw, offset)
        chunk = raw[offset + 8:offset + 8 + size]
        if len(chunk) != size:
            raise ValueError("Truncated WAV chunk")
        if name == b"fmt ":
            format_info = struct.unpack_from("<HHIIHH", chunk)
        elif name == b"data":
            audio = chunk
        offset += 8 + size + (size & 1)
    if format_info is None or audio is None:
        raise ValueError("WAV lacks format/audio chunks")
    encoding, channels, rate, _, _, bits = format_info
    if encoding == 3:
        if channels != 2 or bits != 32:
            raise ValueError("Expected stereo 32-bit float WAV")
        values = [v[0] for v in struct.iter_unpack("<f", audio)]
        if not all(math.isfinite(value) for value in values):
            raise ValueError("Render contains non-finite audio")
        return rate, values[::2], values[1::2]
    del raw, audio
    with wave.open(str(path), "rb") as reader:
        if reader.getnchannels() != 2 or reader.getsampwidth() not in (2, 3, 4):
            raise ValueError("Expected stereo PCM WAV (16/24/32 bit); export with no effects")
        width = reader.getsampwidth()
        rate = reader.getframerate()
        data = reader.readframes(reader.getnframes())
    if width == 2:
        values = [v[0] / 32768.0 for v in struct.iter_unpack("<h", data)]
    elif width == 4:
        values = [v[0] / 2147483648.0 for v in struct.iter_unpack("<i", data)]
    else:
        values = [int.from_bytes(data[i:i + 3], "little", signed=True) / 8388608.0
                  for i in range(0, len(data), 3)]
    return rate, values[::2], values[1::2]


def magnitude(values, rate, frequency):
    # Hann-windowed correlation avoids non-integral-period leakage.
    real = imaginary = 0.0
    for index, value in enumerate(values):
        window = 0.5 - 0.5 * math.cos(2.0 * math.pi * index / (len(values) - 1))
        phase = 2.0 * math.pi * frequency * index / rate
        real += value * window * math.cos(phase)
        imaginary += value * window * math.sin(phase)
    return math.hypot(real, imaginary)


def measure(left, right, rate, probe):
    start = int((float(probe["start_seconds"]) + 0.15) * rate)
    end = int((float(probe["start_seconds"]) + 0.35) * rate)
    a, b = left[start:end], right[start:end]
    errors = []
    if len(a) != end - start:
        return {"errors": ["render is shorter than this probe"]}
    power = sum(x * x for x in a) / len(a)
    rms = math.sqrt(power)
    if probe["layer"] == "silent":
        if rms > 0.00000001:
            errors.append("out-of-range key sounded")
        return {"rms": rms, "errors": errors}
    if rms < 0.000000001:
        return {"rms": rms, "errors": ["in-range key was silent or below measurable PCM level"]}
    frequency = 440.0 * 2.0 ** ((int(probe["note"]) - 69) / 12)
    peaks = [(magnitude(a, rate, frequency * 2.0 ** (cents / 1200)), cents)
             for cents in range(-40, 41, 5)]
    fundamental, cents = max(peaks)
    harmonic_ratio = magnitude(a, rate, frequency * 2) / fundamental
    if abs(cents) > 10:
        errors.append("pitch differs by more than 10 cents")
    expected_ratio = 0.5 if probe["layer"] == "high" else 0.0
    root_ratio = magnitude(a, rate, frequency * 3) / fundamental
    expected_root_ratio = {"60": 0.1, "63": 0.2, "66": 0.3}[probe["root"]]
    if abs(harmonic_ratio - expected_ratio) > 0.08:
        errors.append("wrong velocity layer or overlapping different layers")
    if abs(root_ratio - expected_root_ratio) > 0.03:
        errors.append("wrong sampled root or overlapping different roots")
    right_gain = sum(x * y for x, y in zip(a, b)) / sum(x * x for x in a)
    residual = math.sqrt(sum((y + 0.35 * x) ** 2 for x, y in zip(a, b)) / len(a)) / rms
    if abs(right_gain + 0.35) > 0.03 or residual > 0.05:
        errors.append("stereo channel identity changed")
    return {"rms": rms, "pitch_error_cents": cents, "harmonic_ratio": harmonic_ratio,
            "root_harmonic_ratio": root_ratio,
            "right_gain": right_gain, "stereo_residual": residual, "errors": errors}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("fixture", type=Path)
    parser.add_argument("render", type=Path)
    parser.add_argument("--report", type=Path)
    parser.add_argument("--voice-log", type=Path, help="sfizz_render --log output to check single-voice triggering")
    args = parser.parse_args()
    rate, left, right = read_audio(args.render)
    with (args.fixture / "probes.csv").open(newline="") as source:
        probes = list(csv.DictReader(source))
    results = [dict(probe, **measure(left, right, rate, probe)) for probe in probes]
    if args.voice_log:
        with args.voice_log.open(newline="") as source:
            blocks = list(csv.DictReader(source))
        frame = 0
        for block in blocks:
            midpoint = (frame + int(block["NumSamples"]) / 2) / rate
            frame += int(block["NumSamples"])
            for result in results:
                start = float(result["start_seconds"])
                if start + 0.15 <= midpoint <= start + 0.35:
                    voices = int(block["NumVoices"])
                    expected = 0 if result["layer"] == "silent" else 1
                    result.setdefault("active_voices", []).append(voices)
                    if voices != expected and "unexpected active voice count" not in result["errors"]:
                        result["errors"].append("unexpected active voice count")
        for result in results:
            if not result.get("active_voices"):
                result["errors"].append("voice log did not cover this probe")
    report = {"render": str(args.render.resolve()), "sample_rate": rate,
              "passed": sum(not r["errors"] for r in results), "total": len(results),
              "results": results}
    if args.report:
        args.report.write_text(json.dumps(report, indent=2) + "\n")
    print(f"{report['passed']}/{report['total']} rendered probes passed at {rate} Hz")
    for result in results:
        if result["errors"]:
            print(f"Note {result['note']} velocity {result['velocity']}: " + "; ".join(result["errors"]))
    raise SystemExit(0 if report["passed"] == report["total"] else 1)


if __name__ == "__main__":
    main()
