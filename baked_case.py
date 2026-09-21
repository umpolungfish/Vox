"""Build-only inputs, canonical IMASM encoding, retained per-case executable."""
import os
import fcntl
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parent


def build_case(n, mode, width=None):
    # Hold the lock through the copy: another case must not replace Cargo's
    # output between compilation and retention of this executable.
    (ROOT / 'target').mkdir(exist_ok=True)
    with (ROOT / 'target/baked-case.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        return _build_case(n, mode, width)


def _build_case(n, mode, width):
    subprocess.run(['cargo', 'build', '--release', '--bin', 'phase_case_encode'],
                   cwd=ROOT, check=True, capture_output=True)
    values = [str(n), '2'] + ([str(width)] if width is not None else [])
    words = subprocess.check_output([str(ROOT / 'target/release/phase_case_encode'), *values], text=True).splitlines()
    destination = ROOT / 'target/baked-semiprime' / str(time.time_ns())
    destination.mkdir(parents=True)
    inputs = destination/'inputs.imasm'
    inputs.write_text('\n'.join(words)+'\n')
    env = os.environ.copy()
    env['VOX_BAKED_INPUT_FILE'] = str(inputs)
    env.pop('VOX_PHASE_MODULUS_WORD', None)
    env.pop('VOX_PHASE_BASE_WORD', None)
    env['VOX_PROBE_MODE'] = mode
    env.pop('VOX_PHASE_WIDTH_WORD', None)
    target_bin = {'phaseB': 'phaseB_fac', 'winding': 'winding_membrane'}.get(mode, 'semiprime_probe')
    subprocess.run(['cargo', 'build', '--release', '--bin', target_bin],
                   cwd=ROOT, env=env, check=True, capture_output=True)
    binary = destination / mode
    shutil.copy2(ROOT / 'target/release' / target_bin, binary)
    return binary, words
