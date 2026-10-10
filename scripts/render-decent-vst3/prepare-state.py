#!/usr/bin/env python3
"""Prepare DecentSampler component state for the direct SDK diagnostic host."""
import argparse
from pathlib import Path
import runpy

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("seed", type=Path)
parser.add_argument("preset", type=Path)
parser.add_argument("output", type=Path)
parser.add_argument("--diagnostic-fractional-thresholds", action="store_true",
                    help="Investigative control only: shift group velocity thresholds by -0.00001")
args = parser.parse_args()
if args.output.exists():
    parser.error(f"Refusing to overwrite {args.output}")
renderer = runpy.run_path(str(Path(__file__).parent.parent / "render-decent-sampler.py"))
state = renderer["build_component_state"](args.seed.read_bytes(), args.preset.resolve(strict=True))
if args.diagnostic_fractional_thresholds:
    root, private_data = renderer["unpack_xml"](state)
    for group in root.findall("groups/group"):
        for name in ("loVel", "hiVel"):
            if group.get(name) is not None and float(group.get(name)) != 127.0:
                group.set(name, str(float(group.get(name)) - 0.00001))
    state = renderer["pack_xml"](root) + private_data
    print("Diagnostic thresholds changed; this state is NOT standard export acceptance")
args.output.write_bytes(state)
print(f"Prepared {len(state)} bytes of native player component state")
