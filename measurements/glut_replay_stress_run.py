"""Measure supplied-witness replay independently of factor extraction."""
import json
import os
from pathlib import Path
import shutil
import subprocess
ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT/('membranes/glut_replay_stress_' + os.environ.get('GLUT_REPLAY_RUN', '2026-09-29'))
OUT.mkdir(exist_ok=True)
rows=[]
for shape, width in [('sparse', 128), ('sparse', 512), ('sparse', 2048), ('dense',128), ('dense',512)]:
    p = (1<<width)+3 if shape=='sparse' else (1<<width)-1
    q = (1<<(width+1))+9 if shape=='sparse' else (1<<(width+1))-1
    folder=OUT/f'{shape}_{width}'
    folder.mkdir(exist_ok=True)
    words=[subprocess.check_output([str(ROOT/'target/release/vox'),'numeral',str(x)],text=True).strip() for x in [p,q]]
    for name,word in zip(['p','q'], words): (folder/f'{name}.imasm').write_text(word+'\n')
    env=dict(os.environ, GLUT_REPLAY_P=words[0], GLUT_REPLAY_Q=words[1], RUSTFLAGS='-D warnings -C target-feature=+crt-static -C relocation-model=static')
    with (folder/'build.stdout').open('w') as log:
        subprocess.run(['cargo','build','--release','--target','x86_64-unknown-linux-musl','--example','glut_replay_stress_baked'],cwd=ROOT,env=env,stdout=log,stderr=log,check=True,timeout=60)
    binary=folder/'glut_replay_stress_baked'
    shutil.copy2(ROOT/'target/x86_64-unknown-linux-musl/release/examples/glut_replay_stress_baked',binary)
    with (folder/'native.stdout').open('w') as stdout,(folder/'native.stderr').open('w') as stderr:
        subprocess.run([str(binary)],stdout=stdout,stderr=stderr,check=True,timeout=60)
    row=dict(shape=shape,factor_width=width,p=str(p),q=str(q),product_bits=(p*q).bit_length(),output=(folder/'native.stdout').read_text())
    rows.append(row)
    (OUT/'manifest.json').write_text(json.dumps(rows,indent=2)+'\n')
    print(folder.name,row['output'].strip(),flush=True)
