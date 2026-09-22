#!/usr/bin/env python3
"""
Belnap FOUR-style ⊤/⊥ encoding with an optional holographic layer.

Two encoding modes:

1. DIRECT  (encode/decode)
    n  ->  reverse(complement(bin(n)))  ->  ⊤/⊥ string

2. HOLOGRAPHIC  (hencode/hdecode)
    n  ->  bin(n)[2:]  (binary string, MSB first)
       ->  int(binary_string)  (treat that string as decimal digits)
       ->  reverse(complement(bin(that)))  ->  ⊤/⊥ string

The holographic layer wraps n's binary form in a decimal layer,
then encodes the decimal integer.  It's the "state contains its
own index" pattern: the ⊤/⊥ string describes a number whose
decimal digits spell out n's binary.

Convention:  ⊤ = 1,  ⊥ = 0
Encoding:    reverse(complement(binary(n))), LSB-first
0            -> "" (empty string)
"""

import argparse
import sys

TOP = "⊤"
BOT = "⊥"


# ---------------------------------------------------------------------------
# Direct encoding
# ---------------------------------------------------------------------------

def encode(n: int) -> str:
    if n < 0:
        raise ValueError("only non-negative integers supported")
    if n == 0:
        return ""
    bits_lsb = bin(n)[2:][::-1]
    comp = ''.join('1' if b == '0' else '0' for b in bits_lsb)
    return ''.join(TOP if c == '1' else BOT for c in comp)


def decode(s: str) -> int:
    if s == "":
        return 0
    comp_lsb = ''.join('1' if ch == TOP else '0' for ch in s)
    comp_msb = comp_lsb[::-1]
    orig_msb = ''.join('1' if b == '0' else '0' for b in comp_msb)
    return int(orig_msb, 2)


# ---------------------------------------------------------------------------
# Holographic encoding
# ---------------------------------------------------------------------------

def hencode(n: int) -> str:
    """Holographic encode: wrap bin(n) in a decimal layer, then ⊤/⊥ encode."""
    if n < 0:
        raise ValueError("only non-negative integers supported")
    if n == 0:
        return encode(0)
    b = bin(n)[2:]              # binary string, MSB first
    M = int(b)                  # treat the binary digits as decimal digits
    return encode(M)


def hdecode(s: str) -> int:
    """Inverse of hencode."""
    if s == "":
        return 0
    M = decode(s)               # recovers the decimal-wrapped integer
    b = str(M)                  # decimal digits spell out the binary string
    return int(b, 2)            # interpret those digits as binary


# ---------------------------------------------------------------------------
# Bit helpers
# ---------------------------------------------------------------------------

def bit_list(n: int):
    if n == 0:
        return [0]
    return [(n >> i) & 1 for i in range(n.bit_length())]


def bits_to_int(bits):
    n = 0
    for i, b in enumerate(bits):
        if b:
            n |= 1 << i
    return n


# ---------------------------------------------------------------------------
# Fold
# ---------------------------------------------------------------------------

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


# ---------------------------------------------------------------------------
# Unfold
# ---------------------------------------------------------------------------

def unfold(N: int, num_factors: int, max_extra_bits: int = 32,
           max_states: int = 4096, verbose: bool = False):
    from itertools import combinations

    if N <= 0:
        raise ValueError("N must be positive")
    if N % 2 == 0:
        raise ValueError("N must be odd")

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
            T = dp.get(k, 0)

            base = nk - carry - T
            for count in range(num_factors + 1):
                diff = count - base
                if diff < 0 or (diff & 1):
                    continue
                new_carry = diff >> 1
                if T + count + carry != nk + 2 * new_carry:
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
            print(f"    step k={k:3d}  nk={nk}  states={len(states)}",
                  file=sys.stderr)

        if k >= L - 1:
            finished = []
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
                if ok:
                    finished.append(fb_tuple)
            if finished:
                seen_vals = set()
                out = []
                for fb_tuple in finished:
                    vals = tuple(sorted(bits_to_int(fb) for fb in fb_tuple))
                    if vals in seen_vals:
                        continue
                    seen_vals.add(vals)
                    out.append(fb_tuple)
                    prod = 1
                    for v in vals:
                        prod *= v
                    if prod == N:
                        return out
                if out:
                    return out

    out = [fb for fb, c in states if c == 0]
    return out if out else None


