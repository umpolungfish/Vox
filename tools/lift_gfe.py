#!/usr/bin/env python3
"""Lift the complete GFE image through the existing Vox wiring instruments."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_diagram(word, anchor, output):
    sys.path.insert(0, str(ROOT.parent / "IMSCRIBr"))
    from tokens import Token
    from wiring import imscr_wiring
    from native_wiring import inspect_wiring
    from symbolic_diagram import render_wiring_ascii, render_wiring_svg_v3
    sys.path.insert(0, str(ROOT))
    from render_carrier_diagram import GLYPH_TO_NAME

    graph = imscr_wiring(tuple(Token[GLYPH_TO_NAME[mark]] for mark in word))
    report = inspect_wiring(graph)
    (output / "source.graph.json").write_text(json.dumps(report, indent=2) + "\n")
    if not report["fully_reconnected"] or not report["closure_verified"]:
        raise ValueError("the native source graph has not closed its transformed fork/fuse ancestry")
    text = render_wiring_ascii(graph, anchor, graph_report=report).splitlines()
    paired = {pair[0] for pair in report["pairs"]}
    open_forks = sum(token == Token.FSPLIT and index not in paired
                     for index, token in enumerate(graph.tokens))
    text[0] += f" open forks {open_forks}"
    text.append("  walk: CLOSED")
    diagram = output / "source.diagrammar.txt"
    diagram.write_text("\n".join(text) + "\n")
    render_wiring_svg_v3(graph, anchor, "", word, "", pen_mode=True,
                        graph_report=report).save(output / "source.svg")
    return diagram


def lift(binary, vox, output):
    binary, vox, output = binary.resolve(), vox.resolve(), output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    manifest = output / "lift.json"
    source = ROOT / "src/geometer_factor_extractor.rs"
    renderer = ROOT / "render_carrier_diagram.py"
    rewirer = ROOT.parent / "G-mOMonadOS/tools/vox_binary_rewire.py"
    bound_paths = [binary, vox, source, renderer, rewirer, Path(__file__),
                   ROOT.parent / "IMSCRIBr/native_wiring.py",
                   ROOT.parent / "IMSCRIBr/native/wiring_probe.rs",
                   ROOT.parent / "IMSCRIBr/symbolic_diagram.py",
                   ROOT.parent / "IMSCRIBr/wiring.py", ROOT.parent / "IMSCRIBr/tokens.py"]
    bound_paths.extend(sorted((ROOT.parent / "MoDoT/imasm_core/src").glob("*.rs")))
    bindings = {str(p.resolve()): digest(p) for p in bound_paths}
    glyphs = output / "membrane.glyphs"
    if manifest.exists():
        previous = json.loads(manifest.read_text())
        if (previous.get("bindings") == bindings and previous.get("complete")
                and glyphs.exists() and digest(glyphs) == previous.get("glyph_sha256")):
            return glyphs
        raise ValueError(f"preserve the existing lift and choose a new output directory: {output}")
    if any(output.iterdir()):
        raise ValueError(f"preserve the incomplete lift and choose a new output directory: {output}")

    started = time.monotonic()
    wiring = output / "image.wiring.tsv"
    with wiring.open("w") as out, (output / "wiring.stderr").open("w") as err:
        subprocess.run([str(vox), "disasm", str(binary), "--wiring-complete"],
                       stdout=out, stderr=err, check=True)
    anchors = set()
    with wiring.open() as rows:
        for row in rows:
            if row.startswith("# symbol\t"):
                name = row.rstrip().split("\t", 2)[2]
                for candidate in ("GfeMorphismMachine", "FactorSynthesis", "GeometerFactorExtractor"):
                    if candidate in name:
                        anchors.add(candidate)
    anchor = next((name for name in ("GfeMorphismMachine", "FactorSynthesis", "GeometerFactorExtractor")
                   if name in anchors), None)
    if anchor is None:
        raise ValueError("GFE image has no retained morphism/synthesis symbol anchor; rebuild without stripping")
    word = re.search(r'pub const GFE_MORPHISM_WORD: &str = "([^"]+)";', source.read_text())
    if word is None:
        raise ValueError("the live GFE source does not expose its morphism word")
    diagram = source_diagram(word.group(1), anchor, output)
    tape = output / "membrane.imasm"
    report = output / "rewired.json.gz"
    with (output / "rewire.stdout").open("w") as out, (output / "rewire.stderr").open("w") as err:
        subprocess.run([sys.executable, str(rewirer), "--vox-wiring", str(wiring),
                        "--elf", str(binary), "--vox", str(vox), "--diagrammar", str(diagram),
                        "--output", str(report), "--imasm-output", str(tape),
                        "--diagrammar-output", str(output / "rewired.diagrammar.txt.gz")],
                       stdout=out, stderr=err, check=True)
    import gzip
    with gzip.open(report, "rt") as stream:
        checks = json.load(stream)["checks"]
    if not checks.get("publishable") or not checks.get("rewritten_imasm_tape"):
        raise ValueError("the complete image did not pass the existing wiring and tape gates")
    recovered = output / "recovered.imasm"
    with (output / "codec.log").open("w") as log:
        for command in ([str(vox), "glyphs", str(tape), str(glyphs)],
                        [str(vox), "unglyphs", str(glyphs), str(recovered)]):
            subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, check=True)
    if digest(tape) != digest(recovered):
        raise ValueError("the executable glyph module did not recover the complete tape")
    if any(digest(Path(path)) != value for path, value in bindings.items()):
        raise ValueError("a bound source or instrument changed during the lift")
    manifest.write_text(json.dumps({
        "complete": True, "bindings": bindings, "anchor": anchor,
        "word": word.group(1), "checks": checks, "tape_sha256": digest(tape),
        "glyph_sha256": digest(glyphs), "roundtrip_exact": True,
        "lift_seconds": time.monotonic() - started,
        "preparation_boundary": "runtime resident construction retained from the source image",
        "execution": "not performed by the lifting instrument",
    }, indent=2) + "\n")
    return glyphs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--vox", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        print(lift(args.binary, args.vox, args.output))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"GFE lift failed: {error}; evidence retained in {args.output}\n")


if __name__ == "__main__":
    main()
