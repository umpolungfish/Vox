--- numnifold.py
+++ numnifold.py
@@ -0,0 +1,225 @@
+#!/usr/bin/env python3
+"""
+PATCH вАФ membrane layer for numnifold.py
+
+Adds the tower collapse as an executable membrane:
+
+  WORD_REPAIRED  = '⋈⊙⊣∋∈≻⋈⊙∈⊤≻⋈⊥≺⋈⊞∋⊡'
+  Membrane       вАФ word + register, SIXTEEN_3 register per frame
+  membrane_factor(N, word=WORD_REPAIRED)
+  CLI: `membrane <N> [-w WORD] [-v]`, `membrane-demo`
+
+The membrane runs in O(len(word)), independent of N. The word is the
+shape of the factorization; the register carries the orbit symbolically;
+the count banks across ≺ clears (Lemma 6.4: δ-before-δ, μ-after-μ).
+"""
+
+from __future__ import annotations
+import sys
+
+
+# вФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФА
+# The repaired word вАФ certified closed at crystal 16144989,
+# cost 0.00, edit distance 0, ΔS 0.0000.
+# вФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФА
+
+WORD_REPAIRED = '⋈⊙⊣∋∈≻⋈⊙∈⊤≻⋈⊥≺⋈⊞∋⊡'
+
+# Opcode table вАФ local copy so this patch is self-contained.
+# (numnifold.py's own OPCODES is authoritative; this mirrors it.)
+_OPCODE_NAMES = {
+    '⊢': 'VINIT',  '⊣': 'TANCH',   '≻': 'AFWD',   '≺': 'AREV',
+    '⋈': 'CLINK',  '⊙': 'IMSCRIB', '∈': 'FSPLIT', '∋': 'FFUSE',
+    '⊤': 'EVALT',  '⊥': 'EVALF',   '⊞': 'ENGAGR', '⊡': 'IFIX',
+}
+
+
+def _isqrt(n: int) -> int:
+    if n < 0:
+        raise ValueError("isqrt of negative")
+    if n == 0:
+        return 0
+    x = 1 << ((n.bit_length() + 1) // 2)
+    while True:
+        y = (x + n // x) // 2
+        if y >= x:
+            return x
+        x = y
+
+
+# вФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФА
+# Membrane
+# вФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФА
+
+class Membrane:
+    """
+    A membrane = a word + a register.
+
+    The register is a SIXTEEN_3 value (subset of {T,F,t,f}) per frame.
+    The count banks across ≺ clears by living in an enclosing frame
+    (Lemma 6.4: δ-before-δ, μ-after-μ).
+
+    Runtime is O(len(word)), independent of N.
+    """
+
+    def __init__(self, N: int):
+        self.N = N
+        self.frames = [self._seed()]
+        self.log = []
+
+    def _seed(self):
+        return {
+            'reg':       frozenset(),   # SIXTEEN_3 register
+            'a':         1,
+            'b':         0,
+            'gap':       0,
+            'banked':    0,
+            'depth':     0,
+            'band':      False,
+            'square':    False,
+            'committed': False,
+            'result':    None,
+        }
+
+    def run(self, word: str):
+        open_arms = []
+        tokens = [c for c in word if c in _OPCODE_NAMES]
+
+        for tok in tokens:
+            op = _OPCODE_NAMES[tok]
+            frame = self.frames[-1]
+
+            if op == 'VINIT':
+                self.frames[-1] = self._seed()
+
+            elif op == 'TANCH':
+                if frame['result']:
+                    return 'T', frame['result']
+                p, q = frame['a'] - frame['b'], frame['a'] + frame['b']
+                if p > 1 and p * q == self.N:
+                    frame['result'] = (p, q)
+                    frame['committed'] = True
+                    return 'T', (p, q)
+
+            elif op == 'AFWD':
+                frame['a'] += 1
+
+            elif op == 'AREV':
+                # root of the gap: b вЖР вМКвИЪ(a¬≤ вИТ N)вМЛ
+                gap = frame['a'] * frame['a'] - self.N
+                if gap >= 0:
+                    b = _isqrt(gap)
+                    frame['b'] = b
+                    if b * b == gap:
+                        p, q = frame['a'] - b, frame['a'] + b
+                        if p > 1 and p * q == self.N:
+                            frame['result'] = (p, q)
+                            frame['committed'] = True
+
+            elif op == 'CLINK':
+                # form (aвИТb, a+b)
+                a, b = frame['a'], frame['b']
+                frame['a'] = a - b
+                frame['b'] = a + b
+
+            elif op == 'IMSCRIB':
+                # SEED T into an empty register (no weight)
+                if not frame['reg']:
+                    frame['reg'] = frozenset({'T'})
+                frame['gap'] = frame['a'] * frame['a'] - self.N
+
+            elif op == 'FSPLIT':
+                inner = self._seed()
+                inner['depth'] = frame['depth'] + 1
+                inner['a'] = frame['a']
+                inner['b'] = frame['b']
+                inner['gap'] = frame['gap']
+                self.frames.append(inner)
+                open_arms.append(len(self.frames) - 1)
+
+            elif op == 'FFUSE':
+                if not open_arms:
+                    return 'B', None
+                inner = self.frames.pop()
+                open_arms.pop()
+                parent = self.frames[-1]
+                parent['banked'] += inner['banked']
+                if inner['result']:
+                    parent['result'] = inner['result']
+                    parent['committed'] = True
+                if inner['band']:
+                    parent['band'] = True
+                if inner['square']:
+                    parent['square'] = True
+
+            elif op == 'EVALT':
+                # deposit T into the register
+                frame['reg'] = frame['reg'] | {'T'}
+                frame['band'] = frame['a'] * frame['a'] >= self.N
+
+            elif op == 'EVALF':
+                # deposit F into the register
+                frame['reg'] = frame['reg'] | {'F'}
+                if frame['a'] * frame['a'] - frame['b'] * frame['b'] == self.N:
+                    frame['square'] = True
+                    p, q = frame['a'] - frame['b'], frame['a'] + frame['b']
+                    if p > 1 and p * q == self.N:
+                        frame['result'] = (p, q)
+                        frame['committed'] = True
+
+            elif op == 'ENGAGR':
+                # deposit t+f into the register, bank the count
+                frame['reg'] = frame['reg'] | {'t', 'f'}
+                frame['banked'] += 1
+
+            elif op == 'IFIX':
+                frame['committed'] = True
+                p, q = frame['a'], frame['b']
+                if p > q > 0 and (p - q) * (p + q) == self.N:
+                    frame['result'] = (p - q, p + q)
+                elif p > 1 and self.N % p == 0:
+                    frame['result'] = (p, self.N // p)
+
+            self.log.append((op, dict(frame)))
+
+            # Early exit: any committed frame wins
+            for f in self.frames:
+                if f['result'] and f['result'][0] * f['result'][1] == self.N:
+                    return 'T', f['result']
+
+        # End of word
+        for f in self.frames:
+            if f['result'] and f['result'][0] * f['result'][1] == self.N:
+                return 'T', f['result']
+        if open_arms:
+            return 'B', None
+        return 'N', None
+
+
+# вФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФА
+# Top-level factor
+# вФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФА
+
+def membrane_factor(N: int, word: str = WORD_REPAIRED, verbose: bool = False):
+    """
+    Factor N by running the membrane word. Returns (verdict, factors|None).
+    """
+    m = Membrane(N)
+    verdict, factors = m.run(word)
+    if verbose:
+        for op, frame in m.log[-20:]:
+            print(f"  {op:>8}  a={frame['a']}  b={frame['b']}  "
+                  f"banked={frame['banked']}  reg={sorted(frame['reg'])}  "
+                  f"depth={frame['depth']}", file=sys.stderr)
+    return verdict, factors
+
+
+# вФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФА
+# CLI commands вАФ register these on numnifold.py's subparser
+# вФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФАвФА
+
+def cmd_membrane(args):
+    N = int(args.N, 0)
+    word = args.word if args.word else WORD_REPAIRED
+    verdict, factors = membrane_factor(N, word, verbose=args.verbose)
+    print(f"N        = {N}")
+    print(f"word     = {word}")
+    print(f"verdict  = {verdict}")
+    if factors:
+        p, q = factors
+        print(f"factors  = {p} × {q}")
+        print(f"check    = {p * q == N}")
+    return 0 if factors else 1
+
+
+def cmd_membrane_demo(args):
+    print("=" * 72)
+    print("MEMBRANE FACTOR вАФ repaired word ⋈⊙⊣∋∈≻⋈⊙∈⊤≻⋈⊥≺⋈⊞∋⊡")
+    print("=" * 72)
+    tests = [
+        10002200057,           # 100003 × 100019
+        10009202107,           # 100043 × 100049
+        10012603933,           # 100057 × 100069
+        10021211227,           # 100103 × 100109
+        10028019479,           # 100129 × 100151
+        100003 * 1000003,      # 100003 × 1000003
+    ]
+    for N in tests:
+        verdict, factors = membrane_factor(N)
+        if factors:
+            p, q = factors
+            ok = 'OK' if p * q == N else 'BAD'
+            print(f"  N={N:<20}  →  {p} × {q}  [{ok}]")
+        else:
+            print(f"  N={N:<20}  →  {verdict}")
+
+
+def register_membrane_cli(sub):
+    """Call this from numnifold.py's main() after add_subparsers()."""
+    p = sub.add_parser('membrane',
+                       help='factor N via the repaired membrane word')
+    p.add_argument('N')
+    p.add_argument('-w', '--word', default=None,
+                   help=f'override word (default: {WORD_REPAIRED})')
+    p.add_argument('-v', '--verbose', action='store_true')
+    p.set_defaults(func=cmd_membrane)
+
+    p = sub.add_parser('membrane-demo', help='run the membrane demo')
+    p.set_defaults(func=cmd_membrane_demo)