#!/usr/bin/env python3
"""Run the canonical Vox table generator, preserving its live Lean default."""
from pathlib import Path
import runpy
runpy.run_path(str(Path(__file__).resolve().parents[1] / 'gen_genetic_table.py'), run_name='__main__')
