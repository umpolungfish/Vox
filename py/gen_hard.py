#!/usr/bin/env python3
# Generate hard-shape semiprimes: large factors, wide gap, neither p-1 nor p+1 smooth.
# Threshold on the largest prime factor of p-1 / p+1 scales with the prime size.
import random
random.seed(7)

def is_prime(n):
    if n < 2: return False
    for p in (2,3,5,7,11,13,17,19,23,29,31,37):
        if n % p == 0: return n == p
    d = n - 1; r = 0
    while d % 2 == 0: d //= 2; r += 1
    for a in (2,3,5,7,11,13,17,19,23,29,31,37):
        x = pow(a, d, n)
        if x in (1, n - 1): continue
        for _ in range(r - 1):
            x = x * x % n
            if x == n - 1: break
        else: return False
    return True

def largest_factor(n):
    m = n; big = 1; d = 2
    while d * d <= m and d < 10**6:
        while m % d == 0: big = max(big, d); m //= d
        d += 1
    return max(big, m) if m > 1 else big

def hard_prime(bits, smooth=True):
    # ask for a large-ish prime factor of p +/- 1, but never more than the size allows
    thresh = min(10**5, 1 << max(1, bits - 6))
    while True:
        p = random.getrandbits(bits) | (1 << (bits - 1)) | 1
        if not is_prime(p): continue
        if smooth and (largest_factor(p - 1) < thresh or largest_factor(p + 1) < thresh):
            continue
        return p

cases = []
for b in (24, 28, 32, 36):
    p = hard_prime(b); q = hard_prime(b)
    while q == p: q = hard_prime(b)
    cases.append((p * q, p, q, 2 * b))
# wide-gap unbalanced: small prime (no smoothness ask) x 40-bit hard prime
p = hard_prime(12, smooth=False); q = hard_prime(40)
cases.append((p * q, p, q, 52))

for N, p, q, bits in cases:
    print(f"{N} {p} {q} ~{bits}bit")

# bigger balanced semiprimes to stress rho vs QS (called with arg 'big')
import sys
if len(sys.argv) > 1 and sys.argv[1] == 'big':
    for b in (40, 48, 56):
        p = hard_prime(b); q = hard_prime(b)
        while q == p: q = hard_prime(b)
        print(f"{p*q} {p} {q} ~{2*b}bit")
