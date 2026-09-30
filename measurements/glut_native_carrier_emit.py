"""Retain complete IMASM carriers and their native closure status."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
ROOT=Path(__file__).resolve().parents[1]
run=ROOT/('membranes/glut_rsa_native_'+os.environ['GLUT_RSA_RUN'])
vox=ROOT/'target/release/vox'
alphabet=set('⊢⊣≻≺⋈⊤∈∋⊙⊥⊞⊡')
rows=[]
for row in json.loads((run/'manifest.json').read_text()):
    folder=run/row['name']; elf=folder/'payload.elf'
    module=folder/'payload.elf.imasm'; glyph=folder/'payload.glyphs'; recovered=folder/'recovered.imasm'
    if glyph.exists(): raise RuntimeError('Retain the existing emission; use a fresh run for changes.')
    with (folder/'emission.stdout').open('w') as log:
        subprocess.run([str(vox),'imasm',str(elf)],stdout=subprocess.DEVNULL,stderr=log,check=True,timeout=60)
        subprocess.run([str(vox),'glyphs',str(module),str(glyph)],stdout=log,stderr=log,check=True,timeout=60)
        subprocess.run([str(vox),'unglyphs',str(glyph),str(recovered)],stdout=log,stderr=log,check=True,timeout=60)
    assert module.read_bytes()==recovered.read_bytes()
    assert set(glyph.read_text())<=alphabet
    assert set((folder/'native.stdout').read_text().replace('\n',''))<=alphabet
    verified=dict(name=row['name'],native_closed=row['closed'],complete_module_recovery=True,
                  artifact_alphabet_only=True,native_output_alphabet_only=True,
                  vm_execution_performed=False,binary_sha256=row['binary_sha256'],
                  artifact_sha256=hashlib.sha256(glyph.read_bytes()).hexdigest())
    (folder/'emission_verification.json').write_text(json.dumps(verified,indent=2)+'\n')
    rows.append(verified); print(row['name'],'complete IMASM emission verified',flush=True)
(run/'emission_manifest.json').write_text(json.dumps(rows,indent=2)+'\n')
