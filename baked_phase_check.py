"""Compile one executable per IMASM case; execute without numeric inputs.

Run from any directory: python3 baked_phase_check.py
Results and copies of the executables are retained under target/baked-phase/.
Build-time input uses the repository's canonical Rust numeral emitter.
"""
import hashlib
import fcntl
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
from imasm16_3_core import IMASM16_3_Machine

ROOT = Path(__file__).resolve().parent
BLUEPRINT = ROOT.parent / 'ob3ect/digital/obtaining_informative_phase_observations_without_913a8f7c/obtaining_informative_phase_observations_without_913a8f7c_ob3ect.json'


def run(command, **kwargs):
    return subprocess.run(command, cwd=ROOT, check=True, text=True,
                          capture_output=True, **kwargs)


def main():
    blueprint = json.loads(BLUEPRINT.read_text())
    execution_word = blueprint['glyph_word']
    run(['cargo', 'build', '--release', '--bin', 'phase_case_encode'])
    encoder = ROOT / 'target/release/phase_case_encode'
    output = ROOT / 'target/baked-phase' / str(time.time_ns())
    output.mkdir(parents=True)
    # Small case is an arithmetic control. Campaign inputs start at 175 bits.
    wide = 34708385599522211756186926321308977827402135941491069
    cases = [('control', 2, 15, 8, 0), ('balanced175-q10', 2, wide, 10, 0)]
    cases += [(f'balanced175-cut{k}', 2, wide, 8, k) for k in range(len(execution_word))]
    with (output / 'results.jsonl').open('x') as report:
        for name, a, n, width, cut in cases:
            words = run([str(encoder), str(a), str(n), str(width)]).stdout.splitlines()
            assert len(words) == 3
            env = os.environ.copy()
            cut_word = execution_word[cut:] + execution_word[:cut]
            env['VOX_PHASE_EXECUTION_WORD'] = cut_word
            env.update(zip(['VOX_PHASE_BASE_WORD', 'VOX_PHASE_MODULUS_WORD',
                            'VOX_PHASE_WIDTH_WORD'], words))
            binary = output / name
            with (ROOT / 'target/baked-phase.lock').open('a') as lock:
                fcntl.flock(lock, fcntl.LOCK_EX)
                run(['cargo', 'build', '--release', '--bin', 'phase_baked'], env=env)
                shutil.copy2(ROOT / 'target/release/phase_baked', binary)
            started = time.monotonic()
            result = run([str(binary)], env={}, input='')
            elapsed = time.monotonic() - started
            # Runtime values cannot override the compiled word.
            poisoned = run([str(binary), '999', '3'],
                           env={'VOX_PHASE_MODULUS_WORD': 'invalid'}, input='999\n')
            assert poisoned.stdout == result.stdout
            assert f'modulus {n}\n' in result.stdout
            reference = IMASM16_3_Machine()
            for mark in cut_word:
                reference.transition(mark)
            expected_register = sum(bit for lane, bit in [('T',1),('F',2),('t',4),('f',8)]
                                    if lane in reference.reg)
            assert int(result.stdout.splitlines()[0].split()[1]) == expected_register
            holds = cut in (0, 1, 2, 11)
            if holds:
                assert 'banked_restored true\n' in result.stdout
                assert 'register 15 surviving 4 cleared 4 restored 4 exposed 0\n' in result.stdout
            else:
                assert 'unresolved: no surviving phase observation\n' in result.stdout
                assert 'probability ' not in result.stdout
            m = 1 << width
            positions = [x for x in range(m) if pow(a, x, n) == 1]
            if holds:
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
            row = dict(case=name, bits=n.bit_length(), words=words, execution_word=cut_word, cut=cut,
                       blueprint_sha256=hashlib.sha256(BLUEPRINT.read_bytes()).hexdigest(),
                       binary=str(binary), binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                       seconds=elapsed, runtime_override_ignored=True, stdout=result.stdout)
            report.write(json.dumps(row) + '\n')
            report.flush()
            print(f'{name}: passed; {elapsed:.6f}s', flush=True)
    print(output / 'results.jsonl')


if __name__ == '__main__':
    main()
