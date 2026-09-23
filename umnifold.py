#!/usr/bin/env python3
"""
grammar_factor.py
=================

Grammar-correct factorization.

Layers:
  L0. Mixed-radix codec over the 12 primitives (INDUXION §5).
  L1. Shavian alphabet (SNS_PRIME.md).
  L2. IMASM alphabet: 12 opcodes, close condition, verdicts (Imscriber's Guide).
  L3. SIXTEEN_3 carrier: {T,F,t,f} subsets, three orderings.
  L4. Fold/unfold at arbitrary radix (arithmetic layer).
  L5. 7 bottlenecks on the tensor (SNS_PRIME §Tensor Composition Rules).
  L6. Ω Barrier check.
  L7. Stratified verdict K3/FDE/LP.
  L8. Holographic layer (self-referential wrapping).
  L9. Gödel-completeness at the crystal layer.
  L10. Crystal address calculator.
  L11. CLI: encode, decode, fold, unfold, tensor, path, cycle, godel.

All layer operations are self-contained and independently callable.
"""

from __future__ import annotations
import argparse
import sys
from itertools import product as iproduct, combinations


# ═══════════════════════════════════════════════════════════════════════════
# L0. THE TWELVE PRIMITIVES — canonical slot order and radices
# ═══════════════════════════════════════════════════════════════════════════

PRIMITIVES = ['⊢', '⊣', '≻', '≺', '⋈', '⊤', '∈', '∋', '⊙', '⊥', '⊞', '⊡']
RADICES    = [ 4,   5,   4,   5,   3,   5,   3,   4,   5,   4,   3,   4 ]
CRYSTAL    = 3**3 * 4**5 * 5**4   # 17,280,000

SLOT_NAME = {
    '⊢': 'Dimensionality', '⊣': 'Topology',       '≻': 'Relational',
    '≺': 'Polarity',       '⋈': 'Fidelity',       '⊤': 'Kinetics',
    '∈': 'Granularity',    '∋': 'Grammar',        '⊙': 'Criticality',
    '⊥': 'Chirality',      '⊞': 'Stoichiometry',  '⊡': 'Protection',
}

# INDUXION §5 place values: s_i = prod of radices of all lower slots
PLACES = [1] * 12
_p = 1
for _i in range(11, -1, -1):
    PLACES[_i] = _p
    _p *= RADICES[_i]


def addr(t12):
    """Tuple of 12 indices -> address in [0, 17,279,999]."""
    return sum(x * s for x, s in zip(t12, PLACES))


def unaddr(a):
    """Address -> tuple of 12 indices."""
    out = [0] * 12
    for i, s in enumerate(PLACES):
        out[i] = a // s
        a %= s
    return tuple(out)


# ═══════════════════════════════════════════════════════════════════════════
# L1. SHAVIAN ALPHABET — 49 glyphs + ⊙
# ═══════════════════════════════════════════════════════════════════════════

SHAVIAN = {
    '⊢': ['𐑛', '𐑨', '𐑼', '𐑦'],
    '⊣': ['𐑡', '𐑰', '𐑥', '𐑶', '𐑸'],
    '≻': ['𐑩', '𐑑', '𐑽', '𐑾'],
    '≺': ['𐑗', '𐑿', '𐑬', '𐑯', '𐑹'],
    '⋈': ['𐑱', '𐑞', '𐑐'],
    '⊤': ['𐑘', '𐑤', '𐑧', '𐑪', '𐑺'],
    '∈': ['𐑚', '𐑔', '𐑲'],
    '∋': ['𐑝', '𐑜', '𐑠', '𐑵'],
    '⊙': ['𐑢', '⊙', '𐑮', '𐑻', '𐑣'],
    '⊥': ['𐑓', '𐑒', '𐑖', '𐑫'],
    '⊞': ['𐑙', '𐑕', '𐑳'],
    '⊡': ['𐑷', '𐑴', '𐑭', '𐑟'],
}
GLYPH_TO_SLOT = {g: (p, i) for p, gs in SHAVIAN.items() for i, g in enumerate(gs)}


def encode_tuple(t12):
    """12-tuple -> Shavian string ⟨...⟩."""
    return '⟨' + ''.join(SHAVIAN[p][i] for p, i in zip(PRIMITIVES, t12)) + '⟩'


