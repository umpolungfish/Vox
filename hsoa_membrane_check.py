"""HSOA readout through compiled binary execution and serialized membrane via Vox VM."""
import fcntl
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import random
import sys
import numpy as np

if hasattr(sys, 'set_int_max_str_digits'):
    sys.set_int_max_str_digits(0)

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT.parent / 'G-mOMonadOS/hsoa_shor_state.py'
BLUEPRINT = ROOT.parent / 'ob3ect/digital/obtaining_informative_phase_observations_without_913a8f7c/obtaining_informative_phase_observations_without_913a8f7c_ob3ect.json'


def call(args, **kwargs):
    result = subprocess.run([str(x) for x in args], cwd=ROOT,
                            capture_output=True, text=True, **kwargs)
    if result.returncode:
        print(result.stdout,end='')
        print(result.stderr,end='',file=sys.stderr)
        result.check_returncode()
    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--stages', nargs='+', help='Escalating BITS:QUBITS balanced semiprime cases')
    parser.add_argument('--base-mode', choices=['two', 'minus-one'], default='two',
                        help='minus-one tests a public nontrivial return that cannot close factors')
    parser.add_argument('--compare-serialized', action='store_true',
                        help='explicit focused regression only; default is compiled execution plus audit/lift')
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location('hsoa_reference', SOURCE)
    hsoa = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(hsoa)
    call(['cargo', 'build', '--release', '--bin', 'vox', '--bin', 'phase_case_encode'])
    vox = ROOT / 'target/release/vox'
    encoder = ROOT / 'target/release/phase_case_encode'
    output = ROOT / 'membranes/hsoa' / str(time.time_ns())
    output.mkdir(parents=True)
    word = json.loads(BLUEPRINT.read_text())['glyph_word']
    cases = [(7,15,6), (2,21,6), (2,35,8),
             (2,34708385599522211756186926321308977827402135941491069,4)]
    if args.stages:
        from sympy import nextprime
        cases = []
        for stage in args.stages:
            bits, width = map(int, stage.split(':'))
            if bits < 175 or width < 1:
                parser.error('stages require at least 175 bits and a positive observation width')
            rng = random.Random(f'hsoa-battery-v1:{bits}:{width}')
            while True:
                p = int(nextprime(rng.randrange(1 << (bits//2-1), 1 << (bits//2))))
                q = int(nextprime(rng.randrange(1 << (bits-bits//2-1), 1 << (bits-bits//2))))
                if p != q and (p*q).bit_length() == bits:
                    break
            n = p*q
            cases.append((2 if args.base_mode == 'two' else n-1,n,width))
    failed = 0
    with (output / 'results.jsonl').open('x') as report:
        for a,n,width in cases:
            # Large decimal moduli can exceed filesystem component limits.
            identity = hashlib.sha256(f'{a}:{n}:{width}'.encode()).hexdigest()
            case = output / f'{n.bit_length()}bit-q{width}-{identity}'
            case.mkdir()
            words = call([encoder,a,n,width]).stdout.splitlines()
            env = os.environ.copy()
            inputs = case/'inputs.imasm'
            inputs.write_text('\n'.join([words[1],words[0],words[2]])+'\n')
            env['VOX_BAKED_INPUT_FILE'] = str(inputs)
            for key in ['VOX_PHASE_BASE_WORD','VOX_PHASE_MODULUS_WORD','VOX_PHASE_WIDTH_WORD']:
                env.pop(key,None)
            env['VOX_PHASE_EXECUTION_WORD'] = word
            env['VOX_PHASE_ACQUISITION'] = 'hsoa'
            # Self-contained musl startup needs no host dynamic-linker service.
            env.pop('RUSTFLAGS', None)
            binary = case / 'payload.elf'
            with (ROOT / 'target/baked-phase.lock').open('a') as lock:
                fcntl.flock(lock, fcntl.LOCK_EX)
                call(['cargo','build','--release','--target','x86_64-unknown-linux-musl','--bin','phase_baked'],env=env)
                shutil.copy2(ROOT / 'target/x86_64-unknown-linux-musl/release/phase_baked', binary)
            audit = call([vox,'lift',binary])
            (case/'audit.stdout').write_text(audit.stdout)
            (case/'audit.stderr').write_text(audit.stderr)
            audit_summary = next(line.strip() for line in audit.stdout.splitlines() if 'verdicts  ' in line)
            started = time.monotonic()
            native = call([binary],env={},input='')
            compiled_seconds = time.monotonic()-started
            (case/'native.stdout').write_text(native.stdout)
            (case/'native.stderr').write_text(native.stderr)
            # Independent state acquisition: no true_period call, no supplied r.
            state = np.array([complex(pow(a,x,n)==1) for x in range(1<<width)])
            population = np.count_nonzero(state)
            state /= np.linalg.norm(state)
            expected = abs(np.fft.fft(state)/np.sqrt(len(state)))**2
            measured = [float(line.split()[2]) for line in native.stdout.splitlines()
                        if line.startswith('probability ')]
            assert len(measured) == len(expected)
            assert np.isfinite(measured).all() and min(measured) >= 0
            assert abs(sum(measured)-1) < 1e-10
            assert np.allclose(measured,expected,atol=1e-10,rtol=0)
            assert abs(hsoa.frobenius_mu_delta(state)-1)<1e-10
            windings = [int(line.split()[1]) for line in native.stdout.splitlines() if line.startswith('winding ')]
            factors = [int(line.split()[1]) for line in native.stdout.splitlines() if line.startswith('factor ')]
            assert len(windings) <= 1
            for winding in windings:
                assert population > 1 and winding > 0 and pow(a,winding,n) == 1
            if factors:
                assert windings and len(factors)==2 and factors[0]*factors[1]==n and min(factors)>1
            status = 'factored' if factors else ('return_without_factors' if windings else 'unresolved')
            if not windings:
                assert 'unresolved:' in native.stdout
            if n in (15,21,35):
                assert status == 'factored'
                assert windings == [hsoa.winding_number(state,a,n)]
            if a == n-1 and width >= 2:
                assert windings == [2] and status == 'return_without_factors'
            print(f'{n.bit_length()} bits / width {width}: compiled spectrum verified, {status}; lifting',flush=True)
            call([vox,'imasm',binary])
            module = Path(str(binary)+'.imasm')
            if not args.compare_serialized:
                row = dict(a=a,n=str(n),bits=n.bit_length(),width=width,population=int(population),
                           status=status,words=words,execution_word=word,audit_summary=audit_summary,
                           compiled_seconds=compiled_seconds,vm_seconds=None,vm_executed=False,
                           binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),case=str(case))
                report.write(json.dumps(row)+'\n'); report.flush()
                print(f'{n.bit_length()} bits: audited and lifted; compiled binary execution={compiled_seconds:.6f}s',flush=True)
                continue
            glyphs = case/'payload.glyphs'
            recovered = case/'recovered.imasm'
            call([vox,'glyphs',module,glyphs])
            call([vox,'unglyphs',glyphs,recovered])
            assert module.read_bytes()==recovered.read_bytes()
            started = time.monotonic()
            executed = call([vox,'run',glyphs],env={},input='')
            vm_seconds = time.monotonic()-started
            (case/'vox.stdout').write_text(executed.stdout)
            (case/'vox.stderr').write_text(executed.stderr)
            cleaned = re.sub(r'^entry\(\.\.\.\) exited\(0\)[^\n]*\n?', '',executed.stdout,flags=re.M)
            agrees = bool(re.search(r'^entry\(\.\.\.\) exited\(0\)',executed.stdout,re.M)) and cleaned==native.stdout and executed.stderr==native.stderr
            failed += int(not agrees)
            row=dict(a=a,n=str(n),width=width,population=int(population),words=words,
                     bits=n.bit_length(),status=status,compiled_seconds=compiled_seconds,vm_seconds=vm_seconds,
                     audit_summary=audit_summary,
                     execution_word=word,native_vm_equal=agrees,exact_module_recovery=True,
                     vm_first_line=executed.stdout.splitlines()[0] if executed.stdout else '',
                     binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                     reference_sha256=hashlib.sha256(SOURCE.read_bytes()).hexdigest(),case=str(case))
            report.write(json.dumps(row)+'\n'); report.flush()
            print(f'{n.bit_length()} bits / width {width}: complete membrane agreement={agrees}; '
                  f'compiled binary execution={compiled_seconds:.6f}s; '
                  f'serialized membrane via Vox VM={vm_seconds:.6f}s',flush=True)
            if not agrees:
                raise SystemExit(f'Failed stage; traces retained at {case}')
    print(output/'results.jsonl')
    if failed:
        raise SystemExit(f'{failed} complete-membrane checks failed; native controls and failure traces retained')


if __name__=='__main__':
    main()