# ---------------------------------------------------------------------------
# CLI
# ---------------------------------------------------------------------------

def _parse_int_or_encoding(s: str) -> int:
    if s and all(ch in (TOP, BOT) for ch in s):
        return decode(s)
    return int(s, 0)


def cmd_encode(args):
    for n_str in args.numbers:
        n = int(n_str, 0)
        print(f"{n} --- {encode(n)}")


def cmd_decode(args):
    for s in args.strings:
        print(f"{s} --- {decode(s)}")


def cmd_hencode(args):
    for n_str in args.numbers:
        n = int(n_str, 0)
        print(f"{n} --- {hencode(n)}")


def cmd_hdecode(args):
    for s in args.strings:
        print(f"{s} --- {hdecode(s)}")


def cmd_fold(args):
    factors = [int(x, 0) for x in args.factors]
    N = 1
    for f in factors:
        N *= f
    conv = multiconvolution(factors)
    recovered = bits_to_int(bits_from_convolution(conv))
    print(f"factors = {factors}")
    print(f"N        = {N}")
    print(f"conv->N  = {recovered}")
    print(f"match    = {recovered == N}")


def cmd_unfold(args):
    N = _parse_int_or_encoding(args.N)
    m = args.factors
    print(f"N = {N}  (bits={N.bit_length()})")
    print(f"num_factors = {m}")
    res = unfold(N, num_factors=m,
                 max_extra_bits=args.extra,
                 max_states=args.max_states,
                 verbose=args.verbose)
    if not res:
        print("unfolding FAILED")
        return 1
    found = False
    for fb_tuple in res[:args.show]:
        vals = sorted(bits_to_int(fb) for fb in fb_tuple)
        prod = 1
        for v in vals:
            prod *= v
        mark = "✓" if prod == N else "✗"
        print(f"candidate: {vals}  product={prod}  {mark}")
        if prod == N:
            found = True
            break
    return 0 if found else 1


