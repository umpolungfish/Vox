"""Reproducible N-only producer probes; prime oracles remain in this process."""
import argparse
import json
import random
import subprocess
import time
import hashlib
from pathlib import Path
from sympy import nextprime, isprime

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--bits', nargs='+', type=int, default=[175, 192, 224, 256, 320, 384, 512])
parser.add_argument('--samples', type=int, default=5)
parser.add_argument('--seconds', type=float, default=60)
parser.add_argument('--families', nargs='+', choices=['close', 'multiplier', 'balanced', 'unbalanced'], default=['close', 'multiplier', 'balanced', 'unbalanced'])
parser.add_argument('--output', required=True)
args = parser.parse_args()
assert min(args.bits) >= 175
revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
binary_hash = hashlib.sha256((ROOT/'target/release/semiprime_probe').read_bytes()).hexdigest()
with open(args.output, 'x') as output:
    for bits in args.bits:
        for family in args.families:
            for sample in range(args.samples):
                seed = f'vox-semiprime-v1:{bits}:{family}:{sample}'
                rng = random.Random(seed)
                while True:
                    pb = bits // 3 if family == 'unbalanced' else bits // 2
                    p = int(nextprime(rng.randrange(1 << (pb-1), 1 << pb)))
                    if family == 'close':
                        p = int(nextprime(rng.randrange(1 << ((bits+1)//2-1), 1 << ((bits+1)//2))))
                        q = int(nextprime(p + 1000*(sample+1)))
                    elif family == 'multiplier':
                        q = int(nextprime(2*p))
                    else:
                        qb = bits-pb
                        q = int(nextprime(rng.randrange(1 << (qb-1), 1 << qb)))
                    n = p*q
                    if n.bit_length() == bits and p != q:
                        break
                assert isprime(p) and isprime(q)
                for producer in ['dialectic', 'resident']:
                    row = dict(bits=bits, family=family, sample=sample, seed=seed,
                               n=str(n), p=str(p), q=str(q), producer=producer,
                               revision=revision, binary_sha256=binary_hash,
                               primality='sympy probable-prime checks', budget=args.seconds)
                    started = time.monotonic()
                    try:
                        result = subprocess.run([str(ROOT/'target/release/semiprime_probe'), producer, str(n)],
                                                capture_output=True, text=True, timeout=args.seconds)
                        row.update(stdout=result.stdout, stderr=result.stderr, returncode=result.returncode)
                        factors = [int(line.split()[1]) for line in result.stdout.splitlines() if line.startswith('factor ')]
                        row['status'] = 'success' if result.returncode == 0 and p in factors and q in factors else 'failure'
                    except subprocess.TimeoutExpired:
                        row['status'] = 'timeout'
                    row['elapsed'] = time.monotonic()-started
                    output.write(json.dumps(row)+'\n')
                    output.flush()
                    print(bits, family, sample, producer, row['status'], round(row['elapsed'], 3), flush=True)