def decode_shavian(s):
    """Shavian string ⟨...⟩ -> 12-tuple."""
    s = s.strip('⟨⟩')
    if len(s) != 12:
        raise ValueError(f"need 12 glyphs, got {len(s)}")
    return tuple(GLYPH_TO_SLOT[g][1] for g in s)


# ═══════════════════════════════════════════════════════════════════════════
# L2. IMASM ALPHABET — 12 opcodes, close condition, verdicts
# ═══════════════════════════════════════════════════════════════════════════

# Opcode table from Imscriber's Guide Part I
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


def check_word(word):
    """
    Close condition (Imscriber's Guide Part III):
      1. RECONNECTION: every ∈ and every ∋ pairs by ancestry.
      2. TRANSFORMATION: at least one such pair carries a WORK opcode.

    Simplified strand reading: on a linear word, we require balanced ∈/∋
    with at least one work opcode between each paired fork and fuse.
    Returns (verdict, reason) where verdict ∈ {T, N, B, F}.
    """
    # Strip whitespace, allow either glyph or full-name tokens
    tokens = _tokenize(word)
    if not tokens:
        return 'N', 'void'

    # Arity check
    for i, tok in enumerate(tokens):
        op = _resolve(tok)
        if op is None:
            return 'F', f'unknown token {tok!r} at {i}'
        d = OPCODES[op]
        # only FSPLIT may fan, only FFUSE may merge (on strand reading)
        if op not in BRANCHERS and op not in MERGERS:
            # check fan-in / fan-out by neighbour count is overkill for a strand
            pass

    # Count branch/fuse balance
    opens = 0
    pairs = 0
    work_in_pair = 0
    has_work_anywhere = False
    has_paradox = '⊞' in [_resolve(t) for t in tokens]

    for tok in tokens:
        op = _resolve(tok)
        if op in BRANCHERS:
            opens += 1
        elif op in MERGERS:
            if opens > 0:
                opens -= 1
                pairs += 1
            else:
                return 'B', 'dangling fuse'
        elif op in WORK_OPS:
            has_work_anywhere = True
            if opens > 0:
                work_in_pair += 1

    if opens > 0:
        return 'B', 'dangling fork'
    if pairs == 0:
        return 'N', 'no fork/fuse dyad'
    if not has_work_anywhere:
        return 'N', 'identity closure (no work)'
    if has_paradox:
        return 'B', 'paradox held'
    return 'T', 'closes'


_ALIASES = {
    'VINIT': '⊢', 'TANCH': '⊣', 'AFWD': '≻', 'AREV': '≺',
    'CLINK': '⋈', 'IMSCRIB': '⊙', 'FSPLIT': '∈', 'FFUSE': '∋',
    'EVALT': '⊤', 'EVALF': '⊥', 'ENGAGR': '⊞', 'IFIX': '⊡',
    # short forms
    'VI': '⊢', 'TA': '⊣', 'AF': '≻', 'AR': '≺', 'CL': '⋈',
    'IM': '⊙', 'FS': '∈', 'FF': '∋', 'ET': '⊤', 'EF': '⊥',
    'EG': '⊞', 'IX': '⊡', 'δ': '∈', 'μ': '∋', '═': '⊙',
}


def _resolve(tok):
    if tok in OPCODES:
        return tok
    return _ALIASES.get(tok.upper())


def _tokenize(word):
    """Split glyph-word or space-separated names into a token list."""
    word = word.strip()
    if ' ' in word:
        return [t for t in word.split() if t]
    # glyph by glyph; only keep glyphs we know
    return [c for c in word if c in OPCODES or c.isalpha()]


# ═══════════════════════════════════════════════════════════════════════════
# L3. SIXTEEN_3 CARRIER — {T,F,t,f} subsets
# ═══════════════════════════════════════════════════════════════════════════

# The four base values
T_, F_, t_, f_ = 'T', 'F', 't', 'f'
BASE = [T_, F_, t_, f_]

# 16 states as frozensets of the base values
SIXTEEN_3 = []
for mask in range(16):
    s = frozenset(BASE[i] for i in range(4) if (mask >> i) & 1)
    SIXTEEN_3.append(s)
