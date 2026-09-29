"""Bake complete glut executables as glyph-only IMASM, and execute those words."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
ROOT=Path(__file__).resolve().parents[1]
VOX=ROOT/'target/release/vox'
OUT=ROOT/('membranes/glut_imasm_'+os.environ.get('GLUT_IMASM_RUN','2026-09-29_folded'))
OUT.mkdir(exist_ok=True)
ALPHABET=set('⊢⊣≻≺⋈⊤∈∋⊙⊥⊞⊡')
CASES=[('control_35',35), ('balanced_64',4294967291*4294967279),
       ('sparse_257',1<<256), ('unbalanced_258',3*((1<<256)+1))]
selected=os.environ.get('GLUT_IMASM_CASES')
if selected: CASES=[c for c in CASES if c[0] in selected.split(',')]
rows=[]
for name,n in CASES:
    folder=OUT/name
    folder.mkdir(exist_ok=True)
    glyph_path=folder/'payload.glyphs'
    if glyph_path.exists(): raise RuntimeError('Choose a fresh GLUT_IMASM_RUN to preserve existing executable words.')
    word=subprocess.check_output([str(VOX),'numeral',str(n)],text=True).strip()
    (folder/'input.imasm').write_text(word+'\n')
    env=dict(os.environ,GLUT_SOURCE_WORD=word,RUSTFLAGS='-C target-feature=+crt-static -C relocation-model=static')
    with (folder/'build.stdout').open('w') as log:
        subprocess.run(['cargo','build','--release','--target','x86_64-unknown-linux-musl','--bin','glut_one'],cwd=ROOT,env=env,stdout=log,stderr=log,check=True)
    elf=folder/'payload.elf'
    shutil.copy2(ROOT/'target/x86_64-unknown-linux-musl/release/glut_one',elf)
    native=subprocess.check_output([str(elf)],text=True)
    (folder/'native.stdout').write_text(native)
    assert len(native.splitlines())==4
    assert set(native.replace('\n','')) <= ALPHABET
    with (folder/'emission.stdout').open('w') as log:
        subprocess.run([str(VOX),'imasm',str(elf)],stdout=subprocess.DEVNULL,stderr=log,check=True)
        module=folder/'payload.elf.imasm'
        subprocess.run([str(VOX),'glyphs',str(module),str(glyph_path)],stdout=log,stderr=log,check=True)
        recovered=folder/'recovered.imasm'
        subprocess.run([str(VOX),'unglyphs',str(glyph_path),str(recovered)],stdout=log,stderr=log,check=True)
    assert module.read_bytes()==recovered.read_bytes()
    artifact=glyph_path.read_text()
    assert set(artifact) <= ALPHABET
    start=time.monotonic()
    with (folder/'imasm.stdout').open('w') as stdout,(folder/'imasm.stderr').open('w') as stderr:
        proc=subprocess.run([str(VOX),'run',str(glyph_path)],stdout=stdout,stderr=stderr,timeout=120)
    assert proc.returncode == 0, proc.returncode
    executed=(folder/'imasm.stdout').read_text()
    footer=next(line for line in executed.splitlines() if line.startswith('entry('))
    assert 'exited(0)' in footer, footer
    assert not (folder/'imasm.stderr').read_text()
    actual='\n'.join(line for line in executed.splitlines() if not line.startswith('entry('))+'\n'
    assert actual==native
    row=dict(name=name,source_bits=n.bit_length(),artifact_glyphs=len(artifact),
        artifact_sha256=hashlib.sha256(glyph_path.read_bytes()).hexdigest(),
        glyph_alphabet_only=True,complete_module_recovery=True,execution_exit=0,
        output_glyph_alphabet_only=True,output_matches_native=True,
        imasm_wall_seconds=time.monotonic()-start,vm_footer=footer)
    rows.append(row)
    (folder/'verification.json').write_text(json.dumps(row,indent=2)+'\n')
    (OUT/'manifest.json').write_text(json.dumps(rows,indent=2)+'\n')
    print(name,'IMASM executable and value alphabet verified;',footer,flush=True)
