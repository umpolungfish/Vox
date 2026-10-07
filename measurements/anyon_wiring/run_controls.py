#!/usr/bin/env python3
"""Source-bound dispatch checks on independently generated RSA-style semiprimes."""
import hashlib
import json
import math
import os
import re
import subprocess
import time
from pathlib import Path
from sympy import isprime, nextprime

HERE = Path(__file__).resolve().parent
VOX = HERE.parents[1]


def cases():
    result = []
    for bits in [200, 256]:
        width = bits // 2
        primes = []
        for label in ["p", "q"]:
            seed = hashlib.sha256(f"anyon-dispatch-rsa-{bits}-{label}-v1".encode()).digest()
            candidate = int.from_bytes(seed, "big") & ((1 << (width - 2)) - 1)
            prime = int(nextprime(candidate | (3 << (width - 2))))
            assert prime.bit_length() == width and isprime(prime)
            primes.append(prime)
        p, q = primes
        n = p * q
        assert n.bit_length() == bits and p != q
        assert abs(p - q) > (1 << (width - 8))
        assert math.gcd(65537, (p - 1) * (q - 1)) == 1
        result.append({"bits": bits, "n": str(n), "p": str(p), "q": str(q),
                       "prime_bits": [p.bit_length(), q.bit_length()],
                       "prime_gap_bits": abs(p - q).bit_length(), "public_exponent": 65537})
    return result


def main():
    fixtures = cases()
    (HERE / "rsa_cases.json").write_text(json.dumps(fixtures, indent=2) + "\n")
    dependencies = VOX / "target/release/deps"
    rlib = max(dependencies.glob("libvox-*.rlib"), key=lambda p: p.stat().st_mtime)
    source = VOX / "src/bin/hyperstack_one.rs"
    records = []
    for fixture in fixtures:
        n = int(fixture["n"])
        word = subprocess.check_output([VOX / "target/release/vox", "numeral", str(n)], text=True).strip()
        for carrier in ["anyon", "mk", "semiprime anyon"]:
            env = os.environ.copy()
            # Only N and the selected carrier enter execution; test factors stay in the verifier.
            env.update(FACTOR_N_WORD=word, HYPERSTACK_TYPES=carrier, HYPERSTACK_ROUNDS="50000000")
            binary = HERE / f"rsa_{fixture['bits']}_{carrier.replace(' ', '_')}"
            subprocess.run(["rustc", "+stable", "--edition=2021", "-C", "opt-level=2", source,
                            "--extern", "vox=" + str(rlib), "-L", "dependency=" + str(dependencies),
                            "-o", binary], env=env, check=True)
            start = time.monotonic()
            try:
                output = subprocess.run([binary], capture_output=True, text=True, timeout=10, check=False)
                stdout, stderr, code = output.stdout, output.stderr, output.returncode
            except subprocess.TimeoutExpired as error:
                stdout = (error.stdout or b"").decode()
                stderr = (error.stderr or b"").decode()
                code = None
            pair = re.search(r"= (\d+) x (\d+)", stdout)
            verified = bool(pair and int(pair[1]) > 1 and int(pair[2]) > 1
                            and int(pair[1]) * int(pair[2]) == n
                            and {int(pair[1]), int(pair[2])} == {int(fixture['p']), int(fixture['q'])})
            records.append({"bits": fixture["bits"], "n": str(n), "carrier": carrier,
                            "returncode": code, "stdout": stdout, "stderr": stderr,
                            "seconds": time.monotonic() - start, "product_verified": verified,
                            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
                            "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
                            "vox_rlib_sha256": hashlib.sha256(rlib.read_bytes()).hexdigest()})
            (HERE / "rsa_dispatch.json").write_text(json.dumps(records, indent=2) + "\n")
            print(json.dumps({k: records[-1][k] for k in ["bits", "carrier", "returncode", "product_verified", "seconds"]}), flush=True)
    return 0 if all(record["product_verified"] for record in records) else 1


if __name__ == "__main__":
    raise SystemExit(main())
