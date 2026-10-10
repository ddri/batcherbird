#!/usr/bin/env python3
"""Render exported presets through the actual DecentSampler VST3 engine.

Optional external dependencies: pedalboard 0.9.26, mido 1.3.3.
Tested player: official DecentSampler 1.36.1, macOS Apple Silicon.
No plugin installation, native editor, or audio device is needed.
"""
import argparse
import csv
import importlib.metadata
import json
import os
from pathlib import Path
import plistlib
import struct
import time
import xml.etree.ElementTree as ET

JUCE_ALPHABET = ".ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+"


def decode_memory_block(text):
    length, encoded = text.split(".", 1)
    output = bytearray(int(length))
    if len(encoded) * 6 < len(output) * 8:
        raise ValueError("Truncated JUCE memory block")
    for index, character in enumerate(encoded):
        value = JUCE_ALPHABET.index(character)
        for bit in range(6):
            offset = index * 6 + bit
            if offset < len(output) * 8 and value & (1 << bit):
                output[offset // 8] |= 1 << (offset % 8)
    return bytes(output)


def encode_memory_block(data):
    encoded = []
    for offset in range(0, len(data) * 8, 6):
        value = 0
        for bit in range(6):
            if offset + bit < len(data) * 8:
                value |= ((data[(offset + bit) // 8] >> ((offset + bit) % 8)) & 1) << bit
        encoded.append(JUCE_ALPHABET[value])
    return str(len(data)) + "." + "".join(encoded)


def unpack_xml(data):
    if data[:4] != b"VC2!" or len(data) < 8:
        raise ValueError("Player did not return the tested JUCE XML state format")
    length = struct.unpack_from("<I", data, 4)[0]
    if len(data) < 8 + length:
        raise ValueError("Truncated JUCE XML state")
    return ET.fromstring(data[8:8 + length]), data[8 + length + 1:]


def pack_xml(element):
    data = ET.tostring(element, encoding="utf-8", xml_declaration=True)
    return b"VC2!" + struct.pack("<I", len(data)) + data + b"\0"


def build_component_state(initial_state, preset):
    """Prepare the player's own component state for either independent host."""
    initial, private_data = unpack_xml(initial_state)
    exported = ET.parse(preset).getroot()
    if initial.tag != "DecentSampler" or exported.tag != "DecentSampler":
        raise ValueError("Expected DecentSampler preset/state XML")
    for key, value in initial.attrib.items():
        if key.startswith("_") and key not in exported.attrib:
            exported.set(key, value)
    exported.set("_samplePath", str(preset.parent))
    exported.set("_libraryUrl", str(preset))
    exported.set("_libraryCanonicalUrl", str(preset))
    exported.set("_presetName", preset.stem)
    return pack_xml(exported) + private_data


def restore_export(plugin, preset):
    # Pedalboard exposes VST3 state, not a .dspreset file-picker API. Preserve
    # its wrapper/private data, replace only the instrument XML with the
    # production export, and set the same directory context a file load uses.
    wrapper, _ = unpack_xml(plugin.raw_state)
    component = wrapper.find("IComponent")
    if wrapper.tag != "VST3PluginState" or component is None:
        raise ValueError("Player state has no expected VST3 component")
    component.text = encode_memory_block(build_component_state(decode_memory_block(component.text), preset))
    plugin.raw_state = pack_xml(wrapper)


def write_float_wav(path, audio, rate):
    # Preserve very quiet velocity-1 probes; PCM16 would quantize them away.
    data = audio.T.astype("<f4").tobytes()
    fmt = struct.pack("<HHIIHH", 3, 2, rate, rate * 8, 8, 32)
    body = b"WAVEfmt " + struct.pack("<I", len(fmt)) + fmt
    body += b"data" + struct.pack("<I", len(data)) + data
    path.write_bytes(b"RIFF" + struct.pack("<I", len(body)) + body)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("plugin", type=Path)
    parser.add_argument("preset", type=Path)
    parser.add_argument("midi", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--rate", type=int, default=48000)
    parser.add_argument("--block-size", type=int, default=256)
    parser.add_argument("--state-wait", type=float, default=3.0)
    parser.add_argument("--tail-probe", type=Path, help="Also render a held C4/127 note for four seconds")
    parser.add_argument("--velocity-sweep", type=Path, help="Also write a CSV of every C4 velocity from 1 through 127")
    args = parser.parse_args()
    if not os.environ.get("CFFIXED_USER_HOME"):
        parser.error("Set CFFIXED_USER_HOME to an isolated test directory before loading the player")
    for path in [args.output, args.tail_probe, args.velocity_sweep, args.output.with_suffix(".metadata.json")]:
        if path and path.exists():
            parser.error(f"Refusing to overwrite {path}")
    from pedalboard import load_plugin
    import mido
    import numpy as np

    preset = args.preset.resolve(strict=True)
    plugin = load_plugin(str(args.plugin.resolve(strict=True)))
    if not plugin.is_instrument:
        raise ValueError("Expected an instrument plugin")
    restore_export(plugin, preset)
    # State restoration/sample verification is asynchronous in this player.
    time.sleep(args.state_wait)
    messages, elapsed = [], 0.0
    for event in mido.MidiFile(args.midi):
        elapsed += event.time
        if not event.is_meta:
            messages.append((bytes(event.bytes()), elapsed))
    audio = plugin(messages, duration=elapsed + 0.5, sample_rate=args.rate,
                   num_channels=2, buffer_size=args.block_size, reset=False)
    if not np.isfinite(audio).all() or not np.any(audio):
        raise ValueError("Player returned silent or non-finite audio; inspect its isolated logs")
    write_float_wav(args.output, audio, args.rate)
    metadata = {"plugin": str(args.plugin.resolve()), "preset": str(preset),
                "midi": str(args.midi.resolve()), "sample_rate": args.rate,
                "block_size": args.block_size, "pedalboard": importlib.metadata.version("pedalboard"),
                "mido": importlib.metadata.version("mido"), "duration_seconds": elapsed + 0.5,
                "preset_load_method": "VST3 component XML state restoration with sample-directory context"}
    if args.tail_probe:
        plugin.reset()
        tail = plugin([(bytes([0x90, 60, 127]), 0.0), (bytes([0x80, 60, 0]), 3.5)],
                      duration=4.0, sample_rate=args.rate, num_channels=2,
                      buffer_size=args.block_size, reset=False)
        if not np.isfinite(tail).all() or not np.any(tail[:, :int(0.5 * args.rate)]):
            raise ValueError("Held-note probe contains non-finite audio or did not start sounding")
        write_float_wav(args.tail_probe, tail, args.rate)
        metadata["held_note_tail_peak_after_2_1_seconds"] = float(np.max(np.abs(tail[:, int(2.1 * args.rate):])))
    if args.velocity_sweep:
        plugin.reset()
        restore_export(plugin, preset)
        time.sleep(args.state_wait)
        sweep_messages = []
        for velocity in range(1, 128):
            start = (velocity - 1) * 2.0
            sweep_messages.extend([(bytes([0x90, 60, velocity]), start),
                                   (bytes([0x80, 60, 0]), start + 0.5)])
        sweep = plugin(sweep_messages, duration=254.0, sample_rate=args.rate,
                       num_channels=2, buffer_size=args.block_size, reset=False)
        if not np.isfinite(sweep).all():
            raise ValueError("Velocity sweep contains non-finite audio")
        count = int(0.2 * args.rate)
        times = np.arange(count) / args.rate
        window = np.hanning(count)
        frequency = 440 * 2 ** ((60 - 69) / 12)
        rows = []
        for velocity in range(1, 128):
            start = int(((velocity - 1) * 2.0 + 0.15) * args.rate)
            signal = sweep[0, start:start + count]
            magnitude = lambda f: abs(np.sum(signal * window * np.exp(-2j * np.pi * f * times)))
            fundamental = magnitude(frequency)
            ratio = float(magnitude(2 * frequency) / fundamental) if fundamental > 1e-12 else None
            observed = "silent" if ratio is None else ("high" if ratio > 0.25 else "low")
            rows.append({"velocity": velocity, "rms": float(np.sqrt(np.mean(signal * signal))),
                         "second_harmonic_ratio": ratio, "observed_layer": observed,
                         "expected_layer": "low" if velocity <= 95 else "high"})
        with args.velocity_sweep.open("w", newline="") as source:
            writer = csv.DictWriter(source, fieldnames=list(rows[0]))
            writer.writeheader()
            writer.writerows(rows)
    info = args.plugin.resolve() / "Contents" / "Info.plist"
    if info.exists():
        with info.open("rb") as source:
            metadata["plugin_version"] = plistlib.load(source).get("CFBundleShortVersionString")
    args.output.with_suffix(".metadata.json").write_text(json.dumps(metadata, indent=2) + "\n")
    print(f"Rendered {len(messages)} MIDI events to {args.output}")


if __name__ == "__main__":
    main()
