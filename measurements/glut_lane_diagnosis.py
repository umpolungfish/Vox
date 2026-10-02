"""Serial external lane observations; prepared carriers remain unchanged."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import uuid
ROOT = Path(__file__).resolve().parents[1]
diagnosis_id = uuid.uuid4().hex
label = os.environ.get('GLUT_DIAG_RUN')
OUT = ROOT/('membranes/glut_lane_diagnosis_'+(label+'_' if label else '')+diagnosis_id)
OUT.mkdir(exist_ok=False)
sources = ['Cargo.toml', 'src/glut_fold.rs', 'src/glut_frame_elimination.rs', 'src/glut_correlation.rs', 'src/glut_midpoint.rs',
           'src/glut_transport.rs', 'src/glut_quantum.rs', 'src/glut_square_phase.rs', 'src/glut_residue.rs', 'src/glut_support.rs', 'src/glut_frames.rs', 'src/glut_parity.rs', 'src/glut_square_fold.rs',
           'examples/glut_lane_diagnosis.rs']
snapshot = OUT/'source'; snapshot.mkdir()
hashes = {}
for name in sources:
    shutil.copy2(ROOT/name, snapshot/Path(name).name)
    hashes[name] = hashlib.sha256((ROOT/name).read_bytes()).hexdigest()
(OUT/'source_manifest.json').write_text(json.dumps(hashes, indent=2)+'\n')
word = (ROOT/os.environ.get('GLUT_DIAG_INPUT','tests/fixtures/random_rsa_2048.imasm')).read_text().strip()
for lane in os.environ.get('GLUT_DIAG_LANES','combined,square,masked,correlation').split(','):
    folder = OUT/lane; folder.mkdir()
    env = dict(os.environ, GLUT_SOURCE_WORD=word, GLUT_DIAGNOSTIC_LANE=lane,
               RUSTFLAGS='-D warnings')
    with (folder/'build.stdout').open('w') as log:
        subprocess.run(['cargo','build','--release','--features','root-work-profile','--example','glut_lane_diagnosis'],
                       cwd=ROOT,env=env,stdout=log,stderr=log,check=True,timeout=60)
    binary=folder/'diagnosis.elf'
    shutil.copy2(ROOT/'target/release/examples/glut_lane_diagnosis',binary)
    result=subprocess.run([str(binary)],capture_output=True,text=True,check=True,timeout=60)
    (folder/'diagnosis.stdout').write_text(result.stdout)
    (folder/'diagnosis.stderr').write_text(result.stderr)
    assert result.stderr == ''
    print(result.stdout.strip(),flush=True)
