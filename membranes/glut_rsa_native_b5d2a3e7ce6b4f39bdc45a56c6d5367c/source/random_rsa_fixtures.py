"""Reproducible independently sampled prime pairs for parent-side checks."""
import hashlib
import math
import random
import uuid
import sympy


def fixture(bits, index, fixture_id=None):
    fixture_id = uuid.uuid4().hex if fixture_id is None else uuid.UUID(fixture_id).hex
    seed = hashlib.sha256(f'glut-rsa-independent-v2:{bits}:{fixture_id}'.encode()).hexdigest()
    rng = random.Random(int(seed, 16))
    widths = (bits // 2, bits - bits // 2)
    while True:
        primes = []
        for width in widths:
            candidate = rng.getrandbits(width) | (1 << (width - 1)) | 1
            prime = int(sympy.nextprime(candidate))
            if prime.bit_length() != width or math.gcd(prime - 1, 65537) != 1:
                break
            primes.append(prime)
        if len(primes) != 2:
            continue
        p, q = sorted(primes)
        if p == q or (p * q).bit_length() != bits:
            continue
        if q - p < (1 << (min(widths) // 2)):
            continue
        assert sympy.isprime(p) and sympy.isprime(q)
        return dict(name=f'rsa_{bits}_{index}_{fixture_id}', fixture_id=fixture_id,
                    generator='glut-rsa-independent-v2', seed=seed, p=str(p), q=str(q),
                    n=str(p * q), bits=bits, factor_bits=min(widths),
                    factor_bit_widths=[p.bit_length(), q.bit_length()],
                    factor_gap_bits=(q - p).bit_length(), public_exponent=65537,
                    primality='deterministic' if max(widths) <= 64 else 'probable prime',
                    primality_instrument='SymPy ' + sympy.__version__)
