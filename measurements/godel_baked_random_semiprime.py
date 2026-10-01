"""Bake independently generated semiprimes into native IMASM factor binaries."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]
if len(sys.argv) != 2:
    raise SystemExit(f'usage: {Path(sys.argv[0]).name} INPUT.jsonl')
INPUT = ROOT / sys.argv[1]
OUT = ROOT / (os.environ.get('GODEL_BAKED_RUN',
                             'membranes/godel_random_semiprime_baked_20261001'))
VOX = ROOT / 'target/release/vox'
GODEL = ROOT / 'target/release/godel'
ALPHABET = set('⊢⊣≻≺⋈⊤∈∋⊙⊥⊞⊡')


def run(args, *, cwd=ROOT, env=None, stdout=subprocess.PIPE):
    return subprocess.run(args, cwd=cwd, env=env, text=True, stdout=stdout,
                          stderr=subprocess.PIPE, check=True)


def numeral(value):
    word = run([str(VOX), 'numeral', value]).stdout.strip()
    if not word or not set(word) <= ALPHABET:
        raise RuntimeError('Vox did not emit a native IMASM numeral')
    return word


def cell_word(value):
    output = run([str(GODEL), 'encode', value]).stdout
    for line in output.splitlines():
        if line.startswith('word '):
            word = line.split(None, 1)[1]
            if set(word) <= ALPHABET:
                return word
    raise RuntimeError('godel encode did not emit its cell-binary word')


def main():
    fixtures = [json.loads(line) for line in INPUT.read_text().splitlines() if line]
    selected = set(filter(None, os.environ.get('GODEL_BAKED_CASES', '').split(',')))
    if selected:
        fixtures = [fixture for fixture in fixtures if fixture['name'] in selected]
        if {fixture['name'] for fixture in fixtures} != selected:
            raise RuntimeError('requested baked case is absent from the input batch')
    if (OUT / 'results.jsonl').exists():
        raise RuntimeError('Use a fresh output directory to preserve a completed run.')
    OUT.mkdir(parents=True, exist_ok=True)
    results = []
    for fixture in fixtures:
        name = fixture['name']
        n = fixture['n']
        case = OUT / name
        case.mkdir(exist_ok=True)
        word = numeral(n)
        (case / 'input.imasm').write_text(word + '\n')
        (case / 'fixture.json').write_text(json.dumps(fixture, indent=2) + '\n')

        env = dict(os.environ, FACTOR_N_WORD=word,
                   RUSTFLAGS='-D warnings')
        os.utime(ROOT / 'src/bin/factor_one.rs', None)
        build_start = time.monotonic()
        try:
            build = run(['cargo', 'build', '--release', '--bin', 'factor_one'], env=env)
        except subprocess.CalledProcessError as error:
            (case / 'build.stderr').write_text(error.stderr or '')
            raise
        build_seconds = time.monotonic() - build_start
        (case / 'build.stderr').write_text(build.stderr)
        binary = case / 'factor_one'
        shutil.copy2(ROOT / 'target/release/factor_one', binary)
        binary_digest = hashlib.sha256(binary.read_bytes()).hexdigest()

        emit = run([str(VOX), 'imasm', str(binary)], cwd=case,
                   stdout=subprocess.DEVNULL)
        module = Path(str(binary) + '.imasm')
        glyphs = case / f'factor_one.{binary_digest}.glyphs'
        recovered = case / f'recovered.{binary_digest}.imasm'
        if not glyphs.exists():
            run([str(VOX), 'glyphs', str(module), str(glyphs)])
        if not recovered.exists():
            run([str(VOX), 'unglyphs', str(glyphs), str(recovered)])
        if module.read_bytes() != recovered.read_bytes():
            raise RuntimeError(f'{name}: ELF-to-IMASM recovery did not close')

        start = time.monotonic()
        execution = run([str(binary)], cwd=case)
        elapsed = time.monotonic() - start
        (case / 'run.stdout').write_text(execution.stdout)
        (case / 'run.stderr').write_text(execution.stderr)
        if execution.stderr:
            raise RuntimeError(f'{name}: emitted output on stderr')
        lines = execution.stdout.splitlines()
        if len(lines) != 1 or ' = ' not in lines[0]:
            raise RuntimeError(f'{name}: unexpected factorizer output {execution.stdout!r}')
        lhs, rhs = lines[0].split(' = ', 1)
        factors = [part.strip() for part in rhs.split(' x ')]
        if lhs != n or len(factors) != 2:
            raise RuntimeError(f'{name}: factorizer did not return one pair')

        for factor in factors:
            answer = run([str(VOX), 'prime-check', factor]).stdout.strip()
            if answer != 'prime':
                raise RuntimeError(f'{name}: Vox primality read returned {answer!r}')
        product = run([str(GODEL), 'check', 'mul', cell_word(factors[0]),
                       cell_word(factors[1]), cell_word(n)]).stdout
        if not product.rstrip().endswith('status     PASS'):
            raise RuntimeError(f'{name}: cell-binary product did not close: {product}')

        result = dict(name=name, bits=fixture['bits'], n=n, factors=factors,
                      elapsed_seconds=elapsed, build_seconds=build_seconds,
                      engine='baked factor_one / Vox IMASM numeral',
                      factor_binary_sha256=binary_digest,
                      factor_word_primality='PASS', godel_product=product,
                      complete_imasm_recovery=True, binary_no_runtime_input=True,
                      closed=True)
        (case / 'result.json').write_text(json.dumps(result, indent=2) + '\n')
        results.append(result)
        with (OUT / 'results.jsonl').open('a') as output:
            output.write(json.dumps(result, sort_keys=True) + '\n')
        print(f"{fixture['bits']} bits closed in {elapsed:.3f}s; "
              f"build {build_seconds:.3f}s; binary {binary_digest}", flush=True)


if __name__ == '__main__':
    main()
