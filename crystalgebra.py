#!/usr/bin/env python3
"""
IG Geometric Crystal — Operational Visualization of the 12-Primitive Lattice
==========================================================================
Renders the canonical geometric carrier of the Imscribing Grammar:

  • 3 equilateral triangles  (ℱ₃ family, edge valence = 3)
  • 5 squares                (ℱ₄ family, edge valence = 4)
  • 4 regular pentagons      (ℱ₅ family, edge valence = 5)

All polygons have identical side length s = 1 (metric commensurability).
Each carries a unique color (orthogonal categorical axis).

Demonstrates:
  1. Equal side-length → unified metric manifold
  2. Edge valence = ordinal chain length of the corresponding logical lattice
  3. Pentagonal tiling defect → geometric origin of the Ω-barrier
  4. Five-fold symmetry → ⊙ absorption (quasicrystalline collapse)

Author: Math⊙perator (Lando⊗⊙perator team)
Requires: matplotlib, numpy
"""

import numpy as np
import matplotlib.pyplot as plt
from matplotlib.patches import Polygon, Circle, FancyBboxPatch
from matplotlib.collections import PatchCollection
import matplotlib.patches as mpatches
from matplotlib.lines import Line2D

# ─────────────────────────────────────────────────────────────────────────────
# Canonical mapping (aligned with crystal cardinality 3³ × 4⁵ × 5⁴)
# ─────────────────────────────────────────────────────────────────────────────

# F3 — TernaryTruth lattice (absent / some / full)
F3 = [
    {"name": "Fidelity f",      "glyph": "f",  "shape": "triangle", "family": "F3"},
    {"name": "Granularity G",   "glyph": "G",  "shape": "triangle", "family": "F3"},
    {"name": "Stoichiometry S", "glyph": "S",  "shape": "triangle", "family": "F3"},
]

# F4 — Belnap FOUR bilattice
F4 = [
    {"name": "Dimensionality D", "glyph": "D", "shape": "square", "family": "F4"},
    {"name": "Relational R",     "glyph": "R", "shape": "square", "family": "F4"},
    {"name": "Grammar g",        "glyph": "g", "shape": "square", "family": "F4"},
    {"name": "Chirality H",      "glyph": "H", "shape": "square", "family": "F4"},
    {"name": "Protection O",     "glyph": "O", "shape": "square", "family": "F4"},
]

# F5 — QuarkBelnap FIVE (confinement ceiling)
F5 = [
    {"name": "Topology T",     "glyph": "T", "shape": "pentagon", "family": "F5"},
    {"name": "Polarity P",     "glyph": "P", "shape": "pentagon", "family": "F5"},
    {"name": "Criticality Odot", "glyph": "odot", "shape": "pentagon", "family": "F5"},
    {"name": "Kinetics K",     "glyph": "K", "shape": "pentagon", "family": "F5"},
]

ALL_PRIMITIVES = F3 + F4 + F5
assert len(ALL_PRIMITIVES) == 12

# Distinct high-contrast colors (12 orthogonal axes)
COLORS = [
    "#E63946", "#F4A261", "#2A9D8F",   # triangles
    "#264653", "#E9C46A", "#F77F00", "#9B5DE5", "#00BBF9",  # squares
    "#00F5D4", "#FEE440", "#F15BB5", "#9B5DE5",            # pentagons (last reused intentionally for visual balance)
]

for i, p in enumerate(ALL_PRIMITIVES):
    p["color"] = COLORS[i]

SIDE = 1.0  # equal side-length — the metric commensurability axiom


# ─────────────────────────────────────────────────────────────────────────────
# Geometry helpers
# ─────────────────────────────────────────────────────────────────────────────

def regular_polygon_vertices(n_sides: int, side: float = SIDE, cx: float = 0.0, cy: float = 0.0, rotation: float = 0.0):
    """Return vertices of a regular n-gon with given side length, centered at (cx,cy)."""
    # Circumradius R from side length: R = s / (2 * sin(π/n))
    R = side / (2.0 * np.sin(np.pi / n_sides))
    angles = np.linspace(0, 2 * np.pi, n_sides, endpoint=False) + rotation
    xs = cx + R * np.cos(angles)
    ys = cy + R * np.sin(angles)
    return np.column_stack([xs, ys])


