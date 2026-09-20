"""HSOA readout control through native and full serialized IMASM membranes."""
import fcntl
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import time
import numpy as np

ROOT = Path(__file__).resolve().parent
SOURCE = ROOT.parent / 'G-mOMonadOS/hsoa_shor_state.py'
BLUEPRINT = ROOT.parent / 'ob3ect/digital/obtaining_informative_phase_observations_without_913a8f7c/obtaining_informative_phase_observations_without_913a8f7c_ob3ect.json'


def call(args, **kwargs):
    return subprocess.run([str(x) for x in args], cwd=ROOT, check=True,
                          capture_output=True, text=True, **kwargs)


def main():
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
    failed = 0
    with (output / 'results.jsonl').open('x') as report:
        for a,n,width in cases:
            case = output / f'{a}-{n}-{width}'
            case.mkdir()
            words = call([encoder,a,n,width]).stdout.splitlines()
            env = os.environ.copy()
            env.update(zip(['VOX_PHASE_BASE_WORD','VOX_PHASE_MODULUS_WORD','VOX_PHASE_WIDTH_WORD'], words))
            env['VOX_PHASE_EXECUTION_WORD'] = word
            env['VOX_PHASE_ACQUISITION'] = 'hsoa'
            # Self-contained musl startup needs no host dynamic-linker service.
            env.pop('RUSTFLAGS', None)
            binary = case / 'payload.elf'
            with (ROOT / 'target/baked-phase.lock').open('a') as lock:
                fcntl.flock(lock, fcntl.LOCK_EX)
                call(['cargo','build','--release','--target','x86_64-unknown-linux-musl','--bin','phase_baked'],env=env)
                shutil.copy2(ROOT / 'target/x86_64-unknown-linux-musl/release/phase_baked', binary)
            native = call([binary],env={},input='')
            (case/'native.stdout').write_text(native.stdout)
            (case/'native.stderr').write_text(native.stderr)
            # Independent state acquisition: no true_period call, no supplied r.
            state = np.array([complex(pow(a,x,n)==1) for x in range(1<<width)])
            population = np.count_nonzero(state)
            state /= np.linalg.norm(state)
            expected = abs(np.fft.fft(state)/np.sqrt(len(state)))**2
            measured = [float(line.split()[2]) for line in native.stdout.splitlines()
                        if line.startswith('probability ')]
            assert np.allclose(measured,expected,atol=1e-10,rtol=0)
            assert abs(hsoa.frobenius_mu_delta(state)-1)<1e-10
            if population>1:
                winding = hsoa.winding_number(state,a,n)
                assert f'winding {winding}\n' in native.stdout
                factors=[int(line.split()[1]) for line in native.stdout.splitlines() if line.startswith('factor ')]
                assert len(factors)==2 and factors[0]*factors[1]==n and min(factors)>1
            else:
                assert 'winding ' not in native.stdout and 'factor ' not in native.stdout
            print(f'{n}: native agrees with HSOA spectrum; lifting',flush=True)
            call([vox,'imasm',binary])
            module = Path(str(binary)+'.imasm')
            glyphs = case/'payload.glyphs'
            recovered = case/'recovered.imasm'
            call([vox,'glyphs',module,glyphs])
            call([vox,'unglyphs',glyphs,recovered])
            assert module.read_bytes()==recovered.read_bytes()
            executed = call([vox,'run',glyphs],env={},input='')
            (case/'vox.stdout').write_text(executed.stdout)
            (case/'vox.stderr').write_text(executed.stderr)
            cleaned = re.sub(r'^entry\(\.\.\.\) exited\(0\)[^\n]*\n?', '',executed.stdout,flags=re.M)
            agrees = bool(re.search(r'^entry\(\.\.\.\) exited\(0\)',executed.stdout,re.M)) and cleaned==native.stdout and executed.stderr==native.stderr
            failed += int(not agrees)
            row=dict(a=a,n=str(n),width=width,population=int(population),words=words,
                     execution_word=word,native_vm_equal=agrees,exact_module_recovery=True,
                     vm_first_line=executed.stdout.splitlines()[0] if executed.stdout else '',
                     binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                     reference_sha256=hashlib.sha256(SOURCE.read_bytes()).hexdigest(),case=str(case))
            report.write(json.dumps(row)+'\n'); report.flush()
            print(f'{n}: complete glyph membrane native agreement={agrees}',flush=True)
    print(output/'results.jsonl')
    if failed:
        raise SystemExit(f'{failed} complete-membrane checks failed; native controls and failure traces retained')


if __name__=='__main__':
    main()
