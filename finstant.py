#!/usr/bin/env python3
"""
finstant.py вАФ instant factor extraction.

Uses the AREV trajectory: walk `a` from ceil(sqrt(N)) upward, compute
b = floor(sqrt(a^2 - N)), and stop when a^2 - N is a perfect square.
That is Fermat's method, and it is the ≺ AREV step of the membrane.

For N with close factors, this is O(1).
For N with far factors, fall back to a bounded sweep.
"""

from __future__ import annotations
import sys
import time
from math import isqrt


def factor(N: int, max_fermat: int = 10_000_000):
    """
    Factor N. Returns (p, q) with p <= q, or None.
    """
    if N <= 0:
        raise ValueError("N must be positive")
    if N == 1:
        return None
    if N % 2 == 0:
        twos = (N & -N).bit_length() - 1
        rest = N >> twos
        if rest == 1:
            return (2 ** twos,)
        sub = factor(rest, max_fermat)
        if sub is None:
            return None
        return (2 ** twos,) + sub

    # Fermat walk: a from ceil(sqrt(N)) upward
    a = isqrt(N)
    if a * a < N:
        a += 1

    for step in range(max_fermat):
        b2 = a * a - N
        b = isqrt(b2)
        if b * b == b2:
            p = a - b
            q = a + b
            if p > 1 and p * q == N:
                return (p, q)
        a += 1

    return None


def factor_with_sweep(N: int, max_fermat: int = 100_000_000_000,
                     max_trial: int = 1_000_000_000_000):
    """
    Fermat walk, then trial division by small primes if Fermat fails.
    """
    res = factor(N, max_fermat)
    if res:
        return res

    # Trial division by 2, 3, 5, ... up to max_trial
    if N % 2 == 0:
        return factor(N)
    for d in range(3, min(max_trial, isqrt(N)) + 1, 2):
        if N % d == 0:
            return (d, N // d)
    return None


def main():
    if len(sys.argv) < 2:
        print("usage: finstant.py <N> [--fermat MAX] [--trial MAX]")
        sys.exit(1)

    N = int(sys.argv[1], 0)
    max_fermat = 100_000_000_000
    max_trial = 1_000_000_000_000
    i = 2
    while i < len(sys.argv):
        if sys.argv[i] == '--fermat':
            max_fermat = int(sys.argv[i+1]); i += 2
        elif sys.argv[i] == '--trial':
            max_trial = int(sys.argv[i+1]); i += 2
        else:
            i += 1

    t0 = time.perf_counter()
    vals = factor_with_sweep(N, max_fermat, max_trial)
    dt = time.perf_counter() - t0

    print(f"N         = {N}")
    print(f"bit length= {N.bit_length()}")
    print(f"time      = {dt*1000:.3f} ms")
    if vals:
        prod = 1
        for v in vals:
            prod *= v
        print(f"factors   = {' × '.join(str(v) for v in vals)}")
        print(f"check     = {prod == N}")
    else:
        print("factors   = (none found within budget)")


if __name__ == '__main__':
    main()