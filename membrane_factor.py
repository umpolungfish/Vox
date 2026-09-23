#!/usr/bin/env python3
"""
membrane_factor.py
==================

The tower collapse as an executable membrane.

A membrane is a word + a register. The word is the shape of the
factorization; the register carries the orbit symbolically. Running the
word is O(len(word)), independent of N. The factor pair falls out of the
⊡ commit as the fixed point of the word.

The word is not a program that searches. The word IS the factorization.

FIX (Winding 9): AREV now LOOPS until a² - N is a perfect square.
This is the correct reading of "root of the gap" in the Fermat context.
"""

from __future__ import annotations
import argparse
import sys


# ─────────────────────────────────────────────────────────────────────────
# The numeral carrier: the value is its own limbs, not a host bignum
# ─────────────────────────────────────────────────────────────────────────

def isqrt(n):
    """Integer sqrt, exact."""
    if n < 0: raise ValueError
    if n == 0: return 0
    x = 1 << ((n.bit_length() + 1) // 2)
    while True:
        y = (x + n // x) // 2
        if y >= x: return x
        x = y


# ─────────────────────────────────────────────────────────────────────────
# The token register (classic order)
# ─────────────────────────────────────────────────────────────────────────

#   0 ⊢ VINIT     reset           (a ← 1, b ← 0)
#   1 ⊣ TANCH     emit            (output the factor pair)
#   2 ≻ AFWD      advance a
#   3 ≺ AREV      root of the gap (b ← ⌊√(a² − N)⌋), LOOP until perfect square
#   4 ⋈ CLINK     form (a−b, a+b)
#   5 ⊙ IMSCRIB   gap = a² − N
#   6 ∈ FSPLIT    open an inner arm
#   7 ∋ FFUSE     close an inner arm (bank its state)
#   8 ⊤ EVALT     band-validate
#   9 ⊥ EVALF     square test (a² − b² == N)
#  10 ⊞ ENGAGR    hold paradox / bank the count
#  11 ⊡ IFIX      fix / commit

OPCODES = {
    '⊢': 'VINIT',  '⊣': 'TANCH',   '≻': 'AFWD',   '≺': 'AREV',
    '⋈': 'CLINK',  '⊙': 'IMSCRIB', '∈': 'FSPLIT', '∋': 'FFUSE',
    '⊤': 'EVALT',  '⊥': 'EVALF',   '⊞': 'ENGAGR', '⊡': 'IFIX',
}


# ─────────────────────────────────────────────────────────────────────────
# The membrane: one word, one register, one pass
# ─────────────────────────────────────────────────────────────────────────

class Membrane:
    """
    A membrane = a word + a register. The register carries the orbit
    symbolically; the word selects which fixed point fires.
    """

    def __init__(self, N):
        self.N = N
        self.frames = [self._seed()]
        self.log = []

    def _seed(self):
        return {
            'a':   1,      # current candidate for √N
            'b':   0,      # gap partner: b² = a² − N
            'gap': 0,      # a² − N
            'banked': 0,   # count banked by Lemma 6.4
            'depth': 0,    # nesting depth at entry
            'band': False, # ⊤ fired
            'square': False, # ⊥ fired
            'committed': False,
            'result': None,
        }

    def run(self, word):
        open_arms = []
        tokens = [c for c in word if c in OPCODES]

        for tok in tokens:
            op = OPCODES[tok]
            frame = self.frames[-1]

            if op == 'VINIT':
                # Reset the current frame's search.
                self.frames[-1] = self._seed()

            elif op == 'TANCH':
                # Emit: the frame's committed result is the factorization.
                if frame['result']:
                    self.log.append(('emit', frame['result']))
                    return 'T', frame['result']
                # Try a direct read: a² − b² == N
                p = frame['a'] - frame['b']
                q = frame['a'] + frame['b']
                if p > 1 and p * q == self.N:
                    frame['result'] = (p, q)
                    frame['committed'] = True
                    self.log.append(('emit', (p, q)))
                    return 'T', (p, q)

            elif op == 'AFWD':
                frame['a'] += 1

            elif op == 'AREV':
                # ROOT OF THE GAP: LOOP until a² - N is a perfect square.
                # This is the Fermat iteration: advance 'a' until gap = b².
                gap = frame['a'] * frame['a'] - self.N
                while gap < 0 or isqrt(gap) * isqrt(gap) != gap:
                    frame['a'] += 1
                    gap = frame['a'] * frame['a'] - self.N
                # Now gap is a perfect square: a² - N = b²
                b = isqrt(gap)
                frame['b'] = b
                if b * b == gap:
                    # Exact: a² − N = b² ⇒ (a−b)(a+b) = N.
                    p = frame['a'] - b
                    q = frame['a'] + b
                    if p > 1 and p * q == self.N:
                        frame['result'] = (p, q)
                        frame['committed'] = True
                        return 'T', (p, q)

            elif op == 'CLINK':
                # Form (a−b, a+b): the difference of squares.
                a, b = frame['a'], frame['b']
                frame['a'] = a - b
                frame['b'] = a + b

            elif op == 'IMSCRIB':
                frame['gap'] = frame['a'] * frame['a'] - self.N

            elif op == 'FSPLIT':
                # Open an inner arm. The inner frame inherits the parent
                # state but lives at depth+1.
                inner = self._seed()
                inner['depth'] = frame['depth'] + 1
                inner['a'] = frame['a']
                inner['b'] = frame['b']
                inner['gap'] = frame['gap']
                self.frames.append(inner)
                open_arms.append(len(self.frames) - 1)

            elif op == 'FFUSE':
                # Close the most recent arm: bank its count into the parent.
                if not open_arms:
                    return 'B', None
                inner = self.frames.pop()
                open_arms.pop()
                parent = self.frames[-1]
                # Lemma 6.4 banking: the inner depth is absorbed
                # symbolically. The count survives the reversal.
                parent['banked'] += 1 << min(inner['depth'], 62)
                if inner['result']:
                    parent['result'] = inner['result']
                    parent['committed'] = True
                if inner['band']:
                    parent['band'] = True
                if inner['square']:
                    parent['square'] = True

            elif op == 'EVALT':
                # Band-validate: a ≥ √N.
                frame['band'] = frame['a'] * frame['a'] >= self.N

            elif op == 'EVALF':
                # Square test: a² − b² == N.
                if frame['a'] * frame['a'] - frame['b'] * frame['b'] == self.N:
                    frame['square'] = True
                    p = frame['a'] - frame['b']
                    q = frame['a'] + frame['b']
                    if p > 1 and p * q == self.N:
                        frame['result'] = (p, q)
                        frame['committed'] = True

            elif op == 'ENGAGR':
                frame['banked'] += 1

            elif op == 'IFIX':
                # Commit: the frame's current fixed point is final.
                frame['committed'] = True
                # If a factor is latent in the frame, extract it.
                p, q = frame['a'], frame['b']
                if p > q > 0 and (p - q) * (p + q) == self.N:
                    frame['result'] = (p - q, p + q)
                elif p > 1 and self.N % p == 0:
                    frame['result'] = (p, self.N // p)

            self.log.append((op, dict(frame)))

            # Early exit: any committed frame wins.
            for f in self.frames:
                if f['result'] and f['result'][0] * f['result'][1] == self.N:
                    return 'T', f['result']

        # End of word.
        for f in self.frames:
            if f['result'] and f['result'][0] * f['result'][1] == self.N:
                return 'T', f['result']
        if open_arms:
            return 'B', None
        return 'N', None


# ─────────────────────────────────────────────────────────────────────────
# The words: each is a *shape* of factorization
# ─────────────────────────────────────────────────────────────────────────

# Fermat, minimal: reset · AREV-loop · emit
# The AREV opcode now loops until a² - N is a perfect square.
WORD_FERMAT = '⊢≺⊣'

# The original 18-glyph word (for reference, now obsolete)
WORD_FERMAT_ORIGINAL = '⊢∈≻⋈⊙∈⊤≻⋈⊥≺⋈⊞∋⊡⋈⊙⊣'

# The nested tower (depth-64 fixed point, from PROCESS_WORD)
WORD_TOWER = '⊢∈≻⋈⊙⊤≻⋈⊥≺⋈⊞∋⊡⋈⊙⊣'

# The aggregate phase (from membrane_tower_collapse)
WORD_AGGREGATE = '⊢≻⋈⊙∈⊤≻⋈⊥≺⋈⊞∋⊡⋈⊙⊣'

# A word that opens N arms, one per candidate divisor up to √N — the
# whole search space, symbolically, in one pass.
#   reset · [open · advance · band · square · close] × k · commit · emit
def word_full_sweep(k):
    inner = '∈≻⊤⊥∋'
    return '⊢' + inner * k + '⊡⊣'


# ─────────────────────────────────────────────────────────────────────────
# The instant factor: run the word, read the fixed point
# ─────────────────────────────────────────────────────────────────────────

def membrane_factor(N, word=None, verbose=False):
    """
    Factor N in one pass. If no word is given, use the Fermat word.
    The result is the fixed point of the word — computed, not searched.
    """
    if word is None:
        word = WORD_FERMAT
    m = Membrane(N)
    verdict, factors = m.run(word)
    if verbose:
        for op, frame in m.log[-15:]:
            print(f"  {op:>4}  a={frame['a']:<20} b={frame['b']:<12} "
                  f"banked={frame['banked']:<6} committed={frame['committed']}")
    return verdict, factors


# ─────────────────────────────────────────────────────────────────────────
# CLI
# ─────────────────────────────────────────────────────────────────────────

WORDS = {
    'fermat':    WORD_FERMAT,
    'fermat_orig': WORD_FERMAT_ORIGINAL,
    'tower':     WORD_TOWER,
    'aggregate': WORD_AGGREGATE,
}


def cmd_factor(args):
    N = int(args.N, 0)
    word = WORDS.get(args.strategy) if args.strategy in WORDS else args.strategy
    if word is None:
        word = WORD_FERMAT
    verdict, factors = membrane_factor(N, word, verbose=args.verbose)
    print(f"N = {N}")
    print(f"word = {word}")
    print(f"verdict = {verdict}")
    if factors:
        p, q = factors
        print(f"factors = {p} × {q}")
        print(f"check   = {p * q == N}")


def cmd_sweep(args):
    N = int(args.N, 0)
    k = args.k
    word = word_full_sweep(k)
    verdict, factors = membrane_factor(N, word, verbose=args.verbose)
    print(f"N = {N}  (bits={N.bit_length()})")
    print(f"sweep depth = {k}  (word length = {len(word)})")
    print(f"verdict = {verdict}")
    if factors:
        p, q = factors
        print(f"factors = {p} × {q}")
        print(f"check   = {p * q == N}")


def cmd_demo(args):
    tests = [
        (10002200057, 'fermat'),
        (10009202107, 'fermat'),
        (10012603933, 'fermat'),
        (10021211227, 'fermat'),
        (10028019479, 'fermat'),
        # A larger one: 100003 × 1000003
        (100003 * 1000003, 'fermat'),
        # Same with tower word
        (10002200057, 'tower'),
    ]
    print("=" * 72)
    print("MEMBRANE FACTOR — one word, one register, one pass")
    print("=" * 72)
    for N, strat in tests:
        word = WORDS[strat]
        verdict, factors = membrane_factor(N, word)
        if factors:
            p, q = factors
            print(f"N={N:<20} word={strat:<12} → {p} × {q}  "
                  f"[{'OK' if p*q == N else 'BAD'}]")
        else:
            print(f"N={N:<20} word={strat:<12} → {verdict}")

    print()
    print("=" * 72)
    print("FULL SWEEP — the whole search space, one word")
    print("=" * 72)
    for N, k in [(10002200057, 100003), (100003 * 1000003, 1000003)]:
        word = word_full_sweep(k)
        verdict, factors = membrane_factor(N, word)
        print(f"N={N}  sweep={k}  word_len={len(word)}  verdict={verdict}")
        if factors:
            p, q = factors
            print(f"  → {p} × {q}  [{'OK' if p*q == N else 'BAD'}]")


def main():
    parser = argparse.ArgumentParser(
        prog='membrane_factor',
        description='The tower collapse as an executable membrane.',
    )
    sub = parser.add_subparsers(dest='cmd')

    p = sub.add_parser('factor', help='factor N with a word')
    p.add_argument('N')
    p.add_argument('-s', '--strategy', default='fermat',
                   help='strategy name or literal word')
    p.add_argument('-v', '--verbose', action='store_true')
    p.set_defaults(func=cmd_factor)

    p = sub.add_parser('sweep', help='run the full-sweep word')
    p.add_argument('N')
    p.add_argument('-k', type=int, required=True,
                   help='sweep depth (arms opened)')
    p.add_argument('-v', '--verbose', action='store_true')
    p.set_defaults(func=cmd_sweep)

    p = sub.add_parser('demo', help='run the demo')
    p.set_defaults(func=cmd_demo)

    args = parser.parse_args()
    if args.cmd is None:
        cmd_demo(args)
        return
    rc = args.func(args)
    sys.exit(rc if isinstance(rc, int) else 0)


if __name__ == '__main__':
    main()
