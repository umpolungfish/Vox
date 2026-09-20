"""Audit, lift, and execute baked factor cases through complete Vox membranes."""
import argparse
import fcntl
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import time
from hsoa_membrane_check import ROOT, call


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('reports', nargs='+')
    parser.add_argument('--bits', nargs='+', type=int, required=True)
    parser.add_argument('--family', default='close', choices=['close', 'multiplier'])
    parser.add_argument('--output', required=True)
    parser.add_argument('--run-vox', action='store_true', help='focused execution of the lifted module')
    args = parser.parse_args()
    rows = [json.loads(line) for path in args.reports for line in Path(path).read_text().splitlines()]
    selected = []
    for bits in args.bits:
        matches = [r for r in rows if r['bits']==bits and r['family']==args.family and r['sample']==0]
        if len(matches)!=1 or matches[0]['status']!='success':
            parser.error(f'need exactly one successful sample zero for {bits} bits / {args.family}')
        selected.append(matches[0])
    call(['cargo','build','--release','--bin','vox'])
    vox = ROOT/'target/release/vox'
    root = ROOT/'membranes/hsoa'/f'factor-{time.time_ns()}'
    root.mkdir(parents=True)
    with Path(args.output).open('x') as report:
        for source in selected:
            identity = hashlib.sha256(f"{source['producer']}:{source['n']}".encode()).hexdigest()
            case = root/f"{source['bits']}bit-{identity}"
            case.mkdir()
            env = os.environ.copy()
            env.pop('RUSTFLAGS',None)
            env.pop('VOX_PHASE_WIDTH_WORD',None)
            inputs = case/'inputs.imasm'
            inputs.write_text('\n'.join(source['imasm_inputs'])+'\n')
            env['VOX_BAKED_INPUT_FILE'] = str(inputs)
            env.pop('VOX_PHASE_MODULUS_WORD',None)
            env.pop('VOX_PHASE_BASE_WORD',None)
            env['VOX_PROBE_MODE'] = source['producer']
            binary = case/'payload.elf'
            with (ROOT/'target/baked-case.lock').open('a') as lock:
                fcntl.flock(lock,fcntl.LOCK_EX)
                call(['cargo','build','--release','--target','x86_64-unknown-linux-musl',
                      '--bin','semiprime_probe'],env=env)
                shutil.copy2(ROOT/'target/x86_64-unknown-linux-musl/release/semiprime_probe',binary)
            audit = call([vox,'lift',binary])
            (case/'audit.stdout').write_text(audit.stdout)
            (case/'audit.stderr').write_text(audit.stderr)
            tally = next(line.strip() for line in audit.stdout.splitlines() if 'verdicts  ' in line)
            assert re.search(r'\bF 0\s',tally),tally
            started = time.monotonic()
            compiled = call([binary],env={},input='')
            compiled_seconds = time.monotonic()-started
            (case/'compiled.stdout').write_text(compiled.stdout)
            (case/'compiled.stderr').write_text(compiled.stderr)
            expected = sorted([int(source['p']),int(source['q'])])
            factors = sorted(int(line.split()[1]) for line in compiled.stdout.splitlines() if line.startswith('factor '))
            assert factors==expected and factors[0]*factors[1]==int(source['n'])
            call([vox,'imasm',binary])
            module = Path(str(binary)+'.imasm')
            vm_seconds,agrees = None,None
            if args.run_vox:
                print(f"{source['bits']} bits / {args.family}: factors verified, audit {tally}; running Vox",flush=True)
                started = time.monotonic()
                executed = call([vox,'run',module],env={},input='')
                vm_seconds = time.monotonic()-started
                (case/'vox.stdout').write_text(executed.stdout)
                (case/'vox.stderr').write_text(executed.stderr)
                agrees = bool(re.search(r'^entry\(\.\.\.\) exited\(0\)',executed.stdout,re.M))
                actual = sorted(int(line.split()[1]) for line in executed.stdout.splitlines() if line.startswith('factor '))
                agrees &= actual==expected
            result = dict(bits=source['bits'],family=args.family,n=source['n'],
                          imasm_inputs=source['imasm_inputs'],producer=source['producer'],
                          compiled_seconds=compiled_seconds,vm_seconds=vm_seconds,
                          audit_summary=tally,vox_factors_verified=agrees,
                          binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),case=str(case))
            report.write(json.dumps(result)+'\n'); report.flush()
            print(f"{source['bits']} bits: audited and lifted; compiled execution={compiled_seconds:.6f}s; "
                  f"Vox factors verified={agrees}",flush=True)
            if agrees is False:
                raise SystemExit(f'failed stage; traces retained at {case}')


if __name__=='__main__':
    main()
