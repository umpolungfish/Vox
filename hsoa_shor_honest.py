#!/usr/bin/env python3
"""
Honest order recovery: build the Shor index state from (a, n) alone, derive r
from the winding/QFT readout, and take the FACTOR from that readout.

The correction the principal raised: in hsoa_shor_state.py, shor_comb() calls
true_period(a, n) up front and run() factors with the known r_true.  Here the
state is built from the modular orbit only, r comes from the readout, and
true_period appears only afterwards, as an independent check.
"""
from __future__ import annotations
import math, time
import numpy as np
from hsoa_shor_state import mod_pow, true_period, winding_number, factor_close

def shor_index_state(a, n, m):
    """Collapsed index register of |x>|a^x mod n>, from (a,n,m) alone.
    Keep x in [0,m) with a^x = 1 (the x=0 output outcome). r is never
    computed here -- but see `first_gap` for the honest caveat."""
    idx = [x for x in range(m) if mod_pow(a, x, n) == 1]
    if not idx:
        raise ValueError("empty index register")
    amp = 1.0 / math.sqrt(len(idx))
    st = np.zeros(m, dtype=complex)
    for x in idx:
        st[x] = amp
    return st, idx

def pipeline(a, n, n_qubits):
    m = 1 << n_qubits
    st, idx = shor_index_state(a, n, m)
    W = winding_number(st, a, n)            # QFT peak -> continued fractions
    first_gap = (idx[1] - idx[0]) if len(idx) > 1 else 0
    fac = factor_close(a, n, W) if W else None
    return W, first_gap, fac

def main():
    cases = [(7,15,6),(2,21,6),(2,35,8),(2,91,8),(2,143,12)]
    print("HONEST order recovery: (a,N) -> state -> QFT readout W -> factor")
    print("-"*82)
    all_ok = True
    for a, n, q in cases:
        t0 = time.time()
        W, first_gap, fac = pipeline(a, n, q)
        dt = time.time() - t0
        r_true = true_period(a, n)          # AFTER the fact, check only
        agree = (W == r_true)
        facok = bool(fac and fac[0]*fac[1] == n)
        all_ok = all_ok and agree and facok
        print(f"a={a:>3} N={n:>5} M=2^{q:<2}  W={W:>3}  r_true={r_true:>3}  "
              f"{'AGREE' if agree else 'MISMATCH':>8}  factors={str(fac):>12}  "
              f"{'OK' if facok else '':2}  ({dt*1000:.1f} ms)")
        print(f"         honest caveat: first gap in the enumerated orbit "
              f"(classical shortcut to r) = {first_gap}")
    print("-"*82)
    print("all readouts agree with true_period AND factor:", all_ok)

if __name__ == "__main__":
    main()
