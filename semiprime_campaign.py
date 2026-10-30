"""Reproducible N-only producer probes; prime oracles remain in this process."""
import argparse
import json
import random
import subprocess
import time
import hashlib
import sys
from pathlib import Path
from sympy import nextprime, isprime
from baked_case import build_case

# These are locally generated test integers, not untrusted service requests.
# Decimal reporting must not impose a factoring-width ceiling.
if hasattr(sys, 'set_int_max_str_digits'):
    sys.set_int_max_str_digits(0)

ROOT = Path(__file__).resolve().parent
parser = argparse.ArgumentParser()
parser.add_argument('--bits', nargs='+', type=int, default=[175, 192, 224, 256, 320, 384, 512])
parser.add_argument('--samples', type=int, default=5)
parser.add_argument('--seconds', type=float, default=None,
                    help='optional external timeout; omitted means run to completion')
parser.add_argument('--families', nargs='+', choices=['close', 'multiplier', 'balanced', 'unbalanced'], default=['close', 'multiplier', 'balanced', 'unbalanced'])
parser.add_argument('--output', required=True)
parser.add_argument('--keep-going', action='store_true', help='continue after a failed or timed-out case')
parser.add_argument('--producers', nargs='+', choices=['dialectic', 'resident', 'phase', 'braid', 'symbolic'], default=['dialectic', 'resident'])
args = parser.parse_args()
if args.seconds is not None and args.seconds <= 0:
    parser.error('--seconds must be positive when supplied')
assert min(args.bits) >= 175
revision = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip()
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
                for producer in args.producers:
                    binary, words = build_case(n, producer)
                    binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
                    row = dict(bits=bits, family=family, sample=sample, seed=seed,
                               n=str(n), p=str(p), q=str(q), producer=producer,
                               revision=revision, binary_sha256=binary_hash,
                               binary=str(binary), imasm_inputs=words,
                               primality='sympy probable-prime checks', budget=args.seconds)
                    started = time.monotonic()
                    try:
                        result = subprocess.run([str(binary)], env={}, input='',
                                                capture_output=True, text=True, timeout=args.seconds)
                        row.update(stdout=result.stdout, stderr=result.stderr, returncode=result.returncode)
                        factors = [int(line.split()[1]) for line in result.stdout.splitlines() if line.startswith('factor ')]
                        row['status'] = 'success' if result.returncode == 0 and p in factors and q in factors else 'failure'
                    except subprocess.TimeoutExpired as exc:
                        row['stdout'] = (exc.stdout or b'').decode(errors='replace')
                        row['stderr'] = (exc.stderr or b'').decode(errors='replace')
                        row['status'] = 'timeout'
                    except KeyboardInterrupt:
                        row['status'] = 'interrupted'
                        row['elapsed'] = time.monotonic()-started
                        output.write(json.dumps(row)+'\n')
                        output.flush()
                        raise SystemExit(130)
                    row['elapsed'] = time.monotonic()-started
                    output.write(json.dumps(row)+'\n')
                    output.flush()
                    print(bits, family, sample, producer, row['status'], round(row['elapsed'], 3), flush=True)
                    if row['status'] != 'success' and not args.keep_going:
                        raise SystemExit('Battery stopped at failed case; evidence retained in '+args.output)