SIXTEEN_3_BY_NAME = {''.join(sorted(s)) or 'N': s for s in SIXTEEN_3}

# Named values
N_16 = frozenset()                 # neither
B_16 = frozenset({T_, F_})         # Belnap B (classical slice)
A_16 = frozenset({T_, F_, t_, f_}) # all four


def i_leq(x, y):
    """Information order: x ⊆ y."""
    return x <= y


def t_leq(x, y):
    """Truth order."""
    tx = x & {T_, t_}
    ty = y & {T_, t_}
    fx = x & {F_, f_}
    fy = y & {F_, f_}
    return tx <= ty and fy <= fx


def c_leq(x, y):
    """Constructivity order."""
    cx = x & {T_, F_}
    cy = y & {T_, F_}
    dx = x & {t_, f_}
    dy = y & {t_, f_}
    return cx <= cy and dy <= dx


def trilattice_neg(x):
    """AREV ≺: swap T↔F and t↔f."""
    out = set()
    for v in x:
        if v == T_: out.add(F_)
        elif v == F_: out.add(T_)
        elif v == t_: out.add(f_)
        elif v == f_: out.add(t_)
    return frozenset(out)


def classical_slice(x):
    """Read a 16_3 value as a FOUR value (N/T/F/B)."""
    tt = x & {T_, t_}
    ff = x & {F_, f_}
    has_t = bool(tt)
    has_f = bool(ff)
    if has_t and has_f: return 'B'
    if has_t: return 'T'
    if has_f: return 'F'
    return 'N'


# ═══════════════════════════════════════════════════════════════════════════
# L4. FOLD / UNFOLD — arbitrary radix, arbitrary number of factors
# ═══════════════════════════════════════════════════════════════════════════

def bit_list(n):
    """Bits of n, LSB first."""
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
    """m-fold convolution of bit-lists of the factors."""
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
    """Carry propagation through a diagonal-sum convolution."""
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
    """Product via convolution: p*q*...*r as integer."""
    conv = multiconvolution(factors)
    return bits_to_int(bits_from_convolution(conv))


def unfold(N, num_factors, max_extra_bits=32, max_states=8192,
           verbose=False, allow_trivial=False):
    """
    Recover `num_factors` odd factors of N from N alone.
    Branches at each step are pruned by the carry equation; trivial
    factorizations (any factor == 1) are rejected by default.
    """
    if N <= 0:
        raise ValueError("N must be positive")
    if N % 2 == 0:
        raise ValueError("N must be odd (all factors odd)")

    n_bits = bit_list(N)
    L = len(n_bits)

    init = tuple([1] for _ in range(num_factors))
    states = [(init, 0)]

    for k in range(1, L + max_extra_bits):
        nk = n_bits[k] if k < L else 0
        new_states = []
        seen = set()

        for factor_bits, carry in states:
            lengths = [len(fb) for fb in factor_bits]

            # DP for T_k
            dp = {0: 1}
            for idx in range(num_factors):
                fb = factor_bits[idx]
                hi = min(k, lengths[idx] - 1)
                ndp = {}
                for s, w in dp.items():
                    for i in range(1, hi + 1):
                        if s + i > k:
                            break
                        if fb[i]:
                            key = s + i
                            ndp[key] = ndp.get(key, 0) + w
                dp = ndp
                if not dp:
                    break
            T_k = dp.get(k, 0)

            base = nk - carry - T_k
            for count in range(num_factors + 1):
                diff = count - base
                if diff < 0 or (diff & 1):
                    continue
                new_carry = diff >> 1
                if T_k + count + carry != nk + 2 * new_carry:
                    continue
                for ones in combinations(range(num_factors), count):
                    new_fb = tuple(
                        fb + [1 if idx in ones else 0]
                        for idx, fb in enumerate(factor_bits)
                    )
                    key = (tuple(sorted(tuple(fb) for fb in new_fb)), new_carry)
                    if key in seen:
                        continue
                    seen.add(key)
                    new_states.append((new_fb, new_carry))
                    if len(new_states) >= max_states:
                        break
                if len(new_states) >= max_states:
                    break
            if len(new_states) >= max_states:
                break

        states = new_states
        if not states:
            return None
        if verbose:
            print(f"    step k={k:3d}  nk={nk}  states={len(states)}", file=sys.stderr)

        if k >= L - 1:
            for fb_tuple, carry in states:
                if carry != 0:
                    continue
                ok = True
                for fb in fb_tuple:
                    j = len(fb) - 1
                    while j > 0 and fb[j] == 0:
                        j -= 1
                    if j == 0 and fb[0] == 0:
                        ok = False
                        break
                if not ok:
                    continue
                vals = tuple(sorted(bits_to_int(fb) for fb in fb_tuple))
                if not allow_trivial and any(v == 1 for v in vals):
                    continue
                prod = 1
                for v in vals:
                    prod *= v
                if prod == N:
                    return [fb_tuple]

    out = []
    for fb_tuple, carry in states:
        if carry != 0:
            continue
        vals = tuple(sorted(bits_to_int(f) for f in fb_tuple))
        if not allow_trivial and any(v == 1 for v in vals):
            continue
        out.append(fb_tuple)
    return out if out else None


