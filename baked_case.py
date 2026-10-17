"""Build-only inputs, canonical IMASM encoding, retained per-case executable."""
import os
import fcntl
from pathlib import Path
import shutil
import subprocess
import time
from standalone import require_standalone, static_env

ROOT = Path(__file__).resolve().parent


def build_case(n, mode, width=None, target=None):
    # Hold the lock through the copy: another case must not replace Cargo's
    # output between compilation and retention of this executable.
    (ROOT / 'target').mkdir(exist_ok=True)
    with (ROOT / 'target/baked-case.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        return _build_case(n, mode, width, target)


def _build_case(n, mode, width, target):
    target = target or 'x86_64-unknown-linux-musl'
    subprocess.run(['cargo', 'build', '--release', '--target', 'x86_64-unknown-linux-musl', '--bin', 'phase_case_encode', '--target-dir', 'target'],
                   cwd=ROOT, env=static_env(), check=True, capture_output=True)
    values = [str(n), '2'] + ([str(width)] if width is not None else [])
    words = subprocess.check_output([str(ROOT / 'target/x86_64-unknown-linux-musl/release/phase_case_encode'), *values], text=True).splitlines()
    destination = ROOT / 'target/baked-semiprime' / str(time.time_ns())
    destination.mkdir(parents=True)
    inputs = destination/'inputs.imasm'
    inputs.write_text('\n'.join(words)+'\n')
    env = static_env()
    env['VOX_BAKED_INPUT_FILE'] = str(inputs)
    env.pop('VOX_PHASE_MODULUS_WORD', None)
    env.pop('VOX_PHASE_BASE_WORD', None)
    env['VOX_PROBE_MODE'] = mode
    env.pop('VOX_PHASE_WIDTH_WORD', None)
    target_bin = {'phaseB': 'phaseB_fac', 'winding': 'winding_membrane', 'ring': 'ring_membrane', 'basemod': 'base_mod_membrane', 'reduce': 'reduce_delete_membrane', 'cylinder': 'cylinder_membrane'}.get(mode, 'semiprime_probe')
    build = ['cargo', 'build', '--release', '--bin', target_bin, '--target-dir', 'target']
    if target is not None:
        build += ['--target', target]
    subprocess.run(build,
                   cwd=ROOT, env=env, check=True, capture_output=True)
    binary = destination / mode
    output = ROOT / 'target'
    if target is not None:
        output /= target
    require_standalone(output / 'release' / target_bin)
    shutil.copy2(output / 'release' / target_bin, binary)
    return binary, words
