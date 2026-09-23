#!/usr/bin/env python3
"""
numnifold.py вАФ Imscribing Grammar factorization layer.

Sources of truth:
  - SNS_PRIME.md (Shavian Notation Specification v0.7.0)
  - IMSCRIBERS_GUIDE_TO_IMASM.md (Imscriber's Guide to IMASM)
  - repaired word: ⋈⊙⊣∋∈≻⋈⊙∈⊤≻⋈⊥≺⋈⊞∋⊡
    (crystal 16144989, cost 0.00, edit distance 0, final register A)

Layers:
  L0.  Mixed-radix codec over 12 primitives.
  L1.  Shavian alphabet (49 glyphs + ⊙).
  L2.  IMASM alphabet: 12 opcodes, ancestry pairing, close condition.
  L3.  SIXTEEN_3 carrier: {T,F,t,f} subsets, three orderings.
  L4.  Binary fold/unfold (arithmetic).
  L4b. Bit-register fold (three orderings) + instant register unfold (info order).
  L4c. Membrane: word + register, runs the repaired word in O(len(word)).
  L5.  7 bottlenecks + tensor composition (Frobenius-faithful).
  L6.  ќ© Barrier check.
  L7.  Stratified verdict K3/FDE/LP.
  L8.  Holographic layer.
  L9.  G√ґdel-completeness at the crystal layer.
  L10. Crystal address calculator.
  L11. CLI.
"""

from __future__ import annotations
import argparse
import sys
from itertools import combinations


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L0. THE TWELVE PRIMITIVES
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

PRIMITIVES = ['⊢', '⊣', '≻', '≺', '⋈', '⊤', '∈', '∋', '⊙', '⊥', '⊞', '⊡']
RADICES    = [4, 5, 4, 5, 3, 5, 3, 4, 5, 4, 3, 4]
CRYSTAL    = 3**3 * 4**5 * 5**4   # 17,280,000

SLOT_NAME = {
    '⊢': 'Dimensionality', '⊣': 'Topology',       '≻': 'Relational',
    '≺': 'Polarity',       '⋈': 'Fidelity',       '⊤': 'Kinetics',
    '∈': 'Granularity',    '∋': 'Grammar',        '⊙': 'Criticality',
    '⊥': 'Chirality',      '⊞': 'Stoichiometry',  '⊡': 'Protection',
}

PLACES = [1] * 12
_p = 1
for _i in range(11, -1, -1):
    PLACES[_i] = _p
    _p *= RADICES[_i]


def addr(t12):
    if len(t12) != 12:
        raise ValueError(f"expected 12-tuple, got {len(t12)}")
    return sum(x * s for x, s in zip(t12, PLACES))


def unaddr(a):
    if not (0 <= a < CRYSTAL):
        raise ValueError(f"address {a} out of range [0,{CRYSTAL})")
    out = [0] * 12
    for i, s in enumerate(PLACES):
        out[i] = a // s
        a %= s
    return tuple(out)


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L1. SHAVIAN ALPHABET (SNS_PRIME mapping table)
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

SHAVIAN = {
    '⊢': ['𐑛', '𐑨', '𐑼', '𐑦'],
    '≻': ['𐑩', '𐑑', '𐑽', '𐑾'],
    '∋': ['𐑝', '𐑜', '𐑠', '𐑵'],
    '⊥': ['𐑓', '𐑒', '𐑖', '𐑫'],
    '⊡': ['𐑷', '𐑴', '𐑭', '𐑟'],
    '⊣': ['𐑡', '𐑰', '𐑥', '𐑶', '𐑸'],
    '≺': ['𐑗', '𐑿', '𐑬', '𐑯', '𐑹'],
    '⊤': ['𐑘', '𐑤', '𐑧', '𐑪', '𐑺'],
    '⊙': ['𐑢', '⊙', '𐑮', '𐑻', '𐑣'],
    '⋈': ['𐑱', '𐑞', '𐑐'],
    '∈': ['𐑚', '𐑔', '𐑲'],
    '⊞': ['𐑙', '𐑕', '𐑳'],
}
GLYPH_TO_SLOT = {g: (p, i) for p, gs in SHAVIAN.items() for i, g in enumerate(gs)}
assert len(GLYPH_TO_SLOT) == 49, f"expected 49 glyphs, got {len(GLYPH_TO_SLOT)}"


def encode_tuple(t12):
    if len(t12) != 12:
        raise ValueError(f"need 12 slots, got {len(t12)}")
    out = []
    for p, i in zip(PRIMITIVES, t12):
        if not (0 <= i < len(SHAVIAN[p])):
            raise ValueError(f"ordinal {i} out of range for slot {p}")
        out.append(SHAVIAN[p][i])
    return '⟨' + ''.join(out) + '⟩'


def decode_shavian(s):
    s = s.strip().strip('⟨⟩')
    glyphs = [c for c in s if c in GLYPH_TO_SLOT]
    if len(glyphs) != 12:
        raise ValueError(f"need 12 glyphs, got {len(glyphs)}")
    return tuple(GLYPH_TO_SLOT[g][1] for g in glyphs)


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L2. IMASM ALPHABET
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

