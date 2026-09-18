#!/usr/bin/env python3
"""Check whether perfect_one's decomposition on the OPEN values reveals a factor.
On RSA-100..260 we have p, q; on 270+ we don't. So the checks here are all
factor-revealing on their own — any non-trivial gcd with N, any exact square
relation, any clean k in N = k*A*B is a candidate."""
import re, subprocess, sys
from math import gcd, isqrt

VOX = "/home/mrnob0dy666/imsgct/Vox"
TIMEOUT = 900  # generous; depth-2 runs at ~3 s, so this is pure safety

def run(n):
    return subprocess.check_output(["./perfect_one.sh", str(n), "2"],
                                   cwd=VOX, text=True, timeout=TIMEOUT)

def parse(out):
    g = lambda pat: int(re.search(pat, out)[1])
    return (g(r"delta L1: lane0=(\d+)"),
            g(r"delta L1: lane0=\d+ lane1=(\d+)"),
            g(r"delta L2: lane0=(\d+)"),
            g(r"delta L2: lane0=\d+ lane1=(\d+)"),
            g(r"core transform: lane product = (\d+)"),
            g(r"mu\s+L2: recombined = (\d+)"),
            g(r"mu\s+L1: recombined = (\d+)"))

def is_square(n):
    if n < 0: return None
    r = isqrt(n)
    return r if r*r == n else None

def probe(N):
    L1_0, L1_1, L2_0, L2_1, core, mu2, mu1 = parse(run(N))
    hits = []

    def g(label, val):
        if 1 < val < N: hits.append((label, val))

    # structural identity check (should be True)
    assert mu2 == L1_0 and mu1 == N and core == L2_0 * L2_1, "membrane identity broke"

    # direct divisor candidates
    for name, x in [("L1_0", L1_0), ("L1_1", L1_1), ("L2_0", L2_0), ("L2_1", L2_1), ("core", core)]:
        g(f"gcd({name},N)", gcd(x, N))
        g(f"gcd({name}+1,N)", gcd(x+1, N))
        g(f"gcd({name}-1,N)", gcd(x-1, N))

    g("gcd(L1_0+L1_1,N)", gcd(L1_0 + L1_1, N))
    g("gcd(L1_1-L1_0,N)", gcd(abs(L1_1 - L1_0), N))
    g("gcd(L1_0*L1_1,N)", gcd(L1_0 * L1_1, N))
    g("gcd(L1_0+L1_1+1,N)", gcd(L1_0 + L1_1 + 1, N))
    g("gcd(L1_0+L1_1-1,N)", gcd(L1_0 + L1_1 - 1, N))

    # Fermat structure: if (A+B)^2 - N or (A-B)^2 - N is a perfect square,
    # then N = (A+B-r)(A+B+r) or similar, giving factors.
    s = isqrt(N)
    cands = [
        ("(L1_0+L1_1)^2-N", (L1_0 + L1_1)**2 - N),
        ("(L1_1-L1_0)^2-N", (L1_1 - L1_0)**2 - N),
        ("N-(L1_1-L1_0)^2", N - (L1_1 - L1_0)**2),
        ("(2*s-L1_0-L1_1)^2", None),  # placeholder
        ("L1_0^2+L1_1^2-N", L1_0**2 + L1_1**2 - N),
        ("L1_1^2-L1_0^2-N", L1_1**2 - L1_0**2 - N),
        ("L1_0^2-L1_1^2+N", L1_0**2 - L1_1**2 + N),
        ("L2_0^2+L2_1^2-L1_0", L2_0**2 + L2_1**2 - L1_0),
        ("(L2_0+L2_1)^2-L1_0", (L2_0 + L2_1)**2 - L1_0),
        ("(L2_0+L2_1)^2-N", (L2_0 + L2_1)**2 - N),
    ]
    for name, v in cands:
        if v is None: continue
        r = is_square(v)
        if r is not None:
            hits.append((f"{name} is square (root)", r))

    # clean division: N = k * L1_0 * L1_1 + eps? if eps == 0 that's structural
    prod = L1_0 * L1_1
    if N % prod == 0:
        hits.append(("N//(L1_0*L1_1) exact", N // prod))
    if N % core == 0:
        hits.append(("N//core exact", N // core))

    # lane times small integer matches a factor? check gcd(lane*2..17,N)
    for x, name in [(L1_0, "L1_0"), (L1_1, "L1_1"), (L2_0, "L2_0"), (L2_1, "L2_1")]:
        for m in range(2, 18):
            if m*x % N != 0:
                continue
        for m in range(2, 18):
            gg = gcd(m*x, N)
            if 1 < gg < N:
                hits.append((f"gcd({m}*{name},N)", gg))

    return (L1_0, L1_1, L2_0, L2_1, core), hits

def main():
    lines = [l.strip() for l in open(f"{VOX}/testvals.txt") if l.strip().isdigit()]
    # skip RSA-260 (already factored), run 270+
    for idx, N_str in enumerate(lines):
        N = int(N_str)
        label = f"val{idx+1}"
        print(f"\n=== {label}  {len(N_str)} digits  {N_str[:32]}… ===", flush=True)
        try:
            lanes, hits = probe(N)
        except subprocess.TimeoutExpired:
            print("  TIMEOUT"); continue
        except Exception as e:
            print("  ERR:", e); continue
        L1_0, L1_1, L2_0, L2_1, core = lanes
        print(f"  L1 ({len(str(L1_0))},{len(str(L1_1))})  L2 ({len(str(L2_0))},{len(str(L2_1))})  core {len(str(core))}")
        if not hits:
            print("  no factor-revealing structure found")
        else:
            for lab, v in hits:
                print(f"  HIT  {lab} = {v}")

if __name__ == "__main__":
    main()