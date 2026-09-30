"""Baked glut extraction on reproducible independent-prime RSA-style fixtures."""
import fcntl
import hashlib
import json
import math
import os
from pathlib import Path
import random
import resource
import shutil
import signal
import subprocess
import time
import sympy
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/('membranes/glut_rsa_'+os.environ.get('GLUT_RSA_RUN','2026-09-29_baseline'))
OUT.mkdir(exist_ok=True)
SECONDS=min(60.0, float(os.environ.get('GLUT_RSA_SECONDS','8')))
WIDTHS=[16,24,32,40,48,56,64,96,128,256,512,1024,2048]
selected=os.environ.get('GLUT_RSA_CASES')
selected=set(selected.split(',')) if selected else None
rows=[]
SOURCE_FILES=['src/morphism_factor.rs','src/glut_fold.rs','src/glut_system.rs','examples/glut_stress_baked.rs','src/glut_parity.rs', 'src/glut_transport.rs', 'src/glut_correlation.rs','src/glut_square_fold.rs','src/glut_midpoint.rs']
sources={name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in SOURCE_FILES}
snapshot=OUT/'source'
snapshot.mkdir(exist_ok=True)
for name in SOURCE_FILES: shutil.copy2(ROOT/name,snapshot/Path(name).name)
(OUT/'source_manifest.json').write_text(json.dumps(sources,indent=2)+'\n')

def fixture(bits,index):
    seed=hashlib.sha256(f'glut-rsa-independent-v1:{bits}:{index}'.encode()).hexdigest()
    rng=random.Random(int(seed,16)); width=bits//2
    while True:
        primes=[]
        for _ in range(2):
            candidate=rng.getrandbits(width) | (1 << (width-1)) | 1
            prime=int(sympy.nextprime(candidate))
            if prime.bit_length()!=width or math.gcd(prime-1,65537)!=1: break
            primes.append(prime)
        if len(primes)!=2: continue
        p,q=sorted(primes)
        if p==q or (p*q).bit_length()!=bits: continue
        # Reject deliberately close pairs. This is parent-only fixture selection.
        if q-p < (1 << (width//2)): continue
        assert sympy.isprime(p) and sympy.isprime(q)
        return dict(seed=seed,p=str(p),q=str(q),n=str(p*q),bits=bits,
            factor_bits=width,factor_gap_bits=(q-p).bit_length(),
            public_exponent=65537,primality='deterministic' if width<=64 else 'probable prime',
            primality_instrument='SymPy '+sympy.__version__)

def child_limits():
    resource.setrlimit(resource.RLIMIT_AS,(768*1024*1024,768*1024*1024))
    resource.setrlimit(resource.RLIMIT_CORE,(0,0))

for bits in WIDTHS:
    for index in range(3 if 32<=bits<=64 else 1):
        name=f'rsa_{bits}_{index}'
        if selected and name not in selected: continue
        folder=OUT/name
        folder.mkdir(exist_ok=True)
        if (folder/'measurement.json').exists(): raise RuntimeError('Use a fresh GLUT_RSA_RUN to preserve measurements.')
        row=dict(name=name,**fixture(bits,index))
        word=subprocess.check_output([str(ROOT/'target/release/vox'),'numeral',row['n']],text=True).strip()
        (folder/'input.imasm').write_text(word+'\n')
        (folder/'fixture.json').write_text(json.dumps(row,indent=2)+'\n')
        env=dict(os.environ,GLUT_STRESS_WORD=word,RUSTFLAGS='-D warnings -C target-feature=+crt-static -C relocation-model=static')
        binary=folder/'glut_stress_baked'
        with (ROOT/'measurements/glut_bake.lock').open('a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX)
            assert all(hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest for name,digest in sources.items()), 'Runtime source changed during campaign.'
            with (folder/'build.stdout').open('w') as log:
                subprocess.run(['cargo','build','--release','--target','x86_64-unknown-linux-musl','--example','glut_stress_baked'],cwd=ROOT,env=env,stdout=log,stderr=log,check=True,timeout=60)
            shutil.copy2(ROOT/'target/x86_64-unknown-linux-musl/release/examples/glut_stress_baked',binary)
        row['source_hashes']=sources
        row['binary_sha256']=hashlib.sha256(binary.read_bytes()).hexdigest()
        start=time.monotonic(); rss=0; interrupted=False
        with (folder/'native.stdout').open('w') as stdout,(folder/'native.stderr').open('w') as stderr:
            proc=subprocess.Popen([str(binary)],stdout=stdout,stderr=stderr,preexec_fn=child_limits,start_new_session=True)
            while proc.poll() is None:
                try:
                    for line in Path(f'/proc/{proc.pid}/status').read_text().splitlines():
                        if line.startswith('VmHWM:'): rss=max(rss,int(line.split()[1]))
                except FileNotFoundError: pass
                if time.monotonic()-start>SECONDS:
                    os.killpg(proc.pid,signal.SIGTERM)
                    try: proc.wait(timeout=1)
                    except subprocess.TimeoutExpired: os.killpg(proc.pid,signal.SIGKILL); proc.wait()
                    interrupted=True; break
                time.sleep(.02)
        lines=(folder/'native.stdout').read_text().splitlines()
        result=next((line.split('\t')[1:] for line in lines if line.startswith('RESULT\t')),None)
        fold=next((line.split('\t')[1:] for line in reversed(lines) if line.startswith('FOLD\t')),None)
        rail=next((line.split('\t')[1:] for line in reversed(lines) if line.startswith('RAIL\t')),None)
        correlation=next((line.split('\t')[1:] for line in reversed(lines) if line.startswith('CORRELATION\t')),None)
        square=next((line.split('\t')[1:] for line in reversed(lines) if line.startswith('SQUARE\t')),None)
        row.update(last_square=square,last_correlation=correlation,last_rail=rail,returncode=proc.returncode,interrupted=interrupted,wall_seconds=time.monotonic()-start,
            process_seconds=SECONDS,process_address_bytes=768*1024*1024,sampled_rss_peak_kib=rss,last_fold=fold,result=result)
        if result:
            assert proc.returncode==0 and result[0]=='closed', result
            assert sorted(map(int,result[1:3]))==[int(row['p']),int(row['q'])]
            assert int(result[1])*int(result[2])==int(row['n'])
        elif not interrupted: raise RuntimeError(f'{name} exited without closure: {proc.returncode}')
        (folder/'measurement.json').write_text(json.dumps(row,indent=2)+'\n')
        rows.append(row); (OUT/'manifest.json').write_text(json.dumps(rows,indent=2)+'\n')
        print(name,'closed '+result[3]+' us' if result else 'observation interrupted', 'last_fold',fold,flush=True)
