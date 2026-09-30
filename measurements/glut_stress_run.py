"""Bake and measure increasing-width glut cases; limits apply to child processes only."""
import fcntl
import json
import os
from pathlib import Path
import resource
import shutil
import signal
import subprocess
import time

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / ('membranes/glut_stress_' + os.environ.get('GLUT_STRESS_RUN', '2026-09-29'))
OUT.mkdir(exist_ok=True)
CASES = [
    ('control_35', 5*7, 'balanced', 5, 7),
    ('control_prime_127', 127, 'prime control', None, None),
    ('control_8051', 83*97, 'balanced', 83, 97),
    ('balanced_14', 101*103, 'balanced', 101, 103),
    ('square_17', 257**2, 'square', 257, 257),
    ('balanced_20', 1009*1013, 'balanced', 1009, 1013),
    ('balanced_27', 10007*10009, 'balanced', 10007, 10009),
    ('dense_32', 65535**2, 'dense square', 65535, 65535),
    ('balanced_40', 1000003*1000033, 'balanced', 1000003, 1000033),
    ('balanced_64', 4294967291*4294967279, 'balanced', 4294967291, 4294967279),
    ('sparse_17', 1<<16, 'power of two', 256, 256),
    ('sparse_33', 1<<32, 'power of two', 65536, 65536),
    ('sparse_65', 1<<64, 'power of two', 1<<32, 1<<32),
    ('sparse_129', 1<<128, 'power of two', 1<<64, 1<<64),
    ('sparse_257', 1<<256, 'power of two', 1<<128, 1<<128),
    ('unbalanced_18', 3*((1<<16)+1), 'unbalanced', 3, (1<<16)+1),
    ('unbalanced_34', 3*((1<<32)+1), 'unbalanced', 3, (1<<32)+1),
    ('unbalanced_66', 3*((1<<64)+1), 'unbalanced', 3, (1<<64)+1),
    ('unbalanced_130', 3*((1<<128)+1), 'unbalanced', 3, (1<<128)+1),
    ('unbalanced_258', 3*((1<<256)+1), 'unbalanced', 3, (1<<256)+1),
    ('unbalanced_5_259', 5*((1<<256)+1), 'unbalanced', 5, (1<<256)+1),
    ('unbalanced_17_261', 17*((1<<256)+1), 'unbalanced', 17, (1<<256)+1),
]
SECONDS = min(60.0, float(os.environ.get('GLUT_STRESS_SECONDS', '8')))
ADDRESS_BYTES = 768*1024*1024

def limits():
    resource.setrlimit(resource.RLIMIT_AS, (ADDRESS_BYTES, ADDRESS_BYTES))
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))

def measure(binary, folder):
    start = time.monotonic()
    rss_peak = 0
    with (folder/'native.stdout').open('w') as stdout, (folder/'native.stderr').open('w') as stderr:
        proc = subprocess.Popen([str(binary)], stdout=stdout, stderr=stderr,
            preexec_fn=limits, start_new_session=True)
        interrupted = False
        while proc.poll() is None:
            try:
                status = Path(f'/proc/{proc.pid}/status').read_text()
                for line in status.splitlines():
                    if line.startswith('VmHWM:'):
                        rss_peak = max(rss_peak, int(line.split()[1]))
            except FileNotFoundError:
                pass
            if time.monotonic() - start > SECONDS:
                os.killpg(proc.pid, signal.SIGTERM)
                try:
                    proc.wait(timeout=1)
                except subprocess.TimeoutExpired:
                    os.killpg(proc.pid, signal.SIGKILL)
                    proc.wait()
                interrupted = True
                break
            time.sleep(0.02)
    lines = (folder/'native.stdout').read_text().splitlines()
    frames = [line.split('\t')[1:] for line in lines if line.startswith('FRAME\t')]
    result = next((line.split('\t')[1:] for line in lines if line.startswith('RESULT\t')), None)
    return dict(returncode=proc.returncode, interrupted=interrupted,
        wall_seconds=time.monotonic()-start, sampled_rss_peak_kib=rss_peak,
        last_frame=frames[-1] if frames else None,
        last_fold=next((line.split('\t')[1:] for line in reversed(lines) if line.startswith('FOLD\t')), None),
        peak_completed_frame_states=max((int(f[2]) for f in frames), default=0), result=result)

selected = os.environ.get('GLUT_STRESS_CASES')
if selected:
    CASES = [c for c in CASES if c[0] in selected.split(',')]
manifest = []
for name, n, shape, p, q in CASES:
    folder = OUT/name
    folder.mkdir(exist_ok=True)
    word = subprocess.check_output([str(ROOT/'target/release/vox'), 'numeral', str(n)], text=True).strip()
    (folder/'input.imasm').write_text(word+'\n')
    env = dict(os.environ, GLUT_STRESS_WORD=word,
        RUSTFLAGS='-D warnings -C target-feature=+crt-static -C relocation-model=static')
    binary = folder/'glut_stress_baked'
    with (ROOT/'measurements/glut_bake.lock').open('a') as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        with (folder/'build.stdout').open('w') as log:
            subprocess.run(['cargo', 'build', '--release', '--target', 'x86_64-unknown-linux-musl',
                '--example', 'glut_stress_baked'], cwd=ROOT, env=env, stdout=log, stderr=log, check=True,timeout=60)
        shutil.copy2(ROOT/'target/x86_64-unknown-linux-musl/release/examples/glut_stress_baked', binary)
    measured = measure(binary, folder)
    row = dict(name=name, n=str(n), bits=n.bit_length(), popcount=n.bit_count(),
        word_chars=len(word), shape=shape, supplied_p=str(p) if p else None,
        supplied_q=str(q) if q else None, process_seconds=SECONDS,
        process_address_bytes=ADDRESS_BYTES, **measured)
    if measured['result'] and measured['result'][0] == 'closed':
        rp, rq = map(int, measured['result'][1:3])
        assert rp > 1 and rq > 1 and rp*rq == n
    if name == 'control_prime_127':
        assert measured['result'] and measured['result'][0] == 'exhausted_exact_set'
    manifest.append(row)
    (folder/'measurement.json').write_text(json.dumps(row, indent=2)+'\n')
    (OUT/'manifest.json').write_text(json.dumps(manifest, indent=2)+'\n')
    print(name, row['bits'], measured['result'] or ('time_interrupt' if measured['interrupted'] else 'process_exit_'+str(measured['returncode'])), 'last', measured['last_frame'], flush=True)
