s = """⊥⊤⊤⊥⊤⊥⊤⊤⊥⊥⊥⊥⊤⊤⊤⊤⊥⊤⊤⊥⊤⊤⊤⊥⊥⊤⊥⊤⊤⊥⊤⊤⊤⊤⊥⊤⊥⊥⊥⊥⊥⊥⊥⊥⊥⊤⊥⊤⊤⊥⊥⊥⊤⊤⊤⊥⊤⊥⊥⊤⊤⊤⊤⊥⊤⊤⊥⊥⊥⊤⊤⊤⊤⊤⊤⊤⊤⊥⊤⊥⊥⊤⊥⊤⊤⊥⊥⊥⊥⊥⊥⊤⊤⊤⊥⊥⊥⊤⊥⊤⊤⊤⊥⊥⊥⊤⊥⊤⊥⊤⊥⊤⊥⊥⊥⊤⊤⊤⊥⊥⊥⊤⊥⊥⊤⊤⊤⊥⊤⊥⊤⊥⊤⊥⊥⊤⊥⊤⊤⊤⊤⊤⊤⊤⊤⊥⊥⊥⊥⊥⊤⊥⊤⊤⊥⊤⊥⊥⊥⊥⊤⊤⊤⊥⊤⊥⊤⊤⊥⊥⊥⊤⊤⊤⊤⊥⊤⊥⊥⊥⊥⊥⊤⊥⊤⊤⊤⊤⊤⊥⊥⊥⊥⊥⊤⊥⊤⊤⊥⊤⊥⊤⊥⊥⊤⊥⊤⊤⊥⊥⊥⊥⊥⊥⊤⊥⊤⊥⊥⊥⊤⊥⊥⊥⊤⊤⊥⊥⊥⊥⊥⊥⊥⊤⊤⊤⊥⊤⊥⊥⊤⊥⊥⊥⊤⊥⊤⊥⊥⊥⊥⊤⊤⊤⊤⊤⊥⊥⊤⊥⊤⊥⊤⊥⊤⊥⊥⊤⊥⊥⊤⊥⊤⊤⊤⊤⊥⊥⊤⊤⊥⊤⊤⊥⊤⊤⊤⊥⊥⊤⊥⊤⊤⊥⊤⊥⊤⊤⊥⊤⊥⊤⊥⊥⊥⊤⊤⊥⊥⊤⊥⊥⊥⊤⊤⊤⊤⊥⊥⊤⊤⊤⊥⊤⊥⊥⊥⊥⊤⊥"""

# accept full expression or bare C
rhs = s.split("=", 1)[1] if ("*" in s and "=" in s) else s
bits = "".join(ch for ch in rhs if ch in "⊥⊤").replace("⊥","0").replace("⊤","1")
C = int(bits, 2)
L = len(bits)
FB = L // 2                 # factor bit-width

# peel powers of 2 so both factors can be treated odd
v = (C & -C).bit_length() - 1
Codd = C >> v
print(f"|C|={L}  v2(C)={v}  bitlength(Codd)={Codd.bit_length()}")

# ── unknit: LSB-first lifting of A,B mod 2^(k+1) against Codd ──
sols = {(1, 1)}             # both factors odd after stripping 2^v
for k in range(1, FB):
    mask = (1 << (k+1)) - 1
    tgt  = Codd & mask
    nxt  = set()
    for A, B in sols:
        for da in (0, 1):
            for db in (0, 1):
                A2, B2 = A | (da << k), B | (db << k)
                if ((A2 * B2) & mask) == tgt and A2 * B2 <= Codd:
                    nxt.add((A2, B2))
    sols = nxt
    if not sols:
        break

# put the 2^v back, split between the two factors
pairs = []
for A, B in sols:
    for x in range(v + 1):
        A2, B2 = A << x, B << (v - x)
        if A2 * B2 == C:
            pairs.append((A2, B2))

print(f"\nproduced factor pairs: {len(pairs)}")
for A, B in pairs:
    print(f"\nA = {A}  ({A.bit_length()} bits)")
    print(f"B = {B}  ({B.bit_length()} bits)")
    print(f"A*B == C            : {A*B == C}")
    acc = sum(A << i for i in range(FB) if (B >> i) & 1)
    print(f"shift-and-add == C  : {acc == C}")

    Ab = bin(A)[2:].zfill(FB)
    Bb = bin(B)[2:].zfill(FB)
    print("A:", Ab.replace("0","⊥").replace("1","⊤"))
    print("B:", Bb.replace("0","⊥").replace("1","⊤"))