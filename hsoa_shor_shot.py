#!/usr/bin/env python3
"""Shor shot: (a,N) -> joint state |x>|a^x mod N> -> QFT -> continued fractions
-> r -> factors.  Built from a,N alone; no true_period anywhere in the pipeline."""
import math, time
import numpy as np
from hsoa_shor_state import mod_pow, true_period, factor_close, _period_from_peak

def shor_shot(a, n, n_qubits):
    m = 1 << n_qubits
    psi = np.zeros((m, n), dtype=complex)          # joint |x>|a^x mod n>
    for x in range(m):
        psi[x, mod_pow(a, x, n)] = 1.0 / math.sqrt(m)
    qft = np.fft.fft(psi, axis=0) / math.sqrt(m)   # QFT on the index register
    probs = np.sum(np.abs(qft) ** 2, axis=1)       # index-register marginal
    for k in np.argsort(-probs):
        if k == 0:
            continue
        r = _period_from_peak(int(k), m, a, n)     # certify a^r = 1 (mod n)
        if r:
            return r
    return 0

def shot(a, n, q):
    t0 = time.time()
    r = shor_shot(a, n, q)
    fac = factor_close(a, n, r) if r else None
    return r, fac, time.time() - t0

def main():
    cases = [(7,15,8),(2,21,8),(2,35,8),(2,91,8),(2,143,12)]
    print("(a,N) -> |x>|a^x mod N> -> QFT -> CF -> r -> factors   [no true_period]")
    print("-"*76)
    allok = True
    for a, n, q in cases:
        r, fac, dt = shot(a, n, q)
        rt = true_period(a, n)
        ok = (r == rt) and bool(fac) and fac[0]*fac[1] == n
        allok = allok and ok
        print(f"a={a:>3} N={n:>5} 2^{q:<2}  r={r:>3} (true {rt:>3})  factors={str(fac):>12}  "
              f"{'OK' if ok else 'FAIL'}  ({dt*1000:.1f} ms)")
    print("-"*76)
    print("pipeline works:", allok)

if __name__ == "__main__":
    main()