def make_patch(prim: dict, cx: float, cy: float, rotation: float = 0.0, alpha: float = 0.85):
    n = {"triangle": 3, "square": 4, "pentagon": 5}[prim["shape"]]
    verts = regular_polygon_vertices(n, SIDE, cx, cy, rotation)
    return Polygon(verts, closed=True,
                   facecolor=prim["color"], edgecolor="black",
                   linewidth=1.4, alpha=alpha)


# ─────────────────────────────────────────────────────────────────────────────
# Main visualization
# ─────────────────────────────────────────────────────────────────────────────

def draw_crystal_gallery(ax):
    """Gallery of all 12 polygons arranged by family."""
    ax.set_aspect("equal")
    ax.set_xlim(-1.5, 13.5)
    ax.set_ylim(-1.8, 4.2)
    ax.axis("off")
    ax.set_title("IG Geometric Crystal — 12 Primitives as Regular Polygons\n"
                 "Equal side-length s = 1  •  Edge valence = ordinal chain length",
                 fontsize=13, pad=12, fontweight="bold")

    # F3 row
    ax.text(0.5, 3.6, "F3  (TernaryTruth)  —  3 triangles", fontsize=10, fontweight="bold", color="#333")
    for i, p in enumerate(F3):
        cx = 1.2 + i * 2.4
        patch = make_patch(p, cx, 2.4)
        ax.add_patch(patch)
        ax.text(cx, 1.35, f"{p['name']}\n{p['glyph']}", ha="center", va="top", fontsize=8)

    # F4 row
    ax.text(0.5, 0.9, "F4  (Belnap FOUR)  —  5 squares", fontsize=10, fontweight="bold", color="#333")
    for i, p in enumerate(F4):
        cx = 1.0 + i * 2.5
        patch = make_patch(p, cx, -0.2, rotation=np.pi/4)  # diamond orientation for visual clarity
        ax.add_patch(patch)
        ax.text(cx, -1.35, f"{p['name']}\n{p['glyph']}", ha="center", va="top", fontsize=7.5)

    # Legend box
    legend_elements = [
        mpatches.Patch(facecolor="#E63946", edgecolor="k", label="F3 triangle (valence 3)"),
        mpatches.Patch(facecolor="#264653", edgecolor="k", label="F4 square   (valence 4)"),
        mpatches.Patch(facecolor="#00F5D4", edgecolor="k", label="F5 pentagon (valence 5)"),
    ]
    ax.legend(handles=legend_elements, loc="upper right", fontsize=8, framealpha=0.9)


