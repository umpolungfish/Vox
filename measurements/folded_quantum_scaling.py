"""Measure source-only prepared coherent carriers at increasing source widths."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/('membranes/folded_quantum_scaling_'+os.environ.get('FOLDED_SCALING_RUN','2026-09-30'))
OUT.mkdir(exist_ok=False)
VOX=ROOT/'target/release/vox'
def word(value):
    return subprocess.check_output([str(VOX),'numeral',str(value)],text=True,timeout=60).strip()
rows=[]
for n,p,q in [(21,3,7),(35,5,7),(77,7,11)]:
    folder=OUT/str(n);folder.mkdir()
    rounds=1<<(n.bit_length()-1)
    env=dict(os.environ,FACTOR_PHASE_SOURCE_WORD=word(n),FACTOR_PHASE_ROUNDS_WORD=word(rounds),
             FACTOR_PHASE_QUANTILE_WORD=word(1),FACTOR_PHASE_DENOMINATOR_WORD=word(2))
    (folder/'prepared.json').write_text(json.dumps(dict(n=n,rounds=rounds,quantile='1/2'),indent=2)+'\n')
    with (folder/'build.stdout').open('w') as log:
        subprocess.run(['cargo','build','--release','--target','x86_64-unknown-linux-musl','--example','factor_phase_baked'],
                       cwd=ROOT,env=env,stdout=log,stderr=log,check=True,timeout=60)
    elf=folder/'payload.elf'
    shutil.copy2(ROOT/'target/x86_64-unknown-linux-musl/release/examples/factor_phase_baked',elf)
    start=time.monotonic()
    with (folder/'native.stdout').open('w') as out,(folder/'native.stderr').open('w') as err:
        process=subprocess.Popen([str(elf)],stdout=out,stderr=err)
        try:
            process.wait(timeout=60);deadline=False
        except subprocess.TimeoutExpired:
            process.kill();process.wait();deadline=True
    elapsed=time.monotonic()-start
    output=(folder/'native.stdout').read_text()
    assert not (folder/'native.stderr').read_bytes(), 'Execution hard error: '+str(folder)
    if deadline: assert output=='', 'Premature prepared telemetry'
    else:
        assert process.returncode==0
        lines=output.splitlines();assert len(lines)==4
        assert lines[0]==word(n) and set(lines[1:3])=={word(p),word(q)}
        assert set(''.join(lines))<=set('⊢⊣≻≺⋈⊤∈∋⊙⊥⊞⊡')
    row=dict(n=n,width=n.bit_length(),rounds=rounds,closed=not deadline,native_seconds=elapsed,
             silent_until_closed=True,binary_sha256=hashlib.sha256(elf.read_bytes()).hexdigest())
    rows.append(row)
    (OUT/'manifest.json').write_text(json.dumps(rows,indent=2)+'\n')
    print(json.dumps(row),flush=True)
    if deadline: break