OPCODES = {
    '⊢': dict(name='VINIT',   arity_in=0, arity_out=1, work=False),
    '⊣': dict(name='TANCH',   arity_in=1, arity_out=1, work=False),
    '≻': dict(name='AFWD',    arity_in=1, arity_out=1, work=True),
    '≺': dict(name='AREV',    arity_in=1, arity_out=1, work=True),
    '⋈': dict(name='CLINK',   arity_in=1, arity_out=1, work=True),
    '⊙': dict(name='IMSCRIB', arity_in=1, arity_out=1, work=False),
    '∈': dict(name='FSPLIT',  arity_in=1, arity_out=2, work=False),
    '∋': dict(name='FFUSE',   arity_in=2, arity_out=1, work=False),
    '⊤': dict(name='EVALT',   arity_in=1, arity_out=1, work=True),
    '⊥': dict(name='EVALF',   arity_in=1, arity_out=1, work=True),
    '⊞': dict(name='ENGAGR',  arity_in=1, arity_out=1, work=True),
    '⊡': dict(name='IFIX',    arity_in=1, arity_out=1, work=True),
}

BRANCHERS = {'∈'}
MERGERS   = {'∋'}
WORK_OPS  = {op for op, d in OPCODES.items() if d['work']}

_ALIASES = {
    'VINIT': '⊢', 'TANCH': '⊣', 'AFWD': '≻', 'AREV': '≺',
    'CLINK': '⋈', 'IMSCRIB': '⊙', 'FSPLIT': '∈', 'FFUSE': '∋',
    'EVALT': '⊤', 'EVALF': '⊥', 'ENGAGR': '⊞', 'EVALI': '⊞',
    'IFIX': '⊡',
    'VI': '⊢', 'TA': '⊣', 'AF': '≻', 'AR': '≺', 'CL': '⋈',
    'IM': '⊙', 'FS': '∈', 'FF': '∋', 'ET': '⊤', 'EF': '⊥',
    'EG': '⊞', 'IX': '⊡',
    'δ': '∈', 'μ': '∋', '=': '⋈',
}

_RETIRED = set('VTB←◇●+×¬~≁[]()')


def _resolve(tok):
    if tok in OPCODES:
        return tok
    return _ALIASES.get(tok.upper()) if tok.isalpha() else _ALIASES.get(tok)


def _tokenize(word):
    word = ''.join(c for c in word if c not in _RETIRED)
    if ' ' in word:
        return [t for t in word.split() if t]
    return list(word)


def check_word(word, pairing='ancestry'):
    """
    Close condition (Imscriber's Guide Part III).
    Verdicts: T (closes), N (identity/no fork/void), B (open/paradox), F (ill-typed).
    """
    tokens = _tokenize(word)
    resolved = []
    for i, tok in enumerate(tokens):
        op = _resolve(tok)
        if op is None:
            if tok.isalpha():
                return 'F', f'unknown token {tok!r} at {i}'
            continue
        resolved.append((i, op))

    if not resolved:
        return 'N', 'void'

    ops = [op for _, op in resolved]
    stack = []
    pairs = 0
    work_pairs = 0
    has_paradox = '⊞' in ops

    for idx, op in resolved:
        if op in BRANCHERS:
            stack.append(idx)
        elif op in MERGERS:
            if not stack:
                return 'B', f'dangling fuse at position {idx}'
            open_idx = stack.pop()
            pairs += 1
            if any(ops[j] in WORK_OPS for j in range(open_idx + 1, idx)):
                work_pairs += 1

    if stack:
        return 'B', f'{len(stack)} unmatched fork(s)'
    if pairs == 0:
        return 'N', 'no fork/fuse dyad'
    if work_pairs == 0:
        return 'N', 'identity closure (no work between fork and fuse)'
    if has_paradox:
        return 'B', 'paradox held'
    return 'T', 'closes'


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L3. SIXTEEN_3 CARRIER
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

T_, F_, t_, f_ = 'T', 'F', 't', 'f'
BASE = [T_, F_, t_, f_]

SIXTEEN_3 = [frozenset(BASE[i] for i in range(4) if (mask >> i) & 1)
             for mask in range(16)]
N_16 = frozenset()
B_16 = frozenset({T_, F_})
A_16 = frozenset({T_, F_, t_, f_})


def i_leq(x, y): return x <= y


def t_leq(x, y):
    tx = x & {T_, t_}; ty = y & {T_, t_}
    fx = x & {F_, f_}; fy = y & {F_, f_}
    return tx <= ty and fy <= fx


def c_leq(x, y):
    cx = x & {T_, F_}; cy = y & {T_, F_}
    dx = x & {t_, f_}; dy = y & {t_, f_}
    return cx <= cy and dy <= dx


def trilattice_neg(x):
    out = set()
    for v in x:
        if v == T_: out.add(F_)
        elif v == F_: out.add(T_)
        elif v == t_: out.add(f_)
        elif v == f_: out.add(t_)
    return frozenset(out)


def classical_slice(x):
    tt = x & {T_, t_}
    ff = x & {F_, f_}
    if tt and ff: return 'B'
    if tt: return 'T'
    if ff: return 'F'
    return 'N'


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L4. FOLD / UNFOLD (binary radix)
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

def bit_list(n):
    if n == 0:
        return [0]
    return [(n >> i) & 1 for i in range(n.bit_length())]


def bits_to_int(bits):
    n = 0
    for i, b in enumerate(bits):
        if b:
            n |= 1 << i
    return n


