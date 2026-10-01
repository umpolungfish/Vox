"""Reproducible independently sampled prime pairs for parent-side checks."""
import hashlib
import json
import math
import os
from pathlib import Path
import random
import subprocess
import uuid

ROOT = Path(__file__).resolve().parents[1]
VOX = ROOT / 'target/release/vox'
_vox_ready = False


def _ensure_vox():
    global _vox_ready
    if _vox_ready:
        return
    env = dict(os.environ, RUSTFLAGS='-D warnings')
    result = subprocess.run(['cargo', 'build', '--release', '--bin', 'vox'], cwd=ROOT,
                            env=env, text=True, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, timeout=60)
    if result.returncode:
        raise RuntimeError('warning-free Vox primality build failed:\n' + result.stdout)
    if any(line.lstrip().startswith('warning') for line in result.stdout.splitlines()):
        raise RuntimeError('Vox primality build emitted a warning:\n' + result.stdout)
    _vox_ready = True


def _is_prime(value):
    _ensure_vox()
    result = subprocess.run([str(VOX), 'prime-check', str(value)], cwd=ROOT,
                            text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            timeout=60, check=True)
    answer = result.stdout.strip()
    if answer not in ('prime', 'composite') or result.stderr:
        raise RuntimeError('unexpected Vox primality response')
    return answer == 'prime'


def _archived_fixture(bits, index, fixture_id):
    pattern = f'membranes/glut_rsa_native_*/rsa_{bits}_{index}_{fixture_id}/fixture.json'
    matches = [json.loads(path.read_text()) for path in ROOT.glob(pattern)]
    if not matches:
        return None
    first = matches[0]
    identity = (first['n'], first['p'], first['q'], first['seed'])
    if any((row['n'], row['p'], row['q'], row['seed']) != identity for row in matches):
        raise RuntimeError('historical fixture identity resolves to conflicting input values')
    p, q, n = int(first['p']), int(first['q']), int(first['n'])
    if (first.get('fixture_id') != fixture_id or first.get('bits') != bits or p * q != n
            or n.bit_length() != bits or p == q):
        raise RuntimeError('historical fixture failed its arithmetic and identity checks')
    if not _is_prime(p) or not _is_prime(q):
        raise RuntimeError('historical fixture factors failed Vox primality checks')
    verified = dict(first)
    verified['factor_validation_instrument'] = 'Vox morphism_factor::miller_rabin'
    return verified


def _random_prime(rng, width):
    while True:
        candidate = rng.getrandbits(width) | (1 << (width - 1)) | 1
        while candidate.bit_length() == width:
            if math.gcd(candidate - 1, 65537) == 1 and _is_prime(candidate):
                return candidate
            candidate += 2


def fixture(bits, index, fixture_id=None):
    fixture_id = uuid.uuid4().hex if fixture_id is None else uuid.UUID(fixture_id).hex
    archived = _archived_fixture(bits, index, fixture_id)
    if archived is not None:
        return archived
    seed = hashlib.sha256(f'glut-rsa-independent-v2:{bits}:{fixture_id}'.encode()).hexdigest()
    rng = random.Random(int(seed, 16))
    widths = (bits // 2, bits - bits // 2)
    while True:
        p = _random_prime(rng, widths[0])
        q = _random_prime(rng, widths[1])
        if p == q or (p * q).bit_length() != bits:
            continue
        if q - p < (1 << (min(widths) // 2)):
            continue
        p, q = sorted((p, q))
        return dict(name=f'rsa_{bits}_{index}_{fixture_id}', fixture_id=fixture_id,
                    generator='glut-rsa-independent-v2', seed=seed, p=str(p), q=str(q),
                    n=str(p * q), bits=bits, factor_bits=min(widths),
                    factor_bit_widths=[p.bit_length(), q.bit_length()],
                    factor_gap_bits=(q - p).bit_length(), public_exponent=65537,
                    primality='probable prime',
                    primality_instrument='Vox morphism_factor::miller_rabin')
