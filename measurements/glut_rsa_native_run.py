"""Measure silent prepared RSA carriers directly on the CPU."""
import fcntl
import hashlib
import json
import os
import re
from pathlib import Path
import shutil
import subprocess
import time
import uuid
from random_rsa_fixtures import fixture as random_fixture
ROOT = Path(__file__).resolve().parents[1]
RUN_ID = uuid.uuid4().hex
RUN_LABEL = os.environ.get('GLUT_RSA_RUN')
RUN_NAME = (RUN_LABEL+'_' if RUN_LABEL else '')+RUN_ID
OUT = ROOT / ('membranes/glut_rsa_native_' + RUN_NAME)
OUT.mkdir(exist_ok=False)
(OUT/'campaign.json').write_text(json.dumps(dict(campaign_id=RUN_ID, run_name=RUN_NAME,
                                                label=RUN_LABEL), indent=2)+'\n')
print('campaign', RUN_NAME, flush=True)
SECONDS = min(60.0, float(os.environ.get('GLUT_RSA_SECONDS', '60')))
VOX = ROOT / 'target/release/vox'
SOURCES = ['src/bin/m3mbrain.rs', 'src/glut_system.rs', 'src/glut_carrier.rs', 'src/phase_word.rs', 'src/fixed_point_hypernest.rs', 'src/glut_fold.rs', 'src/glut_frame_elimination.rs', 'src/glut_parity.rs', 'src/glut_quantum.rs', 'src/glut_transport.rs', 'src/glut_correlation.rs', 'src/glut_midpoint.rs', 'src/glut_square_fold.rs', 'src/glut_square_phase.rs', 'src/glut_frames.rs', 'src/factor_2adic.rs', 'src/morphism_factor.rs', 'measurements/random_rsa_fixtures.py']
hashes = {name: hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in SOURCES}
source = OUT/'source'; source.mkdir()
for name in SOURCES: shutil.copy2(ROOT/name, source/Path(name).name)
(OUT/'source_manifest.json').write_text(json.dumps(hashes, indent=2)+'\n')
rows = []
replay_ids = json.loads(os.environ.get('GLUT_RSA_FIXTURE_IDS', '{}'))
for case in os.environ.get('GLUT_RSA_CASES', 'rsa_128_0,rsa_256_0,rsa_512_0,rsa_1024_0,rsa_2048_0').split(','):
    identity = re.fullmatch(r'rsa_(\d+)_(\d+)', case)
    if identity is None:
        raise ValueError('Factor tests require independently generated random RSA fixtures: '+case)
    bits, index = map(int, identity.groups())
    fixture = random_fixture(bits, index, replay_ids.get(case))
    case = fixture['name']
    folder = OUT/case; folder.mkdir()
    (folder/'fixture.json').write_text(json.dumps(fixture, indent=2)+'\n')
    expected = [subprocess.check_output([str(VOX), 'numeral', fixture[k]], text=True, timeout=60).strip() for k in ['n','p','q']]
    (folder/'input.imasm').write_text(expected[0]+'\n')
    env = dict(os.environ, VOX_PHASE_MODULUS_WORD=expected[0], RUSTFLAGS='-D warnings -C target-feature=+crt-static -C relocation-model=static')
    with (ROOT/'measurements/glut_bake.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        assert all(hashlib.sha256((ROOT/name).read_bytes()).hexdigest()==digest for name,digest in hashes.items())
        with (folder/'build.stdout').open('w') as log:
            subprocess.run(['cargo','build','--release','--target','x86_64-unknown-linux-musl','--bin','m3mbrain'], cwd=ROOT, env=env, stdout=log, stderr=log, check=True, timeout=60)
        binary = folder/'payload.elf'; shutil.copy2(ROOT/'target/x86_64-unknown-linux-musl/release/m3mbrain',binary)
    start = time.monotonic()
    with (folder/'native.stdout').open('w') as out, (folder/'native.stderr').open('w') as err:
        process = subprocess.Popen([str(binary)], stdout=out, stderr=err)
        try: process.wait(timeout=SECONDS); interrupted=False
        except subprocess.TimeoutExpired: process.kill(); process.wait(); interrupted=True
    elapsed = time.monotonic()-start
    output = (folder/'native.stdout').read_text()
    assert not (folder/'native.stderr').read_text()
    if interrupted: assert output == '', 'Prepared carrier emitted before closure'
    else:
        assert process.returncode == 0
        assert len(output.splitlines()) == 4 and output.splitlines()[:3] == expected
        assert set(output.replace('\n','')) <= set('⊢⊣≻≺⋈⊤∈∋⊙⊥⊞⊡')
    row = dict(name=case, expected_fixture=fixture, extracted_factor_words=output.splitlines()[1:3] if not interrupted else None, returncode=process.returncode, interrupted=interrupted,
               wall_seconds=elapsed, deadline_seconds=SECONDS, closed=not interrupted,
               silent_until_closure=True, emitter='m3mbrain', resident_dqi_syndrome=True, carry_reason_fold_preparation=os.environ.get('GLUT_CARRY_REASON_FOLD_WORD'), parity_fold_preparation=os.environ.get('GLUT_PARITY_FOLD_WORD'), square_lane_isolated=os.environ.get('GLUT_ISOLATE_SQUARE_WORD')=='⊥', binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(), source_hashes=hashes)
    (folder/'measurement.json').write_text(json.dumps(row,indent=2)+'\n')
    rows.append(row); (OUT/'manifest.json').write_text(json.dumps(rows,indent=2)+'\n')
    print(case, 'closed' if not interrupted else 'deadline observation', elapsed, flush=True)
if any(row['interrupted'] for row in rows):
    raise SystemExit('FAILED: random semiprimes missed the execution deadline; all width measurements retained.')
