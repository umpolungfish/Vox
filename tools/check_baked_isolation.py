#!/usr/bin/env python3
"""Test CPU baked-image isolation and source binding on fresh 224-bit inputs."""
import argparse
import cmath
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
from datetime import datetime

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
sys.path.insert(0, str(ROOT / 'measurements'))
from standalone import require_standalone
from baked_case import build_case
from random_rsa_fixtures import fixture


def semantic_output(text):
    return '\n'.join(line for line in text.splitlines() if not line.startswith('seconds '))


def validate_sparse_output(number, text):
    size = 256
    positions = [k for k in range(size) if pow(2, k, int(number)) == 1]
    assert f'positions {len(positions)} register_size {size}' in text.splitlines()[0]
    probabilities = [line.split() for line in text.splitlines() if line.startswith('probability ')]
    assert [int(row[1]) for row in probabilities] == [0, 1, size // 2, size - 1]
    for _, index, value in probabilities:
        expected = abs(sum(cmath.exp(2j * cmath.pi * ((int(index) * x) % size) / size)
                           for x in positions))**2 / (size * len(positions))
        assert abs(float(value) - expected) < 1e-12, (index, value, expected)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, help='fresh results directory, or an existing run with --trace')
    parser.add_argument('--trace', action='store_true', help='trace already baked CPU images; no builds or GPU execution')
    parser.add_argument('--glyphs', action='store_true', help='extend an existing run with raw-word baking and glyph round-trip execution')
    args = parser.parse_args()
    if (args.trace or args.glyphs) and args.output is None:
        parser.error('Supply --output with the results directory to extend or trace.')
    output = (args.output or ROOT / 'measurements/program_operations_20261015' /
              datetime.now().strftime('isolation_%Y%m%d_%H%M%S')).resolve()
    receipt = output / 'receipt.json'
    if args.trace and args.glyphs:
        parser.error('Run --glyphs before --trace as separate checks.')
    if args.glyphs:
        report = json.loads(receipt.read_text())
        vox = ROOT / 'target/x86_64-unknown-linux-musl/release/vox'
        for index, source in enumerate(report['sources']):
            case = Path(source['path']).parent
            image = output / f'word_{index}'
            attempt = 1
            while image.exists():
                attempt += 1
                image = output / f'word_{index}_r{attempt}'
            env = dict(os.environ, VOX_MEMBRANE_WORD_FILE=str(case / 'input.imasm'))
            subprocess.run(['bash', str(ROOT / 'vox_membrane.sh'), str(image)],
                           cwd=ROOT, env=env, check=True)
            require_standalone(image)
            native = subprocess.run([str(image)], cwd=output / 'empty', env={},
                                    text=True, capture_output=True, check=True)
            (output / f'{image.name}.stdout').write_text(native.stdout)
            (output / f'{image.name}.stderr').write_text(native.stderr)
            module = Path(str(image) + '.imasm')
            glyphs = Path(str(image) + '.glyphs')
            restored = Path(str(image) + '.restored.imasm')
            for command in ([vox, 'imasm', image, '--output', module],
                            [vox, 'glyphs', module, glyphs],
                            [vox, 'unglyphs', glyphs, restored]):
                subprocess.run(list(map(str, command)), check=True, capture_output=True, text=True)
            assert module.read_bytes() == restored.read_bytes()
            executed = subprocess.run([str(vox), 'run', str(glyphs)], cwd=output / 'empty', env={},
                                      text=True, capture_output=True, check=True)
            (output / f'{image.name}.vox.stdout').write_text(executed.stdout)
            (output / f'{image.name}.vox.stderr').write_text(executed.stderr)
            body = '\n'.join(line for line in executed.stdout.splitlines()
                             if not line.startswith('entry(...) ')) + '\n'
            assert body == native.stdout, (body, native.stdout)
            damaged = glyphs.read_text()
            at = damaged.rfind('⊤')
            assert at >= 0
            corrupt = Path(str(image) + '.corrupt.glyphs')
            corrupt.write_text(damaged[:at] + '⊥' + damaged[at+1:])
            refused = subprocess.run([str(vox), 'unglyphs', str(corrupt), str(corrupt) + '.imasm'],
                                     text=True, capture_output=True)
            assert refused.returncode != 0, 'corrupt glyph image was admitted'
            report['images'].append(dict(name=image.name, image=str(image),
                sha256=hashlib.sha256(image.read_bytes()).hexdigest(), static=True,
                glyph_roundtrip_exact=True, native_vm_stdout_equal=True,
                corrupt_glyph_rejected=True, empty_environment=True, empty_working_directory=True))
            receipt.write_text(json.dumps(report, indent=2) + '\n')
            print(f'word_{index}: exact glyph round trip, CPU execution agreement, corruption rejected', flush=True)
        return
    if args.trace:
        report = json.loads(receipt.read_text())
        for row in report['images']:
            image = Path(row['image'])
            require_standalone(image)
            if hashlib.sha256(image.read_bytes()).hexdigest() != row['sha256']:
                raise ValueError(f'Executable binding changed: {image}')
            trace = output / (row['name'] + '.strace')
            result = subprocess.run(['strace', '-f', '-qq', '-e', 'trace=execve,execveat,open,openat,openat2,creat',
                                     '-o', str(trace), str(image)], cwd=output / 'empty',
                                    env={}, capture_output=True, text=True, check=True)
            recorded = (output / (row['name'] + '.stdout')).read_text()
            assert semantic_output(result.stdout) == semantic_output(recorded), row['name']
            events = trace.read_text()
            assert not re.search(r'\b(?:open|openat|openat2|creat|execveat)\(', events), events
            assert len(re.findall(r'\bexecve\(', events)) == 1, events
            row['trace'] = dict(path=str(trace), file_opens=0, execve_count=1)
            receipt.write_text(json.dumps(report, indent=2) + '\n')
            print(row['name'] + ': no file opens or helper launches', flush=True)
        return

    output.mkdir(parents=True, exist_ok=False)
    (output / 'empty').mkdir()
    report = dict(source_bits=224, gpu_execution=False, factor_searches_executed=False,
                  sources=[], images=[], checks=[])

    def save():
        receipt.write_text(json.dumps(report, indent=2) + '\n')

    def call(name, command, env=None):
        result = subprocess.run(list(map(str, command)), cwd=ROOT, env=env,
                                capture_output=True, text=True)
        (output / (name + '.stdout')).write_text(result.stdout)
        (output / (name + '.stderr')).write_text(result.stderr)
        report['checks'].append(dict(name=name, command=list(map(str, command)), exit_code=result.returncode))
        save()
        if result.returncode:
            raise RuntimeError(f'{name}: {result.stderr}')
        return result.stdout

    def retain(name, source, expected=None):
        require_standalone(source)
        image = output / name
        shutil.copy2(source, image)
        result = subprocess.run([str(image)], cwd=output / 'empty', env={},
                                text=True, capture_output=True, check=True)
        (output / (name + '.stdout')).write_text(result.stdout)
        (output / (name + '.stderr')).write_text(result.stderr)
        poisoned = subprocess.run([str(image), 'invalid-runtime-source'], cwd=output / 'empty',
            env={'PERFECT_N_WORD': 'invalid', 'VOX_BAKED_INPUT_FILE': '/nonexistent',
                 'VOX_PHASE_MODULUS_WORD': 'invalid', 'VOX_PROBE_MODE': 'invalid'},
            input='invalid-runtime-source\n', text=True, capture_output=True, check=True)
        assert semantic_output(result.stdout) == semantic_output(poisoned.stdout), name
        if expected:
            expected(result.stdout)
        report['images'].append(dict(name=name, image=str(image),
            sha256=hashlib.sha256(image.read_bytes()).hexdigest(), static=True,
            empty_environment=True, empty_working_directory=True, runtime_overrides_ignored=True))
        save()
        print(name + ': static image and isolated execution passed', flush=True)

    for index in range(2):
        reference = fixture(224, index)
        case = output / f'source_{index}'
        case.mkdir()
        (case / 'reference.json').write_text(json.dumps(reference, indent=2) + '\n')
        number = reference['n']
        (case / 'input.decimal').write_text(number + '\n')
        word = call(f'encode_{index}', [ROOT / 'target/x86_64-unknown-linux-musl/release/vox', 'numeral', number]).strip()
        (case / 'input.imasm').write_text(word + '\n')
        report['sources'].append(dict(path=str(case / 'input.decimal'), bits=224,
                                      reference=str(case / 'reference.json')))
        for depth in (2, 4):
            env = dict(os.environ, RUSTFLAGS='-C target-feature=-crt-static',
                       CARGO_ENCODED_RUSTFLAGS='-C\x1ftarget-feature=-crt-static')
            stdout = call(f'perfect_build_{index}_{depth}', ['bash', ROOT / 'perfect_one.sh', number, depth], env)
            image = Path(stdout.strip().splitlines()[-1])

            def perfect(text):
                assert f'value in: {number}\n' in text
                assert f'value out: {number}\n' in text
                assert 'CLOSED (every level returned to root)' in text

            retain(f'perfect_{index}_{depth}', image, perfect)
        image, words = build_case(number, 'sparse', width=8)
        (case / 'python_baked_inputs.imasm').write_text('\n'.join(words) + '\n')

        def sparse(text):
            validate_sparse_output(number, text)

        retain(f'python_sparse_{index}', image, sparse)
    print('Receipt: ' + str(receipt))


if __name__ == '__main__':
    main()
