#!/usr/bin/env python3
"""Check state-format handling without requiring a player installation."""
import importlib.util
from pathlib import Path
import tempfile
import unittest
import xml.etree.ElementTree as ET

spec = importlib.util.spec_from_file_location("renderer", Path(__file__).with_name("render-decent-sampler.py"))
renderer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(renderer)


class StateFormatTests(unittest.TestCase):
    def test_memory_block_roundtrip_preserves_binary_private_data(self):
        for data in (b"", b"\0", b"\xff\x80", bytes(range(256))):
            self.assertEqual(renderer.decode_memory_block(renderer.encode_memory_block(data)), data)

    def test_truncated_memory_block_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "Truncated"):
            renderer.decode_memory_block("4.A")

    def test_state_restoration_preserves_musical_mapping_and_unicode_context(self):
        initial = ET.fromstring('<DecentSampler _userVolume="1.0"><groups/></DecentSampler>')
        private = b"\0JUCEPrivateData\0\xff"
        wrapper = ET.Element("VST3PluginState")
        ET.SubElement(wrapper, "IComponent").text = renderer.encode_memory_block(renderer.pack_xml(initial) + private)
        class Plugin:
            raw_state = renderer.pack_xml(wrapper)
        plugin = Plugin()
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory) / "Élan & 你好"
            folder.mkdir()
            preset = folder / "Acceptance.dspreset"
            preset.write_text('<DecentSampler><groups><group loVel="96" hiVel="127">'
                              '<sample path="take.wav" loNote="62" hiNote="64" rootNote="63"/>'
                              '</group></groups></DecentSampler>')
            renderer.restore_export(plugin, preset)
            updated, _ = renderer.unpack_xml(plugin.raw_state)
            instrument, updated_private = renderer.unpack_xml(renderer.decode_memory_block(updated.findtext("IComponent")))
            self.assertEqual(instrument.get("_samplePath"), str(folder))
            self.assertEqual(instrument.find("groups/group").attrib, {"loVel": "96", "hiVel": "127"})
            self.assertEqual(instrument.find("groups/group/sample").attrib,
                             {"path": "take.wav", "loNote": "62", "hiNote": "64", "rootNote": "63"})
            self.assertEqual(updated_private, private)

    def test_unrecognized_player_state_fails_before_restore(self):
        with self.assertRaisesRegex(ValueError, "tested JUCE"):
            renderer.unpack_xml(b"unknown-state")


if __name__ == "__main__":
    unittest.main()