def draw_pentagon_defect(ax):
    """Demonstrate the tiling defect: triangles + squares tile flatly; pentagons force curvature."""
    ax.set_aspect("equal")
    ax.set_xlim(-3.5, 10.5)
    ax.set_ylim(-3.5, 4.5)
    ax.axis("off")
    ax.set_title("Pentagonal Tiling Defect  ->  Geometric Origin of the Omega-Barrier",
                 fontsize=12, pad=10, fontweight="bold")

    # Left: flat tiling attempt with triangles + squares
    ax.text(1.5, 3.8, "Flat region (triangles + squares)", ha="center", fontsize=9, style="italic")
    # a few triangles
    for i, (cx, cy, rot) in enumerate([(0, 2.2, 0), (1.5, 2.2, np.pi), (0.75, 0.9, 0)]):
        verts = regular_polygon_vertices(3, SIDE, cx, cy, rot)
        ax.add_patch(Polygon(verts, facecolor="#E63946", edgecolor="k", alpha=0.7, lw=1.2))
    # a few squares
    for i, (cx, cy) in enumerate([(3.2, 2.0), (4.7, 2.0), (3.95, 0.5)]):
        verts = regular_polygon_vertices(4, SIDE, cx, cy, np.pi/4)
        ax.add_patch(Polygon(verts, facecolor="#264653", edgecolor="k", alpha=0.7, lw=1.2))

    # Right: pentagons producing defects
    ax.text(8.0, 3.8, "Defect region (pentagons)", ha="center", fontsize=9, style="italic")
    # central pentagon
    verts = regular_polygon_vertices(5, SIDE, 8.0, 1.5, -np.pi/2)
    ax.add_patch(Polygon(verts, facecolor="#00F5D4", edgecolor="k", alpha=0.85, lw=1.5))
    # surrounding pentagons with visible gaps / overlaps indicating curvature
    for ang in np.linspace(0, 2*np.pi, 5, endpoint=False):
        R = 2.15
        cx = 8.0 + R * np.cos(ang)
        cy = 1.5 + R * np.sin(ang)
        verts = regular_polygon_vertices(5, SIDE, cx, cy, ang + np.pi/5)
        ax.add_patch(Polygon(verts, facecolor="#F15BB5", edgecolor="k", alpha=0.45, lw=1.0))

    # annotation
    ax.annotate("Curvature / aperiodicity\n(Omega-barrier)",
                xy=(8.0, -1.2), xytext=(8.0, -2.6),
                ha="center", fontsize=9, color="#c1121f",
                arrowprops=dict(arrowstyle="->", color="#c1121f", lw=1.5))

    ax.text(1.5, -2.8, "Euler chi ~ 0  (flat)", ha="center", fontsize=8, color="#2a9d8f")
    ax.text(8.0, -2.8, "Euler chi != 0  (defect)", ha="center", fontsize=8, color="#c1121f")


def draw_absorption_demo(ax):
    """Visualise ⊙ absorption: once a Criticality pentagon appears, local periodicity collapses."""
    ax.set_aspect("equal")
    ax.set_xlim(-1, 11)
    ax.set_ylim(-2.5, 3.5)
    ax.axis("off")
    ax.set_title("Odot Absorption — Five-fold Symmetry Destroys Periodicity",
                 fontsize=12, pad=10, fontweight="bold")

    # Before: ordered lattice of squares
    ax.text(2.5, 2.8, "Before Odot", ha="center", fontsize=9, style="italic")
    for i in range(3):
        for j in range(2):
            cx = 1.0 + i * 1.6
            cy = 1.2 - j * 1.6
            verts = regular_polygon_vertices(4, SIDE*0.9, cx, cy, np.pi/4)
            ax.add_patch(Polygon(verts, facecolor="#264653", edgecolor="k", alpha=0.75, lw=1.0))

    # Arrow
    ax.annotate("", xy=(6.2, 0.8), xytext=(5.0, 0.8),
                arrowprops=dict(arrowstyle="->", color="black", lw=2))

    # After: Criticality pentagon absorbs neighbours into quasicrystalline arrangement
    ax.text(8.5, 2.8, "After Odot (absorbing)", ha="center", fontsize=9, style="italic")
    # central Criticality
    verts = regular_polygon_vertices(5, SIDE, 8.5, 0.8, -np.pi/2)
    ax.add_patch(Polygon(verts, facecolor="#FEE440", edgecolor="k", alpha=0.95, lw=2.0))
    ax.text(8.5, 0.8, "Odot", ha="center", va="center", fontsize=11, fontweight="bold")

    # surrounding distorted / aperiodic fragments
    for ang, col in zip(np.linspace(0, 2*np.pi, 6, endpoint=False),
                        ["#E63946", "#2A9D8F", "#F77F00", "#9B5DE5", "#00BBF9", "#F15BB5"]):
        R = 2.0
        cx = 8.5 + R * np.cos(ang)
        cy = 0.8 + R * np.sin(ang)
        # deliberately irregular scale to suggest absorption of order
        scale = 0.55 + 0.25 * np.sin(3*ang)
        verts = regular_polygon_vertices(4 if ang < np.pi else 3, SIDE*scale, cx, cy, ang)
        ax.add_patch(Polygon(verts, facecolor=col, edgecolor="k", alpha=0.55, lw=0.8))

    ax.text(5.5, -1.8,
            "Odot tensor x = Odot   (any neighbouring structure is absorbed into the critical state)",
            ha="center", fontsize=9, color="#333",
            bbox=dict(boxstyle="round,pad=0.3", facecolor="#fff3b0", alpha=0.9))