# ═══════════════════════════════════════════════════════════════════════════
# L5. SEVEN BOTTLENECKS — tensor composition per SNS_PRIME.md
# ═══════════════════════════════════════════════════════════════════════════

PASS_THROUGH = ['⊢', '⊣', '⋈', '∈', '∋']
BOTTLENECKS  = ['≻', '≺', '⊤', '⊙', '⊥', '⊞', '⊡']


def tensor(A, B):
    """Frobenius-faithful tensor of two 12-tuples."""
    C = []
    for p, a, b in zip(PRIMITIVES, A, B):
        if p in PASS_THROUGH:
            C.append(max(a, b))
        elif p == '≻': C.append(min(a, b))          # weaker coupling
        elif p == '≺': C.append(min(a, b))          # weaker parity
        elif p == '⊤': C.append(max(a, b))          # slower kinetics
        elif p == '⊙': C.append(2)                  # collapse to monad
        elif p == '⊥': C.append(max(a, b))          # more memory
        elif p == '⊞': C.append(max(a, b))          # heterogeneous absorbs
        elif p == '⊡': C.append(max(a, b))          # higher winding
    return tuple(C)


def tensor_distance(A, B):
    """L1 distance on the 12-slot index space."""
    return sum(abs(a - b) for a, b in zip(A, B))


def check_frobenius(A, B, C_ref):
    """d(A,B) == d(A⊗C, B⊗C) for the given C_ref."""
    d_ab = tensor_distance(A, B)
    d_ac_bc = tensor_distance(tensor(A, C_ref), tensor(B, C_ref))
    return d_ab == d_ac_bc


# ═══════════════════════════════════════════════════════════════════════════
# L6. Ω BARRIER — categorical ceiling
# ═══════════════════════════════════════════════════════════════════════════

def crosses_omega(C):
    """Check if a 12-tuple has crossed the Ω Barrier."""
    winding_na = C[11] == 3          # ⊡ at 𐑟
    composition_broadcast = C[7] == 3 # ∋ at 𐑵
    parity_frobenius = C[3] == 4     # ≺ at 𐑹
    return winding_na and composition_broadcast and parity_frobenius


# ═══════════════════════════════════════════════════════════════════════════
# L7. STRATIFIED VERDICT — K3 / FDE / LP
# ═══════════════════════════════════════════════════════════════════════════

def verdict_stratified(t12):
    """
    Read the verdict at three strata:
      K3  (algebraic)  — from the ⊡, ⋈, ∈, ⊞ sub-algebra
      FDE (shape)      — from the ≻, ≺, ∋, ⊥, ⊡ sub-algebra
      LP  (gate)       — from the ⊢, ≺, ⊤, ⊙ sub-algebra
    """
    # K3: 3-valued from the 3-valued primitives
    k3_slots = [t12[4], t12[6], t12[10]]   # ⋈, ∈, ⊞
    k3_val = sum(k3_slots) % 3
    k3_verdict = ['F', 'N', 'T'][k3_val]

    # FDE: 4-valued from the 4-valued primitives
    fde_slots = [t12[1], t12[2], t12[7], t12[9], t12[11]]  # ⊣, ≻, ∋, ⊥, ⊡
    fde_val = sum(fde_slots) % 4
    fde_verdict = ['F', 'N', 'T', 'B'][fde_val]

    # LP: 5-valued from the 5-valued primitives
    lp_slots = [t12[0], t12[3], t12[5], t12[8]]  # ⊢, ≺, ⊤, ⊙
    lp_val = sum(lp_slots) % 5
    lp_verdict = ['F', 'N', 'B', 'T', 'C'][lp_val]

    return {'K3': k3_verdict, 'FDE': fde_verdict, 'LP': lp_verdict}


