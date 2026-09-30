"""Execute complete RSA-style glut glyph carriers, retaining open observations."""
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import time
ROOT=Path(__file__).resolve().parents[1]
VOX=ROOT/'target/release/vox'
OUT=ROOT/('membranes/glut_rsa_imasm_'+os.environ.get('GLUT_RSA_IMASM_RUN','2026-09-29'))
OUT.mkdir(exist_ok=True)
ALPHABET=set('⊢⊣≻≺⋈⊤∈∋⊙⊥⊞⊡')
SECONDS=float(os.environ.get('GLUT_RSA_IMASM_SECONDS','60'))
NATIVE_SECONDS=float(os.environ.get('GLUT_RSA_NATIVE_SECONDS','8'))
rows=[]

def observe(binary,folder,label,args,seconds):
    start=time.monotonic()
    with (folder/(label+'.stdout')).open('w') as stdout,(folder/(label+'.stderr')).open('w') as stderr:
        proc=subprocess.Popen([str(binary),*args],stdout=stdout,stderr=stderr,start_new_session=True)
        try: proc.wait(timeout=seconds); interrupted=False
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid,signal.SIGTERM)
            try: proc.wait(timeout=1)
            except subprocess.TimeoutExpired: os.killpg(proc.pid,signal.SIGKILL); proc.wait()
            interrupted=True
    return dict(returncode=proc.returncode,interrupted=interrupted,wall_seconds=time.monotonic()-start)

for name in os.environ.get('GLUT_RSA_IMASM_CASES','rsa_48_0,rsa_2048_0').split(','):
    candidates=[ROOT/f'membranes/glut_rsa_{run}/{name}/fixture.json' for run in ['2026-09-29_verified','2026-09-29_baseline']]
    fixture=json.loads(next(p for p in candidates if p.exists()).read_text())
    folder=OUT/name
    folder.mkdir(exist_ok=True)
    glyph=folder/'payload.glyphs'
    if glyph.exists(): raise RuntimeError('Use a fresh GLUT_RSA_IMASM_RUN to preserve words.')
    word=subprocess.check_output([str(VOX),'numeral',fixture['n']],text=True).strip()
    (folder/'input.imasm').write_text(word+'\n')
    env=dict(os.environ,GLUT_SOURCE_WORD=word,GLUT_OBSERVE_WORD='⊥',RUSTFLAGS='-C target-feature=+crt-static -C relocation-model=static')
    source_names=['src/glut_fold.rs','src/glut_correlation.rs','src/glut_square_fold.rs','src/glut_system.rs','src/bin/glut_one.rs']
    source=folder/'source';source.mkdir(exist_ok=True)
    hashes={name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in source_names}
    for name in source_names: shutil.copy2(ROOT/name,source/Path(name).name)
    (folder/'source_manifest.json').write_text(json.dumps(hashes,indent=2)+'\n')
    with (ROOT/'measurements/glut_bake.lock').open('a') as lock:
        fcntl.flock(lock,fcntl.LOCK_EX)
        with (folder/'build.stdout').open('w') as log:
            subprocess.run(['cargo','build','--release','--target','x86_64-unknown-linux-musl','--bin','glut_one'],cwd=ROOT,env=env,stdout=log,stderr=log,check=True)
        assert all(hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest for name,digest in hashes.items()), 'Runtime source changed during bake.'
        elf=folder/'payload.elf'; shutil.copy2(ROOT/'target/x86_64-unknown-linux-musl/release/glut_one',elf)
    native=observe(elf,folder,'native',[],NATIVE_SECONDS)
    native_closed=native['returncode']==0 and not native['interrupted']
    native_text=(folder/'native.stdout').read_text()
    assert set(native_text.replace('\n',''))<=ALPHABET
    if native_closed:
        assert native['returncode']==0 and len(native_text.splitlines())>=5
        expected=[subprocess.check_output([str(VOX),'numeral',fixture[k]],text=True).strip() for k in ['n','p','q']]
        assert native_text.splitlines()[-4:-1]==expected
    with (folder/'emission.stdout').open('w') as log:
        subprocess.run([str(VOX),'imasm',str(elf)],stdout=subprocess.DEVNULL,stderr=log,check=True)
        module=folder/'payload.elf.imasm'; recovered=folder/'recovered.imasm'
        subprocess.run([str(VOX),'glyphs',str(module),str(glyph)],stdout=log,stderr=log,check=True)
        subprocess.run([str(VOX),'unglyphs',str(glyph),str(recovered)],stdout=log,stderr=log,check=True)
    assert module.read_bytes()==recovered.read_bytes()
    artifact=glyph.read_text(); assert set(artifact)<=ALPHABET
    vm=observe(VOX,folder,'imasm',['run',str(glyph)],SECONDS)
    (folder/'observations.json').write_text(json.dumps(dict(native=native,imasm=vm),indent=2)+'\n')
    executed=(folder/'imasm.stdout').read_text()
    lines=executed.splitlines()
    footers=[line for line in lines if line.startswith('entry(')]
    actual='\n'.join(line for line in lines if not line.startswith('entry('))+'\n' if lines else ''
    assert set(actual.replace('\n',''))<=ALPHABET
    assert not (folder/'imasm.stderr').read_text()
    actual=executed if not footers else actual
    assert actual.startswith(word+'\n')
    shorter=min(len(actual),len(native_text))
    assert actual[:shorter]==native_text[:shorter], 'VM boundary transport differs from native'
    assert len(actual.splitlines())>=12, 'VM must execute a complete eleven-word boundary record'
    if vm['interrupted']:
        status='open IMASM observation'
    else:
        assert vm['returncode']==0 and len(footers)==1 and 'exited(0)' in footers[0]
        if native_closed: assert actual==native_text
        expected=[subprocess.check_output([str(VOX),'numeral',fixture[k]],text=True).strip() for k in ['n','p','q']]
        assert actual.splitlines()[-4:-1]==expected
        status='verified IMASM closure'
    row=dict(name=name,fixture=fixture,status=status,native_closed=native_closed,native=native,imasm=vm,
        alphabet_only=True,complete_module_recovery=True,output_alphabet_only=True,
        native_prefix_agreement=True,output_lines=len(actual.splitlines()),
        artifact_sha256=hashlib.sha256(glyph.read_bytes()).hexdigest(),artifact_glyphs=len(artifact),
        vm_footer=footers[0] if footers else None)
    rows.append(row); (folder/'verification.json').write_text(json.dumps(row,indent=2)+'\n')
    (OUT/'manifest.json').write_text(json.dumps(rows,indent=2)+'\n')
    print(name,status,'IMASM word, values and executed progress verified;',len(actual.splitlines()),'output words',flush=True)
shutil.copy2(VOX,OUT/'vox')
(OUT/'vox.sha256').write_text(hashlib.sha256((OUT/'vox').read_bytes()).hexdigest()+'  vox\n')
