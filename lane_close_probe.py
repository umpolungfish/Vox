#!/usr/bin/env python3
"""lane_close_probe.py – exploit the small N//(L0*L1) and forced residues"""

from math import gcd, isqrt
import sys

def bits_lsb(n):
    if n == 0: return [0]
    b = []
    while n:
        b.append(n & 1)
        n >>= 1
    return b

def from_bits(b):
    v = 0
    for i, x in enumerate(b):
        if x: v |= 1 << i
    return v

def delta(bits):
    a, b = [], []
    for i, x in enumerate(bits):
        (a if i % 2 == 0 else b).append(x)
    while len(a) > 1 and a[-1] == 0: a.pop()
    while len(b) > 1 and b[-1] == 0: b.pop()
    return a, b

def perfect_lanes(n, depth=2):
    cur = bits_lsb(n)
    lanes = []
    for _ in range(depth):
        a, b = delta(cur)
        lanes.append((from_bits(a), from_bits(b)))
        cur = a
    return lanes

def try_close(n, p_known=None, q_known=None):
    lanes = perfect_lanes(n, depth=2)
    L0, L1 = lanes[0]
    L20, L21 = lanes[1]
    prod = L0 * L1
    print(f"L1 product = {prod}")
    print(f"N // prod  = {n // prod}")
    print(f"N %  prod  = {n % prod}")

    # small-k close
    for k in range(1, 32):
        g = gcd(prod * k - n, n)
        if 1 < g < n:
            print(f"*** factor via k={k}: {g}")
            return g

    # residue close on L2
    for L in (L20, L21, L0, L1):
        r = L % n
        g = gcd(L - r, n)          # trivial, but also
        g = gcd(L, n)
        if 1 < g < n:
            print(f"*** direct lane factor: {g}")
            return g
        # try L ± small
        for d in range(-16, 17):
            g = gcd(L + d, n)
            if 1 < g < n:
                print(f"*** lane±{d} factor: {g}")
                return g

    # classical Fermat on the shorter L2 lanes
    for L in (L20, L21):
        a = isqrt(L) + 1
        for _ in range(100000):
            b2 = a*a - L
            b = isqrt(b2)
            if b*b == b2:
                f = a - b
                if 1 < f < L and n % f == 0:
                    print(f"*** Fermat-on-lane factor of N: {f}")
                    return f
            a += 1
    print("no factor recovered by these closes")
    return None

if __name__ == "__main__":
    # RSA-100
    n = 1522605027922533360535618378132637429718068114961380688657908494580122963258952897654000350692006139
    try_close(n)