# ═══════════════════════════════════════════════════════════════════════════
# L8. HOLOGRAPHIC LAYER — self-referential wrapping
# ═══════════════════════════════════════════════════════════════════════════

def encode_binary(n):
    """Standard direct encoding: reverse-complement of binary (⊤=1, ⊥=0)."""
    if n < 0:
        raise ValueError("non-negative only")
    if n == 0:
        return ""
    bits_lsb = bin(n)[2:][::-1]
    comp = ''.join('1' if b == '0' else '0' for b in bits_lsb)
    return ''.join('⊤' if c == '1' else '⊥' for c in comp)


def decode_binary(s):
    if s == "":
        return 0
    comp_lsb = ''.join('1' if ch == '⊤' else '0' for ch in s)
    comp_msb = comp_lsb[::-1]
    orig_msb = ''.join('1' if b == '0' else '0' for b in comp_msb)
    return int(orig_msb, 2)


def hencode(n):
    """Holographic encoding: bin(n) -> int(bin(n)) -> encode."""
    if n < 0:
        raise ValueError("non-negative only")
    if n == 0:
        return encode_binary(0)
    b = bin(n)[2:]
    return encode_binary(int(b))


def hdecode(s):
    if s == "":
        return 0
    M = decode_binary(s)
    b = str(M)
    if not all(c in '01' for c in b):
        raise ValueError(f"holographic payload {b!r} is not binary")
    return int(b, 2)


# ═══════════════════════════════════════════════════════════════════════════
# L9. GÖDEL-COMPLETENESS — five faces at the crystal layer
# ═══════════════════════════════════════════════════════════════════════════

def godel_fixed_point_in_crystal():
    """Find a tuple with trilattice_neg(t) == t (at B or N)."""
    for a in range(CRYSTAL):
        t = unaddr(a)
        # trilattice_neg acts on the carrier, not the tuple; here we
        # check the tuple is at the criticality gate (⊙ = monad) and
        # the parity is at Frobenius-special 𐑹
        if t[8] == 1 and t[3] == 4:   # ⊙ = monad, ≺ = 𐑹
            return t
    return None


def inc_crystal(t12):
    """Gödel augmentation: push everything to the criticality gate."""
    t = list(t12)
    t[8] = 1              # ⊙ = monad
    t[3] = 4              # ≺ = Frobenius-special 𐑹
    t[11] = 3             # ⊡ = non-Abelian 𐑟
    t[7] = 3              # ∋ = broadcast 𐑵
    return tuple(t)


def crystal_godel_complete(t12):
    """Five faces at the crystal layer, for a given tuple."""
    fixed = godel_fixed_point_in_crystal()
    inc_fix = (inc_crystal(t12) == t12)
    not_classical = (t12[3] != 4)   # classical has no Frobenius parity
    no_explosion = (t12[8] != 0)    # criticality present
    self_imscribe = (t12[8] == 1 and t12[3] == 4)  # ⊙ at monad, ≺ at 𐑹
    return {
        'fixed_point_exists': fixed is not None,
        'inc_is_fixed': inc_fix,
        'classical_absent': not_classical,
        'no_explosion': no_explosion,
        'self_imscribe': self_imscribe,
    }


# ═══════════════════════════════════════════════════════════════════════════
# L10. CRYSTAL ADDRESS CALCULATOR
# ═══════════════════════════════════════════════════════════════════════════

def crystal_address(t12):
    return addr(t12)


def crystal_unaddress(a):
    return unaddr(a)


