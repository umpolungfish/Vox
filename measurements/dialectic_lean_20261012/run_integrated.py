"""N-only baked dialectic execution; verification follows factor production."""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[2]
WORK = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT))
from baked_case import build_case

SOURCES = sorted((ROOT / 'src').rglob('*.rs')) + [ROOT / 'baked_case.py', ROOT / 'build.rs',
           ROOT / 'Cargo.toml', ROOT / 'Cargo.lock',
           ROOT.parent / 'p4rakernel/p4ramill/KernelBridge.lean',
           ROOT.parent / 'p4rakernel/p4ramill/Imscribing/Paraconsistent/DialecticResident.lean',
           ROOT.parent / 'p4rakernel/p4ramill/Imscribing/Paraconsistent/CombinatoryReentry.lean',
           ROOT.parent / 'p4rakernel/p4ramill/Imscribing/Paraconsistent/CombinatoryDynamics.lean',
           ROOT.parent / 'p4rakernel/p4ramill/Imscribing/Paraconsistent/TemporalSemantics.lean',
           WORK / 'lean_plan.txt']


def execute(case, bound, repetitions, tag):
    source_path = WORK / 'inputs' / f'{case}.txt'
    if not source_path.exists():
        original = ROOT.parent / 'G-mOMonadOS/measurements/factor_source_carrier_20261011/inputs' / source_path.name
        shutil.copy2(original, source_path)
    n = int(source_path.read_text())
    retained = WORK / f'{tag}_{case}'
    snapshots = WORK / f'{tag}_{case}.source'
    bindings = []
    for source in SOURCES:
        relative = source.relative_to(ROOT.parent)
        copy = snapshots / relative
        copy.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, copy)
        bindings.append({'path': str(relative), 'sha256': hashlib.sha256(copy.read_bytes()).hexdigest()})
    begin = time.perf_counter()
    binary, _ = build_case(n, 'dialectic-lean')
    build_seconds = time.perf_counter() - begin
    for source, binding in zip(SOURCES, bindings):
        if hashlib.sha256(source.read_bytes()).hexdigest() != binding['sha256']:
            raise RuntimeError(f'source changed during build: {source}')
    shutil.copy2(binary, retained)
    runs = []
    for repeat in range(repetitions):
        stdout = WORK / f'{tag}_{case}.{repeat}.stdout'
        stderr = WORK / f'{tag}_{case}.{repeat}.stderr'
        timed_out = False
        begin = time.perf_counter()
        with stdout.open('w') as out, stderr.open('w') as err:
            process = subprocess.Popen([str(retained)], stdout=out, stderr=err)
            try:
                code = process.wait(timeout=bound)
            except subprocess.TimeoutExpired:
                timed_out = True
                process.kill()
                code = process.wait()
        elapsed = time.perf_counter() - begin
        text = stdout.read_text()
        factors = [int(line.split()[1]) for line in text.splitlines() if line.startswith('factor ')]
        verified = len(factors) == 2 and all(1 < factor < n for factor in factors)
        verified = verified and n % factors[0] == 0 and n // factors[0] == factors[1] and factors[0] * factors[1] == n
        measurements = {}
        for line in (text + '\n' + stderr.read_text()).splitlines():
            key, _, value = line.partition(' ')
            if key in ('preparation_seconds', 'trigger_to_readout_seconds', 'trigger_to_verified_factor_seconds'):
                measurements[key] = float(value)
        runs.append({'exit_status': code, 'timed_out': timed_out, 'bound_seconds': bound,
                     'driver_seconds': elapsed, 'product_verified': verified,
                     'measurements': measurements, 'stdout': str(stdout.relative_to(WORK)),
                     'stderr': str(stderr.relative_to(WORK))})
    receipt = {'source_n': str(n), 'source_bits': n.bit_length(), 'source_only_file': str(source_path.relative_to(WORK)),
               'runtime_arguments': [], 'wrapper_fixed_base': '2, unused by dialectic-lean',
               'build_seconds': build_seconds, 'binary_sha256': hashlib.sha256(retained.read_bytes()).hexdigest(),
               'source_stable_during_build': True,
               'rustc': subprocess.check_output(['rustc', '--version', '--verbose'], text=True),
               'driver_clock': 'includes process.wait timeout polling; native clocks are reported separately',
               'source_bindings': bindings, 'runs': runs}
    (WORK / f'{tag}_{case}.receipt.json').write_text(json.dumps(receipt, indent=2) + '\n')
    print(case, json.dumps(runs), flush=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--cases', nargs='+', required=True)
    parser.add_argument('--bound', type=float, default=90)
    parser.add_argument('--repetitions', type=int, default=1)
    parser.add_argument('--tag', default='carry')
    args = parser.parse_args()
    for case in args.cases:
        execute(case, args.bound, args.repetitions, args.tag)