def cmd_demo(args):
    print("=" * 72)
    print("DIRECT ENCODE / DECODE")
    print("=" * 72)
    reference = {
        1:    "⊥",
        3:    "⊥⊥",
        9:    "⊥⊤⊤⊥",
        27:   "⊥⊥⊤⊥⊥",
        81:   "⊥⊤⊤⊤⊥⊤⊥",
        243:  "⊥⊥⊤⊤⊥⊥⊥⊥",
        729:  "⊥⊤⊤⊥⊥⊤⊥⊥⊤⊥",
        2187: "⊥⊥⊤⊥⊤⊤⊤⊥⊤⊤⊤⊥",
    }
    for n, expected in reference.items():
        got = encode(n)
        assert got == expected, f"encode({n})={got!r} != {expected!r}"
        assert decode(got) == n
    print("✓ reference rows match")
    for n in [0, 1, 2, 3, 9, 27, 81, 243, 729, 2187,
              100003, 100019, 100043, 100049]:
        print(f"{n:>8}  -> {encode(n)}")

    print()
    print("=" * 72)
    print("HOLOGRAPHIC ENCODE / DECODE")
    print("=" * 72)
    for n in [1, 3, 9, 27, 81, 243, 729, 2187,
              100003, 100019, 100043, 100049]:
        s = hencode(n)
        assert hdecode(s) == n, f"hencode round trip failed for {n}"
        print(f"{n:>8}  bin={bin(n)[2:]:<20}  -> {s}")

    print()
    print("=" * 72)
    print("HOLOGRAPHIC LAYER — verify chain")
    print("=" * 72)
    for n in [100003, 100019, 100043, 100049]:
        b = bin(n)[2:]
        M = int(b)
        s = encode(M)
        print(f"n={n}")
        print(f"  bin(n)        = {b}")
        print(f"  int(bin(n))   = {M}")
        print(f"  encode(M)     = {s}")
        print(f"  hencode(n)    = {hencode(n)}")
        print(f"  match: {s == hencode(n)}")
        print()

    print("=" * 72)
    print("FOLD")
    print("=" * 72)
    for factors in [[100003, 100019],
                    [100003, 100019, 100043],
                    [100003, 100019, 100043, 100049]]:
        N = 1
        for f in factors:
            N *= f
        conv = multiconvolution(factors)
        recovered = bits_to_int(bits_from_convolution(conv))
        print(f"{factors}  N={N}  match={recovered == N}")

    print()
    print("=" * 72)
    print("UNFOLD")
    print("=" * 72)
    for factors in [[100003, 100019],
                    [100003, 100019, 100043],
                    [100003, 100019, 100043, 100049]]:
        N = 1
        for f in factors:
            N *= f
        m = len(factors)
        res = unfold(N, num_factors=m, max_extra_bits=32)
        found = False
        if res:
            for fb_tuple in res[:5]:
                vals = sorted(bits_to_int(fb) for fb in fb_tuple)
                prod = 1
                for v in vals:
                    prod *= v
                if prod == N:
                    print(f"{factors}  ->  {vals}  ✓")
                    found = True
                    break
        if not found:
            print(f"{factors}  ->  no match")

    print()
    print("=" * 72)
    print("BLOCK VERIFICATION (holographic layer on factors)")
    print("=" * 72)
    blocks = [
        (100003, 100019),
        (100043, 100049),
        (100057, 100069),
        (100103, 100109),
        (100129, 100151),
    ]
    for p, q in blocks:
        N = p * q
        print(f"p={p}  q={q}  N={N}")
        print(f"  hencode(p) = {hencode(p)}")
        print(f"  hencode(q) = {hencode(q)}")
        print(f"  hencode(N) = {hencode(N)}")
        print()


def main():
    parser = argparse.ArgumentParser(
        prog="umnifold",
        description="⊤/⊥ encoding, holographic layer, folding, unfolding.",
    )
    sub = parser.add_subparsers(dest="cmd")

    p_enc = sub.add_parser("encode", help="direct ⊤/⊥ encode")
    p_enc.add_argument("numbers", nargs="+")
    p_enc.set_defaults(func=cmd_encode)

    p_dec = sub.add_parser("decode", help="direct ⊤/⊥ decode")
    p_dec.add_argument("strings", nargs="+")
    p_dec.set_defaults(func=cmd_decode)

    p_henc = sub.add_parser("hencode", help="holographic ⊤/⊥ encode")
    p_henc.add_argument("numbers", nargs="+")
    p_henc.set_defaults(func=cmd_hencode)

    p_hdec = sub.add_parser("hdecode", help="holographic ⊤/⊥ decode")
    p_hdec.add_argument("strings", nargs="+")
    p_hdec.set_defaults(func=cmd_hdecode)

    p_fold = sub.add_parser("fold", help="multiply factors via convolution")
    p_fold.add_argument("factors", nargs="+")
    p_fold.set_defaults(func=cmd_fold)

    p_unf = sub.add_parser("unfold", help="recover factors from N")
    p_unf.add_argument("N", help="composite — integer or ⊤/⊥ string")
    p_unf.add_argument("-f", "--factors", type=int, required=True,
                       help="number of factors")
    p_unf.add_argument("-e", "--extra", type=int, default=32)
    p_unf.add_argument("--max-states", type=int, default=4096)
    p_unf.add_argument("-s", "--show", type=int, default=5)
    p_unf.add_argument("-v", "--verbose", action="store_true")
    p_unf.set_defaults(func=cmd_unfold)

    p_demo = sub.add_parser("demo", help="run all demos")
    p_demo.set_defaults(func=cmd_demo)

    args = parser.parse_args()
    if args.cmd is None:
        cmd_demo(args)
        return
    rc = args.func(args)
    sys.exit(rc if isinstance(rc, int) else 0)


if __name__ == "__main__":
    main()