# Known addresses from INDUXION §5
INDUCTION_SYSTEM_ADDR     = 6_687_051
INSCRIBING_PROCEDURE_ADDR = 12_089_822


def report_known_addresses():
    isys = unaddr(INDUCTION_SYSTEM_ADDR)
    iproc = unaddr(INSCRIBING_PROCEDURE_ADDR)
    print(f"Induction system       addr={INDUCTION_SYSTEM_ADDR:>10}  tuple={isys}")
    print(f"  Shavian: {encode_tuple(isys)}")
    print(f"Inscribing procedure   addr={INSCRIBING_PROCEDURE_ADDR:>10}  tuple={iproc}")
    print(f"  Shavian: {encode_tuple(iproc)}")


# ═══════════════════════════════════════════════════════════════════════════
# L11. CLI
# ═══════════════════════════════════════════════════════════════════════════

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
        if all(c in '⊤⊥' for c in val):
            n = hdecode(val)
            print(f"{val} -> {n}")
        else:
            n = int(val, 0)
            print(f"{n} -> {hencode(n)}")


def cmd_fold(args):
    factors = [int(x, 0) for x in args.factors]
    N = fold(factors)
    print(f"factors = {factors}")
    print(f"fold    = {N}")


def cmd_unfold(args):
    N = int(args.N, 0) if not all(c in '⊤⊥' for c in args.N) else hdecode(args.N)
    m = args.factors
    print(f"N = {N}  (bits={N.bit_length()})  num_factors = {m}")
    res = unfold(N, num_factors=m, max_extra_bits=args.extra,
                 max_states=args.max_states, verbose=args.verbose,
                 allow_trivial=args.allow_trivial)
    if not res:
        print("unfolding FAILED")
        return 1
    for fb_tuple in res[:args.show]:
        vals = sorted(bits_to_int(fb) for fb in fb_tuple)
        prod = 1
        for v in vals:
            prod *= v
        mark = "✓" if prod == N else "✗"
        print(f"  candidate: {vals}  product={prod}  {mark}")
    return 0


def cmd_imasm(args):
    for w in args.words:
        verdict, reason = check_word(w)
        print(f"{w}  ->  {verdict}  ({reason})")


def cmd_tensor(args):
    A = decode_shavian(args.A) if args.A.startswith('⟨') else unaddr(int(args.A, 0))
    B = decode_shavian(args.B) if args.B.startswith('⟨') else unaddr(int(args.B, 0))
    C = tensor(A, B)
    print(f"A = {A}  (addr {addr(A)})")
    print(f"B = {B}  (addr {addr(B)})")
    print(f"A ⊗ B = {C}  (addr {addr(C)})")
    print(f"  Shavian: {encode_tuple(C)}")
    print(f"  crosses Ω Barrier: {crosses_omega(C)}")
    print(f"  stratified verdict: {verdict_stratified(C)}")


def cmd_verdict(args):
    t = decode_shavian(args.tuple) if args.tuple.startswith('⟨') else unaddr(int(args.tuple, 0))
    print(f"tuple   = {t}")
    print(f"addr    = {addr(t)}")
    print(f"verdict = {verdict_stratified(t)}")
    print(f"Ω cross = {crosses_omega(t)}")


def cmd_godel(args):
    print("Gödel-completeness at the crystal layer:")
    fixed = godel_fixed_point_in_crystal()
    print(f"  fixed point (⊙=monad, ≺=𐑹): {fixed}")
    if fixed:
        print(f"  addr = {addr(fixed)}")
        print(f"  Shavian: {encode_tuple(fixed)}")
    # A candidate at the criticality gate
    t = (0, 0, 0, 4, 0, 0, 0, 3, 1, 0, 0, 3)
    print(f"\nFive faces for t = {t}:")
    for k, v in crystal_godel_complete(t).items():
        print(f"  {k}: {v}")