def draw_metric_commensurability(ax):
    """Show that a single edge length is shared across all families."""
    ax.set_aspect("equal")
    ax.set_xlim(-1, 10)
    ax.set_ylim(-1.5, 3.5)
    ax.axis("off")
    ax.set_title("Metric Commensurability — One Side Length for All 12 Axes",
                 fontsize=12, pad=10, fontweight="bold")

    # Three representative polygons side-by-side with highlighted shared edge
    positions = [(1.5, 1.2, 3, "#E63946", "F3"),
                 (4.5, 1.2, 4, "#264653", "F4"),
                 (7.8, 1.2, 5, "#00F5D4", "F5")]

    for cx, cy, n, col, fam in positions:
        verts = regular_polygon_vertices(n, SIDE, cx, cy, -np.pi/2 if n == 5 else 0)
        ax.add_patch(Polygon(verts, facecolor=col, edgecolor="k", alpha=0.8, lw=1.5))
        # highlight one edge
        ax.plot(verts[0:2, 0], verts[0:2, 1], color="red", lw=3.5, zorder=5)
        ax.text(cx, cy - 1.4, fam, ha="center", fontsize=10, fontweight="bold")

    ax.text(5, -0.9, "Identical red edge length s = 1\n"
                     "=> Hamming / tensor distances live on a single metric manifold",
            ha="center", fontsize=9,
            bbox=dict(boxstyle="round,pad=0.4", facecolor="#e8f5e9", alpha=0.95))


# ─────────────────────────────────────────────────────────────────────────────
# Driver
# ─────────────────────────────────────────────────────────────────────────────

def main():
    fig = plt.figure(figsize=(14, 16), facecolor="#fafafa")
    fig.suptitle("Imscribing Grammar - Geometric Expression of the Crystal of Types\n"
                 "3 triangles + 5 squares + 4 pentagons  =  12 primitives",
                 fontsize=15, fontweight="bold", y=0.98)

    # Panel 1 — full gallery
    ax1 = fig.add_subplot(4, 1, 1)
    draw_crystal_gallery(ax1)

    # Panel 2 — metric commensurability
    ax2 = fig.add_subplot(4, 1, 2)
    draw_metric_commensurability(ax2)

    # Panel 3 — tiling defect / Ω-barrier
    ax3 = fig.add_subplot(4, 1, 3)
    draw_pentagon_defect(ax3)

    # Panel 4 — ⊙ absorption
    ax4 = fig.add_subplot(4, 1, 4)
    draw_absorption_demo(ax4)

    plt.tight_layout(rect=[0, 0.02, 1, 0.96])
    out_path = "/home/workdir/artifacts/ig_geometric_crystal.png"
    plt.savefig(out_path, dpi=160, bbox_inches="tight", facecolor=fig.get_facecolor())
    print(f"Saved geometric crystal visualization → {out_path}")

    # Also emit a compact textual summary of the operational invariants
    print("\n-- Operational Invariants --------------------------------------")
    print(f"Crystal cardinality          : 3^3 x 4^5 x 5^4 = {3**3 * 4**5 * 5**4:,}")
    print(f"Equal side length            : s = {SIDE}")
    print(f"F3 (triangles)               : {[p['name'] for p in F3]}")
    print(f"F4 (squares)                 : {[p['name'] for p in F4]}")
    print(f"F5 (pentagons)               : {[p['name'] for p in F5]}")
    print("Tiling defect (Omega-barrier) : pentagons force non-zero curvature")
    print("Absorption (Odot)             : Odot tensor x = Odot  (five-fold destroys periodicity)")
    print("--------------------------------------------------------------")


if __name__ == "__main__":
    main()
