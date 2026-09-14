#!/usr/bin/env python3
"""Render an IMASM sequence as a symbolic wiring (circuit) diagram.

Reuses IMSCRIBr's diagram code, the same path ob3ect/auto.py drives:
imscr_wiring builds the graph from the token list, render_wiring_svg_v3 draws it.

Usage:
    python3 render_carrier_diagram.py <imasm-sequence> [name] [out.svg]

Only the sequence is required. With no name, "carrier" is used; with no output
path, the SVG is written next to this script as <name>.svg. The ASCII circuit is
always printed to the terminal.
"""
import sys
import hashlib
from pathlib import Path

IMSCRIBR = str(Path(__file__).resolve().parents[1] / "IMSCRIBr")
if IMSCRIBR not in sys.path:
    sys.path.insert(0, IMSCRIBR)

from tokens import Token  # noqa: E402
from wiring import imscr_wiring  # noqa: E402
from symbolic_diagram import render_wiring_svg_v3, render_wiring_ascii  # noqa: E402

GLYPH_TO_NAME = {
    "⊢": "VINIT", "⊣": "TANCH", "≻": "AFWD", "≺": "AREV", "⋈": "CLINK",
    "⊙": "IMSCRIB", "∈": "FSPLIT", "∋": "FFUSE", "⊤": "EVALT", "⊥": "EVALF",
    "⊞": "ENGAGR", "⊡": "IFIX",
}


def render(word: str, name: str = "carrier", out_path: str | None = None) -> str:
    word = word.strip()
    unknown = [g for g in word if g not in GLYPH_TO_NAME]
    if unknown:
        raise SystemExit(f"not an IMASM mark: {' '.join(unknown)}")
    tokens = tuple(Token[GLYPH_TO_NAME[g]] for g in word)
    graph = imscr_wiring(tokens)
    graph.name = name
    if out_path is None:
        out_path = str(Path(__file__).resolve().parent / f"{name}.svg")
    svg = render_wiring_svg_v3(graph, name, "", word, "", pen_mode=True)
    svg.save(Path(out_path))
    ascii_art = render_wiring_ascii(graph, name)
    if ascii_art:
        print(ascii_art)
    print(f"\nwrote {out_path}")
    return out_path


if __name__ == "__main__":
    if len(sys.argv) < 2:
        raise SystemExit("usage: render_carrier_diagram.py <imasm-sequence> [name] [out.svg]")
    seq = sys.argv[1]
    nm = sys.argv[2] if len(sys.argv) > 2 else f"carrier_{hashlib.sha1(seq.encode()).hexdigest()[:8]}"
    out = sys.argv[3] if len(sys.argv) > 3 else None
    render(seq, nm, out)
