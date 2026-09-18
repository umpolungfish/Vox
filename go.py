#!/usr/bin/env python3
"""Relate perfect_one delta lanes to known factors. Every relation tested exactly."""
import re, subprocess
from math import gcd
from fractions import Fraction

VOX = "/home/mrnob0dy666/imsgct/Vox"

# Known factorizations of RSA Factoring Challenge numbers, all bits verified.
FACTORS = {
 "1522605027922533360535618378132637429718068114961380688657908494580122963258952897654000350692006139":
   (37975227936943673922808872755445627854565536638199,
    40094690950920881030683735292761468389214899724061),
 "35794234179725868774991807832568455403003778024228226193532908190484670252364677411513516111204504060317568667":
   (6122421090493547576937037317561418841225758554253106999,
    5846418214406154678836553182979162384198610505601062333),
 "227010481295437363334259960947493668895875336466084780038173258247009162675779735389791151574049166747880487470296548479":
   (327414555693498015751146303749141488063642403240171463406883,
    693342667110830181197325401899700641361965863127336680673013),
 "114381625757888867669235779976146612010218296721242362562561842935706935245733897830597123563958705058989075147599290026879543541":
   (3490529510847650949147849619903898133417764638493387843990820577,
    32769132993266709549961988190834461413177642967992942539798288533),
 "1807082088687404805951656164405905566278102516769401349170127021450056662540244048387341127590812303371781887966563182013214880557":
   (39685999459597454290161126162883786067576449112810064832555157243,
    45534498646735972188403686897274408864356301263205069600999044599),
 "21290246318258757547497882016271517497806703963277216278233383215381949984056495911366573853021918316783107387995317230889569230873441936471":
   (3398717423028438554530123627613875835633986495969597423490929302771479,
    6264200187401285096151654948264442219302037178623509019111660653946049),
 "155089812478348440509606754370011861770654545830995430655466945774312632703463465954363335027577729025391453996787414027003501631772186840890795964683":
   (348009867102283695483970451047593424831012817350385456889559637548278410717,
    445647744903640741533241125787086176005442536297766153493419724623783054483),
 "10941738641570527421809707322040357612003732945449205990913842131476349984288934784717997257891267332497625752899781833797076537244027146743531593354333897":
   (102639592829741105772054196573991675900716567808038066803341933521790711307779,
    106603488380168454820927220360012878679207958575989291522270608237193062808643),
}

def run(n):
    return subprocess.check_output(["./perfect_one.sh", str(n), "2"],
                                   cwd=VOX, text=True, timeout=180)

def parse(out):
    g = lambda pat: int(re.search(pat, out)[1])
    return (g(r"delta L1: lane0=(\d+)"),
            g(r"delta L1: lane0=\d+ lane1=(\d+)"),
            g(r"delta L2: lane0=(\d+)"),
            g(r"delta L2: lane0=\d+ lane1=(\d+)"),
            g(r"core transform: lane product = (\d+)"),
            g(r"mu\s+L2: recombined = (\d+)"),
            g(r"mu\s+L1: recombined = (\d+)"))

def cf_convergents(a, b, limit=40):
    """Continued fraction convergents of a/b, as (num, den) pairs."""
    out = []
    while b and limit > 0:
        q, r = divmod(a, b)
        out.append((q, r))
        a, b = b, r
        limit -= 1
    # Build convergents
    p0, p1 = 0, 1
    q0, q1 = 1, 0
    convs = []
    for k, _ in out:
        p = k*p1 + p0
        q = k*q1 + q0
        convs.append((p, q))
        p0, p1 = p1, p
        q0, q1 = q1, q
    return convs

for N_str, (p_str, q_str) in FACTORS.items():
    N, p, q = int(N_str), int(p_str), int(q_str)
    print(f"\n=== RSA-{len(N_str)} ===")
    try:
        L1_0, L1_1, L2_0, L2_1, core, mu2, mu1 = parse(run(N))
    except Exception as e:
        print("  run failed:", e); continue

    # structural identities
    assert mu2 == L1_0 and mu1 == N and core == L2_0 * L2_1, "identity broke"

    # 1. Lane residues mod factors
    for name, v in [("L1_0", L1_0), ("L1_1", L1_1), ("L2_0", L2_0), ("L2_1", L2_1), ("core", core)]:
        rp, rq = v % p, v % q
        print(f"  {name} mod p = {rp if rp.bit_length() < 200 else str(rp)[:20]+'...'}")
        print(f"  {name} mod q = {rq if rq.bit_length() < 200 else str(rq)[:20]+'...'}")

    # 2. gcd of lanes and small multiples
    for name, v in [("L1_0", L1_0), ("L1_1", L1_1), ("L1_0+L1_1", L1_0+L1_1),
                    ("L1_0-L1_1", abs(L1_0-L1_1)), ("L1_0*L1_1", L1_0*L1_1),
                    ("core", core), ("L2_0+L2_1", L2_0+L2_1)]:
        g = gcd(v, N)
        if 1 < g < N:
            print(f"  *** gcd({name}, N) = {g}  <-- FACTOR")

    # 3. N = k*(L1_0*L1_1) + eps?
    prod = L1_0 * L1_1
    print(f"  N // (L1_0*L1_1) = {N // prod}")
    print(f"  N mod (L1_0*L1_1) = {N % prod}")

    # 4. N mod (L1_0 + L1_1) and mod (L1_1 - L1_0)
    for name, v in [("sum", L1_0+L1_1), ("diff", abs(L1_1-L1_0))]:
        r = N % v
        print(f"  N mod ({name}) = {r}")
        g = gcd(r, N) if r else 0
        if 1 < g < N:
            print(f"  *** gcd(N mod {name}, N) = {g}  <-- FACTOR")

    # 5. Continued fraction of L1_1/L1_0, check for p or q as denominator
    convs = cf_convergents(L1_1, L1_0, 200)
    for num, den in convs:
        if den in (p, q) or num in (p, q):
            print(f"  *** CF convergent hits factor: {num}/{den}")
        g = gcd(den, N)
        if 1 < g < N:
            print(f"  *** CF denominator gcd: {g}")

    # 6. Test specific relations
    for k in range(1, 20):
        if N % (k * L1_0) == 0 or N % (k * L1_1) == 0:
            print(f"  *** {k}*L1 lane divides N")

    # 7. Fermat gap relation on lanes
    a = (p + q) // 2
    b = (q - p) // 2
    print(f"  (L1_0 + L1_1)/2 - a = {(L1_0+L1_1)//2 - a}")
    print(f"  (L1_1 - L1_0)/2 - b = {(L1_1-L1_0)//2 - b}")
    print(f"  L1_1 - L1_0 - (q-p)  = {L1_1-L1_0-(q-p)}")
    print(f"  L1_0 + L1_1 - (p+q)  = {L1_0+L1_1-(p+q)}")