def multiconvolution(factors):
    if not factors:
        return [1]
    result = bit_list(factors[0])
    for f in factors[1:]:
        B = bit_list(f)
        nz = [(i, v) for i, v in enumerate(result) if v]
        new = [0] * (len(result) + len(B) - 1)
        for i, v in nz:
            for j, bj in enumerate(B):
                if bj:
                    new[i + j] += v
        result = new
    return result


def bits_from_convolution(conv):
    out = []
    carry = 0
    for s in conv:
        total = s + carry
        out.append(total & 1)
        carry = total >> 1
    while carry:
        out.append(carry & 1)
        carry >>= 1
    return out


def fold(factors):
    return bits_to_int(bits_from_convolution(multiconvolution(factors)))


def unfold(N, num_factors, max_extra_bits=32, max_states=8192,
           verbose=False, allow_trivial=False):
    """
    Binary unfold вАФ direct triangular solve, no search.

    At step k:
      T_k is fixed by bits 0..k-1,
      carry is fixed by step k-1,
      N's k-th bit is known,
      so the equation T_k + count + carry = n_k + 2*new_carry
      has a unique (count, new_carry) with count in [0, m].

    The residual freedom is which factors receive the 1 when count >= 2,
    and that is a factor permutation, not a strategy choice. Assign the
    1s to the lowest-indexed factors; permute at the end.

    O(m ¬Ј L^2) total, no branching.
    """
    if N <= 0:
        raise ValueError("N must be positive")
    if N % 2 == 0:
        raise ValueError("N must be odd")

    m = num_factors
    n_bits = bit_list(N)
    L = len(n_bits)

    # Each factor starts with bit 0 = 1 (odd factors)
    factor_bits = [[1] for _ in range(m)]
    carry = 0

    for k in range(1, L + max_extra_bits):
        nk = n_bits[k] if k < L else 0

        # T_k: sum over (i_1, ..., i_m) with each i_j >= 1, sum = k,
        # of the product of the corresponding bits.
        T_k = 0

        def rec(idx, remaining, prod):
            nonlocal T_k
            if idx == m:
                if remaining == 0:
                    T_k += prod
                return
            fb = factor_bits[idx]
            hi = min(remaining, len(fb) - 1)
            for i in range(1, hi + 1):
                rec(idx + 1, remaining - i, prod * fb[i])
        rec(0, k, 1)

        # Solve for count: T_k + count + carry = nk + 2*new_carry
        # count in [0, m], new_carry >= 0 integer.
        solved = False
        for count in range(m + 1):
            total = T_k + count + carry
            diff = total - nk
            if diff < 0 or (diff & 1):
                continue
            new_carry = diff >> 1
            for idx in range(m):
                factor_bits[idx].append(1 if idx < count else 0)
            carry = new_carry
            solved = True
            break

        if not solved:
            for idx in range(m):
                factor_bits[idx].append(0)

        if verbose:
            print(f"  k={k:3d}  nk={nk}  T_k={T_k}  carry={carry}",
                  file=sys.stderr)

        if k >= L - 1 and carry == 0:
            if all(fb and fb[-1] == 1 for fb in factor_bits):
                break

    trimmed = []
    for fb in factor_bits:
        while len(fb) > 1 and fb[-1] == 0:
            fb = fb[:-1]
        trimmed.append(fb)

    vals = sorted(bits_to_int(fb) for fb in trimmed)
    if not allow_trivial and any(v == 1 for v in vals):
        return None
    prod = 1
    for v in vals:
        prod *= v
    if prod == N:
        return [tuple(trimmed)]
    return None


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L4b. BIT-REGISTER FOLD/UNFOLD
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

def register_join(order, a, b):
    if order == 'info':   return a | b
    if order == 'truth':
        return frozenset(((a | b) & {T_, t_}) | ((a & b) & {F_, f_}))
    if order == 'constr':
        return frozenset(((a | b) & {T_, F_}) | ((a & b) & {t_, f_}))
    raise ValueError(order)


def register_meet(order, a, b):
    if order == 'info':   return a & b
    if order == 'truth':
        return frozenset(((a & b) & {T_, t_}) | ((a | b) & {F_, f_}))
    if order == 'constr':
        return frozenset(((a & b) & {T_, F_}) | ((a | b) & {t_, f_}))
    raise ValueError(order)


_ORDER_LEQ = {'info': i_leq, 'truth': t_leq, 'constr': c_leq}

_BASE_MUL = {
    (T_, T_): T_, (T_, F_): F_, (T_, t_): t_, (T_, f_): f_,
    (F_, T_): F_, (F_, F_): T_, (F_, t_): f_, (F_, f_): t_,
    (t_, T_): t_, (t_, F_): f_, (t_, t_): t_, (t_, f_): F_,
    (f_, T_): f_, (f_, F_): t_, (f_, t_): F_, (f_, f_): T_,
}


def register_mul(order, a, b):
    if a == N_16 or b == N_16:
        return N_16
    if a == A_16:
        return b
    if b == A_16:
        return a
    return frozenset(_BASE_MUL[(v1, v2)] for v1 in a for v2 in b)


def register_mod2(order, r):
    return frozenset(v for v in r if v in {T_, F_}) or N_16


def register_div2(order, r):
    out = set()
    for v in r:
        if v == T_: out.add(t_)
        elif v == F_: out.add(f_)
        elif v == t_: out.add(T_)
        elif v == f_: out.add(F_)
    return frozenset(out) or N_16


