#!/usr/bin/env python3
"""IMASM tower driver — IMASM running on IMASM operating on IMASM.

Level 1 (operator): the six IMASM operator words in morphism_factor.rs
    PHASE / ARITHMETIC / BRANCH / SELECT / CONTINUE / FIX
    dispatch is read FROM the operator word itself; each operator is both
    the boundary and the action at that boundary.
Level 2 (operand):  the native-numeral IMASM word for N
    ⊢ (≻⋈∈[⊤=0|⊥=1]∋)^k ⊙⊡⊣   (LSB first)
Level 3 (result):   the factor emitted as its own numeral word, decoded here.

Run:  python3 tower_factor.py <decimal N>
"""
import subprocess, sys
sys.set_int_max_str_digits(0)

VOX = "./vox"

def encode(n):
    if n == 0:
        return "⊢⊙⊡⊣"
    cells = []
    while n:
        cells.append("≻⋈∈" + ("⊥" if (n & 1) else "⊤") + "∋")
        n >>= 1
    return "⊢" + "".join(cells) + "⊙⊡⊣"

def decode(word):
    assert word[0] == "⊢" and word[-3:] == "⊙⊡⊣", f"bad word {word}"
    body = word[1:-3]
    bits = [1 if body[i+3] == "⊥" else 0 for i in range(0, len(body), 5)]
    v = 0
    for b in reversed(bits):
        v = (v << 1) | b
    return v

def factor(n, timeout=30):
    w = encode(n)
    r = subprocess.run([VOX, "morphism-factor", w], capture_output=True, text=True, timeout=timeout)
    out = r.stdout.strip()
    return (decode(out) if out else None)

if __name__ == "__main__":
    n = int(sys.argv[1])
    f = factor(n)
    if f and n % f == 0 and 1 < f < n:
        print(f"{n} = {f} x {n//f}   (IMASM tower, verified)")
    else:
        print(f"{n}: no factor within tower budget (rho ceiling ~2^(bits/4))")
