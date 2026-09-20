"""Compile one executable per IMASM case; execute without numeric inputs.

Run from any directory: python3 baked_phase_check.py
Results and copies of the executables are retained under target/baked-phase/.
Build-time input uses the repository's canonical Rust numeral emitter.
"""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time

ROOT = Path(__file__).resolve().parent


def run(command, **kwargs):
    return subprocess.run(command, cwd=ROOT, check=True, text=True,
                          capture_output=True, **kwargs)


def main():
    run(['cargo', 'build', '--release', '--bin', 'phase_case_encode'])
    encoder = ROOT / 'target/release/phase_case_encode'
    output = ROOT / 'target/baked-phase' / str(time.time_ns())
    output.mkdir(parents=True)
    # Small case is an arithmetic control. Campaign inputs start at 175 bits.
    cases = [('control', 2, 15, 8),
             ('balanced175-q8', 2, 34708385599522211756186926321308977827402135941491069, 8),
             ('balanced175-q10', 2, 34708385599522211756186926321308977827402135941491069, 10)]
    with (output / 'results.jsonl').open('x') as report:
        for name, a, n, width in cases:
            words = run([str(encoder), str(a), str(n), str(width)]).stdout.splitlines()
            assert len(words) == 3
            env = os.environ.copy()
            env.update(zip(['VOX_PHASE_BASE_WORD', 'VOX_PHASE_MODULUS_WORD',
                            'VOX_PHASE_WIDTH_WORD'], words))
            run(['cargo', 'build', '--release', '--bin', 'phase_baked'], env=env)
            binary = output / name
            shutil.copy2(ROOT / 'target/release/phase_baked', binary)
            started = time.monotonic()
            result = run([str(binary)], env={}, input='')
            elapsed = time.monotonic() - started
            # Runtime values cannot override the compiled word.
            poisoned = run([str(binary), '999', '3'],
                           env={'VOX_PHASE_MODULUS_WORD': 'invalid'}, input='999\n')
            assert poisoned.stdout == result.stdout
            assert f'modulus {n}\n' in result.stdout
            assert 'banked_restored true\n' in result.stdout
            m = 1 << width
            positions = [x for x in range(m) if pow(a, x, n) == 1]
            assert f'population {len(positions)}\n' in result.stdout
            # Independent Python arithmetic checks the emitted phase queries.
            import cmath
            import math
            for line in result.stdout.splitlines():
                if line.startswith('probability '):
                    _, k, value = line.split()
                    expected = abs(sum(cmath.exp(2j * math.pi * ((int(k)*x) % m) / m)
                                       for x in positions))**2 / (m * len(positions))
                    assert abs(float(value) - expected) < 1e-10
            row = dict(case=name, bits=n.bit_length(), words=words,
                       binary=str(binary), binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                       seconds=elapsed, runtime_override_ignored=True, stdout=result.stdout)
            report.write(json.dumps(row) + '\n')
            report.flush()
            print(f'{name}: passed; {elapsed:.6f}s', flush=True)
    print(output / 'results.jsonl')


if __name__ == '__main__':
    main()