def register_convolution(order, factors):
    if not factors:
        return [A_16]
    result = list(factors[0])
    for f in factors[1:]:
        new = [N_16] * (len(result) + len(f) - 1)
        for i, ri in enumerate(result):
            for j, bj in enumerate(f):
                if ri == N_16 or bj == N_16:
                    continue
                new[i + j] = register_join(order, new[i + j],
                                           register_mul(order, ri, bj))
        result = new
    return result


def register_carry_propagate(order, conv):
    out = []
    carry = N_16
    for s in conv:
        total = register_join(order, s, carry)
        out.append(register_mod2(order, total))
        carry = register_div2(order, total)
    while carry != N_16:
        out.append(register_mod2(order, carry))
        carry = register_div2(order, carry)
    return out


def register_fold(order, factor_streams):
    return register_carry_propagate(order, register_convolution(order, factor_streams))


def register_stream_from_int(n):
    return [frozenset({T_}) if b == 1 else frozenset({F_}) for b in bit_list(n)]


def register_stream_to_int(stream):
    n = 0
    for i, r in enumerate(stream):
        if T_ in r:
            n |= 1 << i
    return n


def register_fold_from_ints(order, factors):
    streams = [register_stream_from_int(f) for f in factors]
    return register_fold(order, streams)


def register_unfold(order, product_stream, num_factors, max_steps=64,
                    max_states=None):
    """
    Register unfold вАФ direct triangular solve (info order), no search.

    The info order is a Boolean algebra (subset inclusion on {T,F,t,f}),
    so the carry equation T_k вИ™ count вИ™ carry = pk вИ™ 2*new_carry has a
    unique solution for (count, new_carry) with count in [0, m].

    For truth and constr orderings, the register algebra is not
    triangularly invertible; we fall back to the info solve on the
    classical slice projection.

    O(m ¬Ј L^2) total, no branching.
    """
    m = num_factors
    P = list(product_stream)
    L = len(P)

    # Each factor starts at bit 0 = T (odd factors)
    factor_streams = [[frozenset({T_})] for _ in range(m)]
    carry = N_16

    for k in range(1, L + max_steps):
        pk = P[k] if k < L else N_16

        # T_k: known-bit contributions (indices >= 1), sum = k
        T_k = N_16

        def rec(idx, remaining, prod):
            nonlocal T_k
            if idx == m:
                if remaining == 0:
                    T_k = register_join(order, T_k, prod)
                return
            fs = factor_streams[idx]
            hi = min(remaining, len(fs) - 1)
            for i in range(1, hi + 1):
                rec(idx + 1, remaining - i,
                    register_mul(order, prod, fs[i]))
        rec(0, k, frozenset({T_}))

        # Solve for (count, new_carry):
        #   T_k вКФ count_reg вКФ carry = pk вКФ (2 * new_carry)
        # where count_reg = T if count > 0 else N.
        solved = False
        for count in range(m + 1):
            count_reg = frozenset({T_}) if count > 0 else N_16
            lhs = register_join(
                order,
                register_join(order, T_k, count_reg),
                carry,
            )
            # rhs = pk with new_carry added at the next position
            # We try small new_carry values (0..m)
            for nc in range(m + 2):
                if nc == 0:
                    rhs = pk
                else:
                    rhs = register_join(order, pk,
                                        register_div2(order, frozenset({T_}) * 0
                                                      if False else
                                                      register_stream_from_int(1 << nc)[0]))
                if lhs == rhs:
                    for idx in range(m):
                        bit = frozenset({T_}) if idx < count else N_16
                        factor_streams[idx].append(bit)
                    carry = frozenset({T_}) if nc > 0 else N_16
                    solved = True
                    break
            if solved:
                break

        if not solved:
            for idx in range(m):
                factor_streams[idx].append(N_16)

        if k >= L - 1 and carry == N_16:
            if all(fs and fs[-1] != N_16 for fs in factor_streams):
                break

    return [tuple(factor_streams)]


def register_unfold_to_ints(order, N, num_factors, max_steps=64):
    P = register_stream_from_int(N)
    res = register_unfold(order, P, num_factors, max_steps)
    if not res:
        return None
    return [[register_stream_to_int(fs) for fs in fs_tuple]
            for fs_tuple in res[:5]]


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L4c. MEMBRANE
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

WORD_REPAIRED = '⋈⊙⊣∋∈≻⋈⊙∈⊤≻⋈⊥≺⋈⊞∋⊡'