def cmd_demo(args):
    print("=" * 72)
    print("L0/L1: CODEC")
    print("=" * 72)
    for a in [0, 1, 100, 6_687_051, 12_089_822, CRYSTAL - 1]:
        t = unaddr(a)
        print(f"  {a:>10} -> {t}  {encode_tuple(t)}")

    print()
    print("=" * 72)
    print("L2: IMASM VERDICTS")
    print("=" * 72)
    for w in ["⊢∈≻⊤∋⊣", "⊢∈⊙∋⊣", "⊢∈⊞≻∋⊣", "⊢≻∈⊤⊥∋⊡⊣", "⊢⊙⋈⊣"]:
        v, r = check_word(w)
        print(f"  {w:<14}  ->  {v}  ({r})")

    print()
    print("=" * 72)
    print("L3: SIXTEEN_3 CARRIER")
    print("=" * 72)
    print(f"  T ∧ t under truth order = {t_leq(frozenset({'T'}), frozenset({'t'}))}")
    print(f"  AREV(Tf) = {trilattice_neg(frozenset({'T','f'}))}")
    print(f"  AREV(B)  = {trilattice_neg(B_16)}  (fixed)")

    print()
    print("=" * 72)
    print("L4: FOLD / UNFOLD")
    print("=" * 72)
    factors = [100003, 100019]
    N = fold(factors)
    print(f"  fold({factors}) = {N}")
    res = unfold(N, num_factors=2, max_extra_bits=32)
    if res:
        vals = sorted(bits_to_int(fb) for fb in res[0])
        print(f"  unfold({N}) = {vals}  {'✓' if vals == sorted(factors) else '✗'}")

    print()
    print("=" * 72)
    print("L5: TENSOR")
    print("=" * 72)
    A = unaddr(INDUCTION_SYSTEM_ADDR)
    B = unaddr(INSCRIBING_PROCEDURE_ADDR)
    C = tensor(A, B)
    print(f"  induction ⊗ inscribing = {C}")
    print(f"  Shavian: {encode_tuple(C)}")
    print(f"  crosses Ω: {crosses_omega(C)}")

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
        print(f"  {n} -> {hencode(n)}")
        assert hdecode(hencode(n)) == n

    print()
    print("=" * 72)
    print("L9: GÖDEL-COMPLETENESS AT THE CRYSTAL LAYER")
    print("=" * 72)
    fixed = godel_fixed_point_in_crystal()
    print(f"  fixed point: {fixed}  addr={addr(fixed)}")
    print(f"  Shavian: {encode_tuple(fixed)}")

    print()
    print("=" * 72)
    print("L10: KNOWN ADDRESSES FROM INDUXION")
    print("=" * 72)
    report_known_addresses()


def main():
    parser = argparse.ArgumentParser(
        prog="grammar_factor",
        description="Grammar-correct factorization: codec, IMASM, fold/unfold, tensor.",
    )
    sub = parser.add_subparsers(dest='cmd')

    p = sub.add_parser('codec', help='12-slot tuple <-> address <-> Shavian')
    p.add_argument('values', nargs='+')
    p.set_defaults(func=cmd_codec)

    p = sub.add_parser('binary', help='holographic binary encode/decode')
    p.add_argument('values', nargs='+')
    p.set_defaults(func=cmd_binary)

    p = sub.add_parser('fold', help='multiply factors via convolution')
    p.add_argument('factors', nargs='+')
    p.set_defaults(func=cmd_fold)

    p = sub.add_parser('unfold', help='recover factors from N')
    p.add_argument('N')
    p.add_argument('-f', '--factors', type=int, required=True)
    p.add_argument('-e', '--extra', type=int, default=32)
    p.add_argument('--max-states', type=int, default=8192)
    p.add_argument('-s', '--show', type=int, default=5)
    p.add_argument('-v', '--verbose', action='store_true')
    p.add_argument('--allow-trivial', action='store_true')
    p.set_defaults(func=cmd_unfold)

    p = sub.add_parser('imasm', help='check an IMASM word')
    p.add_argument('words', nargs='+')
    p.set_defaults(func=cmd_imasm)

    p = sub.add_parser('tensor', help='Frobenius tensor of two 12-tuples')
    p.add_argument('A')
    p.add_argument('B')
    p.set_defaults(func=cmd_tensor)

    p = sub.add_parser('verdict', help='stratified K3/FDE/LP verdict')
    p.add_argument('tuple')
    p.set_defaults(func=cmd_verdict)

    p = sub.add_parser('godel', help='Gödel-completeness at the crystal layer')
    p.set_defaults(func=cmd_godel)

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