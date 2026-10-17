#!/usr/bin/env python3
"""Check documented CPU commands and optionally bake CPU membrane families."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
from datetime import datetime

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from standalone import require_standalone


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--bake', action='store_true', help='also bake word, factor, resident, phase and ququart images; never run factor searches')
    parser.add_argument('--output', type=Path, help='fresh directory for logs and receipts')
    args = parser.parse_args()
    output = (args.output or ROOT / 'measurements/program_operations_20261015' /
              datetime.now().strftime('check_%Y%m%d_%H%M%S')).resolve()
    output.mkdir(parents=True, exist_ok=False)
    gm = ROOT.parent / 'G-mOMonadOS'
    source = ROOT / 'measurements/static_bakers_20261015/source.decimal'
    word = source.with_suffix('.imasm')
    number = source.read_text().strip()
    if int(number).bit_length() < 200:
        raise ValueError('The validation source must be at least 200 bits.')
    rows = []

    def check(name, command, image=None, env=None):
        with (output / f'{name}.stdout').open('w') as stdout, (output / f'{name}.stderr').open('w') as stderr:
            result = subprocess.run(list(map(str, command)), cwd=ROOT.parent,
                                    stdout=stdout, stderr=stderr, env=env)
        row = dict(name=name, command=list(map(str, command)), exit_code=result.returncode)
        if result.returncode == 0 and image:
            require_standalone(image)
            row.update(image=str(image), static=True,
                       sha256=hashlib.sha256(image.read_bytes()).hexdigest())
        rows.append(row)
        (output / 'receipt.json').write_text(json.dumps(dict(source=str(source),
            source_bits=int(number).bit_length(), gpu_execution=False, checks=rows), indent=2) + '\n')
        print(f'{name}: exit {result.returncode}', flush=True)
        if result.returncode:
            raise RuntimeError(f'{name} failed. Read {output / (name + ".stderr")}')

    for repo in (ROOT, gm):
        launcher = repo / 'programs'
        for name, arguments in [('help', ['--help']), ('catalog', ['list', '--all', '--json']),
                                ('bakers', ['bake', '--list']), ('bake_help', ['bake', '--help']),
                                ('cli_help', ['--no-build', 'cli', '--help'])]:
            check(f'{repo.name}_{name}', [launcher, *arguments])
        arguments = ['numeral', number] if repo == ROOT else ['numeral', 'encode', number]
        check(f'{repo.name}_numeral', [launcher, '--no-build', 'cli', *arguments])
        check(f'{repo.name}_menu', ['bash', '-c', 'printf "q\\n" | "$1" menu', 'menu-check', launcher])
    for family in (['word', 'factor', 'resident', 'rust-phase', 'c-phase', 'ququart'] if args.bake else []):
        destination = output / family
        command = [gm / 'programs', 'bake', family, '--output', destination]
        command += ['--word-file', word] if family in ('word', 'ququart') else ['--input', source]
        if family == 'ququart':
            command += ['--native-fourier']
        image = destination / {'factor': 'factor_one', 'resident': 'resident_factor_baked',
                               'ququart': 'membrane'}[family] if family in ('factor', 'resident', 'ququart') else destination
        check(f'bake_{family}', command, image)
        if family == 'word':
            isolated = output / 'empty'
            isolated.mkdir()
            with (output / 'word_isolated.stdout').open('w') as stdout:
                result = subprocess.run([str(image)], cwd=isolated, env={}, stdout=stdout)
            if result.returncode:
                raise RuntimeError('Isolated word execution failed')
    print(f'Receipt: {output / "receipt.json"}')


if __name__ == '__main__':
    main()