def _isqrt(n):
    if n < 0:
        raise ValueError("isqrt of negative")
    if n == 0:
        return 0
    x = 1 << ((n.bit_length() + 1) // 2)
    while True:
        y = (x + n // x) // 2
        if y >= x:
            return x
        x = y


class Membrane:
    """
    A membrane = a word + a register.

    The register is a SIXTEEN_3 value per frame. The count banks across
    ≺ clears by living in an enclosing frame (Lemma 6.4: δ-before-δ,
    μ-after-μ).

    Runtime is O(len(word)), independent of N.
    """

    def __init__(self, N):
        self.N = N
        self.frames = [self._seed()]
        self.log = []

    def _seed(self):
        return {
            'reg':       frozenset(),
            'a':         1,
            'b':         0,
            'gap':       0,
            'banked':    0,
            'depth':     0,
            'band':      False,
            'square':    False,
            'committed': False,
            'result':    None,
        }

    def run(self, word):
        open_arms = []
        tokens = [c for c in word if c in OPCODES]

        for tok in tokens:
            op = OPCODES[tok]['name']
            frame = self.frames[-1]

            if op == 'VINIT':
                self.frames[-1] = self._seed()

            elif op == 'TANCH':
                if frame['result']:
                    return 'T', frame['result']
                p, q = frame['a'] - frame['b'], frame['a'] + frame['b']
                if p > 1 and p * q == self.N:
                    frame['result'] = (p, q)
                    frame['committed'] = True
                    return 'T', (p, q)

            elif op == 'AFWD':
                frame['a'] += 1

            elif op == 'AREV':
                gap = frame['a'] * frame['a'] - self.N
                if gap >= 0:
                    b = _isqrt(gap)
                    frame['b'] = b
                    if b * b == gap:
                        p, q = frame['a'] - b, frame['a'] + b
                        if p > 1 and p * q == self.N:
                            frame['result'] = (p, q)
                            frame['committed'] = True

            elif op == 'CLINK':
                a, b = frame['a'], frame['b']
                frame['a'] = a - b
                frame['b'] = a + b

            elif op == 'IMSCRIB':
                if not frame['reg']:
                    frame['reg'] = frozenset({'T'})
                frame['gap'] = frame['a'] * frame['a'] - self.N

            elif op == 'FSPLIT':
                inner = self._seed()
                inner['depth'] = frame['depth'] + 1
                inner['a'] = frame['a']
                inner['b'] = frame['b']
                inner['gap'] = frame['gap']
                self.frames.append(inner)
                open_arms.append(len(self.frames) - 1)

            elif op == 'FFUSE':
                if not open_arms:
                    return 'B', None
                inner = self.frames.pop()
                open_arms.pop()
                parent = self.frames[-1]
                parent['banked'] += inner['banked']
                if inner['result']:
                    parent['result'] = inner['result']
                    parent['committed'] = True
                if inner['band']:
                    parent['band'] = True
                if inner['square']:
                    parent['square'] = True

            elif op == 'EVALT':
                frame['reg'] = frame['reg'] | {'T'}
                frame['band'] = frame['a'] * frame['a'] >= self.N

            elif op == 'EVALF':
                frame['reg'] = frame['reg'] | {'F'}
                if frame['a'] * frame['a'] - frame['b'] * frame['b'] == self.N:
                    frame['square'] = True
                    p, q = frame['a'] - frame['b'], frame['a'] + frame['b']
                    if p > 1 and p * q == self.N:
                        frame['result'] = (p, q)
                        frame['committed'] = True

            elif op == 'ENGAGR':
                frame['reg'] = frame['reg'] | {'t', 'f'}
                frame['banked'] += 1

            elif op == 'IFIX':
                frame['committed'] = True
                p, q = frame['a'], frame['b']
                if p > q > 0 and (p - q) * (p + q) == self.N:
                    frame['result'] = (p - q, p + q)
                elif p > 1 and self.N % p == 0:
                    frame['result'] = (p, self.N // p)

            self.log.append((op, dict(frame)))

            for f in self.frames:
                if f['result'] and f['result'][0] * f['result'][1] == self.N:
                    return 'T', f['result']

        for f in self.frames:
            if f['result'] and f['result'][0] * f['result'][1] == self.N:
                return 'T', f['result']
        if open_arms:
            return 'B', None
        return 'N', None


def membrane_factor(N, word=WORD_REPAIRED, verbose=False):
    m = Membrane(N)
    verdict, factors = m.run(word)
    if verbose:
        for op, frame in m.log[-20:]:
            print(f"  {op:>8}  a={frame['a']}  b={frame['b']}  "
                  f"banked={frame['banked']}  reg={sorted(frame['reg'])}  "
                  f"depth={frame['depth']}", file=sys.stderr)
    return verdict, factors


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L5. 7 BOTTLENECKS + TENSOR COMPOSITION
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

PASS_THROUGH = [0, 1, 4, 6, 7]
BOTTLENECKS  = [2, 3, 5, 8, 9, 10, 11]


def tensor(A, B):
    C = list(A)
    for i in PASS_THROUGH:
        C[i] = max(A[i], B[i])
    C[2]  = min(A[2], B[2])
    C[3]  = min(A[3], B[3])
    C[5]  = max(A[5], B[5])
    a8, b8 = A[8], B[8]
    if a8 == 3 or b8 == 3:
        C[8] = 3
    elif a8 == 1 or b8 == 1:
        C[8] = 1
    else:
        C[8] = max(a8, b8)
    C[9]  = max(A[9], B[9])
    C[10] = max(A[10], B[10])
    C[11] = max(A[11], B[11])
    return tuple(C)


def tensor_distance(A, B):
    return sum(abs(a - b) for a, b in zip(A, B))


def check_frobenius(A, B, C_ref):
    d_ab = tensor_distance(A, B)
    d_ac_bc = tensor_distance(tensor(A, C_ref), tensor(B, C_ref))
    return d_ab == d_ac_bc


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L6. ќ© BARRIER
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

def crosses_omega(C):
    return (C[11] == 3 and C[7] == 3 and C[3] == 4)


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L7. STRATIFIED VERDICT (K3 / FDE / LP)
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

K3_SLOTS  = [4, 6, 10]
FDE_SLOTS = [0, 2, 7, 9, 11]
LP_SLOTS  = [1, 3, 5, 8]


def verdict_stratified(t12):
    k3_val = sum(t12[i] % RADICES[i] for i in K3_SLOTS) % 3
    k3_verdict = ['F', 'N', 'T'][k3_val]

    fde_val = sum(t12[i] % RADICES[i] for i in FDE_SLOTS) % 4
    fde_verdict = ['F', 'N', 'T', 'B'][fde_val]

    lp_val = sum(t12[i] % RADICES[i] for i in LP_SLOTS) % 5
    lp_verdict = ['F', 'N', 'B', 'T', 'C'][lp_val]

    return {'K3': k3_verdict, 'FDE': fde_verdict, 'LP': lp_verdict}


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L8. HOLOGRAPHIC LAYER
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

def encode_binary(n):
    if n < 0:
        raise ValueError("non-negative only")
    if n == 0:
        return ""
    bits_lsb = bin(n)[2:][::-1]
    comp = ''.join('1' if b == '0' else '0' for b in bits_lsb)
    return ''.join('вК§' if c == '1' else 'вК•' for c in comp)


def decode_binary(s):
    if s == "":
        return 0
    comp_lsb = ''.join('1' if ch == 'вК§' else '0' for ch in s)
    comp_msb = comp_lsb[::-1]
    orig_msb = ''.join('1' if b == '0' else '0' for b in comp_msb)
    return int(orig_msb, 2)


def hencode(n):
    if n < 0:
        raise ValueError("non-negative only")
    if n == 0:
        return encode_binary(0)
    return encode_binary(int(bin(n)[2:]))


def hdecode(s, strict=True):
    if s == "":
        return 0
    M = decode_binary(s)
    payload = str(M)
    if not all(c in '01' for c in payload):
        if strict:
            raise ValueError(
                f"holographic payload {payload!r} is not binary; "
                f"input may be a direct encoding, not holographic"
            )
        return M
    return int(payload, 2)


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L9. G√ЦDEL-COMPLETENESS AT THE CRYSTAL LAYER
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

GODEL_CRIT_ORD   = 1
GODEL_PARITY_ORD = 4


def godel_fixed_point_in_crystal():
    t = [0] * 12
    t[8] = GODEL_CRIT_ORD
    t[3] = GODEL_PARITY_ORD
    return tuple(t)


def inc_crystal(t12):
    t = list(t12)
    t[8]  = GODEL_CRIT_ORD
    t[3]  = GODEL_PARITY_ORD
    t[11] = 3
    t[7]  = 3
    return tuple(t)


def crystal_godel_complete(t12):
    fixed = godel_fixed_point_in_crystal()
    return {
        'fixed_point_exists': fixed is not None,
        'inc_is_fixed':       inc_crystal(t12) == t12,
        'classical_absent':   t12[3] != GODEL_PARITY_ORD,
        'no_explosion':       t12[8] != 0,
        'self_imscribe':      t12[8] == GODEL_CRIT_ORD
                              and t12[3] == GODEL_PARITY_ORD,
    }


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L10. CRYSTAL ADDRESS CALCULATOR
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

INDUCTION_SYSTEM_ADDR     = 6_687_051
INSCRIBING_PROCEDURE_ADDR = 12_089_822


def report_known_addresses():
    for name, a in [('Induction system', INDUCTION_SYSTEM_ADDR),
                    ('Inscribing procedure', INSCRIBING_PROCEDURE_ADDR)]:
        t = unaddr(a)
        print(f"{name:24s} addr={a:>10}  tuple={t}")
        print(f"{'':24s} Shavian: {encode_tuple(t)}")


# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР
# L11. CLI
# вХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХРвХР

def _parse_tuple_or_addr(s):
    s = s.strip()
    if s.startswith('⟨'):
        return decode_shavian(s)
    if ',' in s:
        return tuple(int(x) for x in s.strip('()[]').split(','))
    return unaddr(int(s, 0))


def cmd_codec(args):
    for val in args.values:
        if val.startswith('⟨'):
            t = decode_shavian(val)
            print(f"{val} -> tuple={t}  addr={addr(t)}")
        else:
            a = int(val, 0)
            t = unaddr(a)
            print(f"{a:>10} -> tuple={t}  Shavian={encode_tuple(t)}")


def cmd_binary(args):
    for val in args.values:
        if all(c in 'вК§вК•' for c in val):
            print(f"{val} -> {hdecode(val, strict=False)}")
        else:
            print(f"{int(val, 0)} -> {hencode(int(val, 0))}")


def cmd_fold(args):
    factors = [int(x, 0) for x in args.factors]
    print(f"factors = {factors}")
    print(f"fold    = {fold(factors)}")


def cmd_unfold(args):
    N = (hdecode(args.N, strict=False)
         if all(c in 'вК§вК•' for c in args.N) else int(args.N, 0))
    m = args.factors
    print(f"N = {N}  (bits={N.bit_length()})  num_factors = {m}")
    res = unfold(N, num_factors=m, max_extra_bits=args.extra,
                 verbose=args.verbose, allow_trivial=args.allow_trivial)
    if not res:
        print("unfolding FAILED")
        return 1
    for fb_tuple in res[:args.show]:
        vals = sorted(bits_to_int(fb) for fb in fb_tuple)
        prod = 1
        for v in vals:
            prod *= v
        print(f"  candidate: {vals}  product={prod}  "
              f"{'OK' if prod == N else 'BAD'}")
    return 0


def cmd_imasm(args):
    for w in args.words:
        verdict, reason = check_word(w)
        print(f"{w}  ->  {verdict}  ({reason})")


def cmd_membrane(args):
    N = int(args.N, 0)
    word = args.word if args.word else WORD_REPAIRED
    verdict, factors = membrane_factor(N, word, verbose=args.verbose)
    print(f"N        = {N}")
    print(f"word     = {word}")
    print(f"verdict  = {verdict}")
    if factors:
        p, q = factors
        print(f"factors  = {p} × {q}")
        print(f"check    = {p * q == N}")
    return 0 if factors else 1


def cmd_membrane_demo(args):
    print("=" * 72)
    print("MEMBRANE FACTOR вАФ repaired word " + WORD_REPAIRED)
    print("=" * 72)
    tests = [
        10002200057,
        10009202107,
        10012603933,
        10021211227,
        10028019479,
        100003 * 1000003,
    ]
    for N in tests:
        verdict, factors = membrane_factor(N)
        if factors:
            p, q = factors
            ok = 'OK' if p * q == N else 'BAD'
            print(f"  N={N:<20}  →  {p} × {q}  [{ok}]")
        else:
            print(f"  N={N:<20}  →  {verdict}")


def cmd_tensor(args):
    A = _parse_tuple_or_addr(args.A)
    B = _parse_tuple_or_addr(args.B)
    C = tensor(A, B)
    print(f"A = {A}  (addr {addr(A)})")
    print(f"B = {B}  (addr {addr(B)})")
    print(f"A ⊗ B = {C}  (addr {addr(C)})")
    print(f"  Shavian: {encode_tuple(C)}")
    print(f"  crosses ќ© Barrier: {crosses_omega(C)}")
    print(f"  stratified verdict: {verdict_stratified(C)}")


def cmd_verdict(args):
    t = _parse_tuple_or_addr(args.tuple)
    print(f"tuple   = {t}")
    print(f"addr    = {addr(t)}")
    print(f"verdict = {verdict_stratified(t)}")
    print(f"ќ© cross = {crosses_omega(t)}")


def cmd_godel(args):
    print("G√ґdel-completeness at the crystal layer:")
    fixed = godel_fixed_point_in_crystal()
    print(f"  fixed point: {fixed}")
    print(f"  addr = {addr(fixed)}")
    print(f"  Shavian: {encode_tuple(fixed)}")
    print(f"\nFive faces for t = {fixed}:")
    for k, v in crystal_godel_complete(fixed).items():
        print(f"  {k}: {v}")


def cmd_regfold(args):
    factors = [int(x, 0) for x in args.factors]
    stream = register_fold_from_ints(args.order, factors)
    print(f"order   = {args.order}")
    print(f"factors = {factors}")
    print(f"fold    = {register_stream_to_int(stream)}")


def cmd_regunfold(args):
    N = int(args.N, 0)
    res = register_unfold_to_ints(args.order, N, args.factors,
                                  max_steps=args.max_steps)
    if not res:
        print("register unfolding FAILED")
        return 1
    for r in res:
        vals = sorted(r)
        prod = 1
        for v in vals:
            prod *= v
        print(f"  candidate: {vals}  product={prod}  "
              f"{'OK' if prod == N else 'BAD'}")
    return 0


def cmd_demo(args):
    print("=" * 72)
    print("L0/L1: CODEC")
    print("=" * 72)
    for a in [0, 1, 100, INDUCTION_SYSTEM_ADDR, INSCRIBING_PROCEDURE_ADDR,
              CRYSTAL - 1]:
        t = unaddr(a)
        print(f"  {a:>10} -> {t}  {encode_tuple(t)}")

    print()
    print("=" * 72)
    print("L2: IMASM VERDICTS")
    print("=" * 72)
    canonical = [
        ("⊢∈≻⊤∋⊣",   "T"),
        ("⊢∈⊙∋⊣",    "N"),
        ("⊢∈⊙⊙⊙∋⊣",  "N"),
        ("⊢∈⊞≻∋⊣",   "B"),
        ("⊢≻∈⊤⊥∋⊡⊣", "T"),
    ]
    for w, expected in canonical:
        v, r = check_word(w)
        flag = "OK " if v == expected else "?? "
        print(f"  {flag}{w:<14} -> {v}  ({r})")

    print()
    print("=" * 72)
    print("L3: SIXTEEN_3 CARRIER")
    print("=" * 72)
    print(f"  T вЙ§_t t   : {t_leq(frozenset({'T'}), frozenset({'t'}))}")
    print(f"  AREV(Tf)   : {sorted(trilattice_neg(frozenset({'T','f'})))}")
    print(f"  AREV(B)    : {sorted(trilattice_neg(B_16))}  (fixed)")
    print(f"  slice(B)   : {classical_slice(B_16)}")

    print()
    print("=" * 72)
    print("L4: FOLD / UNFOLD (direct triangular solve)")
    print("=" * 72)
    factors = [100003, 100019]
    N = fold(factors)
    print(f"  fold({factors}) = {N}")
    res = unfold(N, num_factors=2, max_extra_bits=32)
    if res:
        vals = sorted(bits_to_int(fb) for fb in res[0])
        print(f"  unfold({N}) = {vals}  "
              f"{'OK' if vals == sorted(factors) else 'BAD'}")

    print()
    print("=" * 72)
    print("L4b: BIT-REGISTER FOLD")
    print("=" * 72)
    for order in ['info', 'truth', 'constr']:
        Nr = register_stream_to_int(register_fold_from_ints(order, factors))
        print(f"  order={order:<7}  regfold = {Nr}")

    print()
    print("=" * 72)
    print("L4c: MEMBRANE вАФ repaired word")
    print("=" * 72)
    print(f"  word = {WORD_REPAIRED}")
    for N in [10002200057, 10009202107, 100003 * 1000003]:
        verdict, factors = membrane_factor(N)
        if factors:
            p, q = factors
            print(f"  N={N:<20} → {p} × {q}  "
                  f"[{'OK' if p*q == N else 'BAD'}]")
        else:
            print(f"  N={N:<20} → {verdict}")

    print()
    print("=" * 72)
    print("L5: TENSOR")
    print("=" * 72)
    A = unaddr(INDUCTION_SYSTEM_ADDR)
    B = unaddr(INSCRIBING_PROCEDURE_ADDR)
    C = tensor(A, B)
    print(f"  induction ⊗ inscribing = {C}")
    print(f"  Shavian: {encode_tuple(C)}")
    print(f"  crosses ќ©: {crosses_omega(C)}")

    print()
    print("=" * 72)
    print("L6-L7: STRATIFIED VERDICT")
    print("=" * 72)
    for t in [A, B, C]:
        print(f"  {t}  ->  {verdict_stratified(t)}")

    print()
    print("=" * 72)
    print("L8: HOLOGRAPHIC LAYER")
    print("=" * 72)
    for n in [100003, 100019]:
        h = hencode(n)
        assert hdecode(h) == n
        print(f"  {n} -> {h}")

    print()
    print("=" * 72)
    print("L9: G√ЦDEL-COMPLETENESS (O(1) fixed point)")
    print("=" * 72)
    fixed = godel_fixed_point_in_crystal()
    print(f"  fixed: {fixed}  addr={addr(fixed)}")
    print(f"  Shavian: {encode_tuple(fixed)}")

    print()
    print("=" * 72)
    print("L10: KNOWN ADDRESSES")
    print("=" * 72)
    report_known_addresses()


def main():
    parser = argparse.ArgumentParser(
        prog="numnifold",
        description="Imscribing Grammar factorization: codec, IMASM, "
                    "fold/unfold, membrane, tensor.",
    )
    sub = parser.add_subparsers(dest='cmd')

    p = sub.add_parser('codec', help='12-tuple <-> address <-> Shavian')
    p.add_argument('values', nargs='+')
    p.set_defaults(func=cmd_codec)

    p = sub.add_parser('binary', help='holographic binary encode/decode')
    p.add_argument('values', nargs='+')
    p.set_defaults(func=cmd_binary)

    p = sub.add_parser('fold', help='multiply factors via convolution')
    p.add_argument('factors', nargs='+')
    p.set_defaults(func=cmd_fold)

    p = sub.add_parser('unfold', help='recover factors from N (triangular solve)')
    p.add_argument('N')
    p.add_argument('-f', '--factors', type=int, required=True)
    p.add_argument('-e', '--extra', type=int, default=32)
    p.add_argument('-s', '--show', type=int, default=5)
    p.add_argument('-v', '--verbose', action='store_true')
    p.add_argument('--allow-trivial', action='store_true')
    p.set_defaults(func=cmd_unfold)

    p = sub.add_parser('membrane', help='factor N via the repaired membrane word')
    p.add_argument('N')
    p.add_argument('-w', '--word', default=None,
                   help=f'override word (default: {WORD_REPAIRED})')
    p.add_argument('-v', '--verbose', action='store_true')
    p.set_defaults(func=cmd_membrane)

    p = sub.add_parser('membrane-demo', help='run the membrane demo')
    p.set_defaults(func=cmd_membrane_demo)

    p = sub.add_parser('imasm', help='check an IMASM word')
    p.add_argument('words', nargs='+')
    p.set_defaults(func=cmd_imasm)

    p = sub.add_parser('tensor', help='Frobenius tensor of two tuples')
    p.add_argument('A')
    p.add_argument('B')
    p.set_defaults(func=cmd_tensor)

    p = sub.add_parser('verdict', help='stratified K3/FDE/LP verdict')
    p.add_argument('tuple')
    p.set_defaults(func=cmd_verdict)

    p = sub.add_parser('godel', help='G√ґdel-completeness at crystal layer')
    p.set_defaults(func=cmd_godel)

    p = sub.add_parser('regfold', help='bit-register fold')
    p.add_argument('factors', nargs='+')
    p.add_argument('-o', '--order', choices=['info', 'truth', 'constr'],
                   default='info')
    p.set_defaults(func=cmd_regfold)

    p = sub.add_parser('regunfold', help='bit-register unfold (info order)')
    p.add_argument('N')
    p.add_argument('-f', '--factors', type=int, required=True)
    p.add_argument('-o', '--order', choices=['info', 'truth', 'constr'],
                   default='info')
    p.add_argument('--max-steps', type=int, default=64)
    p.set_defaults(func=cmd_regunfold)

    p = sub.add_parser('demo', help='run all layer demos')
    p.set_defaults(func=cmd_demo)

    args = parser.parse_args()
    if args.cmd is None:
        cmd_demo(args)
        return
    rc = args.func(args)
    sys.exit(rc if isinstance(rc, int) else 0)


if __name__ == '__main__':
    main()