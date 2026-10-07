#!/usr/bin/env python3
"""Bake identical operands with anyon and branch carriers, then compare readouts."""
import hashlib
import json
import os
import re
import subprocess
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
VOX = HERE.parents[1]
ROOT = VOX.parent


def main():
    word = subprocess.check_output([VOX / "target/release/vox", "numeral", "8051"],
                                   text=True).strip()
    # Match the stable compiler used by the local Vox build. No source is copied.
    dependencies = VOX / "target/release/deps"
    rlib = max(dependencies.glob("libvox-*.rlib"), key=lambda p: p.stat().st_mtime)
    source = VOX / "src/bin/hyperstack_one.rs"
    records = []
    for carrier in ["anyon", "branch"]:
        env = os.environ.copy()
        env.update(FACTOR_N_WORD=word, HYPERSTACK_TYPES=carrier)
        binary = HERE / ("control_8051_" + carrier)
        subprocess.run(["rustc", "+stable", "--edition=2021", "-C", "opt-level=2",
                        source, "--extern", "vox=" + str(rlib), "-L",
                        "dependency=" + str(dependencies), "-o", binary],
                       env=env, check=True)
        start = time.monotonic()
        result = subprocess.run([binary], capture_output=True, text=True, timeout=5,
                                check=True)
        factors = re.search(r"8051 = (\d+) x (\d+)", result.stdout)
        records.append({
            "carrier": carrier, "binary": str(binary), "returncode": result.returncode,
            "stdout": result.stdout, "stderr": result.stderr,
            "seconds": time.monotonic() - start,
            "product_verified": bool(factors and int(factors[1]) * int(factors[2]) == 8051),
            "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
            "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
            "vox_rlib_sha256": hashlib.sha256(rlib.read_bytes()).hexdigest(),
        })
    (HERE / "carrier_controls.json").write_text(json.dumps(records, indent=2) + "\n")
    assert all(r["product_verified"] for r in records)
    print(json.dumps(records, indent=2))
    subprocess.run(["python3", ROOT / "render_membrane_diagram.py",
                    HERE / "control_8051_anyon", "--out", HERE / "control_wiring"],
                   check=True)
    original = ROOT / "m3mbr4n3s/factor_2112313232131132133113544696487913131_semiprime-anyon"
    subprocess.run(["python3", ROOT / "render_membrane_diagram.py", original,
                    "--out", HERE / "semiprime_anyon"], check=True)
    try:
        result = subprocess.run([original], capture_output=True, text=True,
                                timeout=5, check=False)
        reading = {"returncode": result.returncode, "stdout": result.stdout,
                   "stderr": result.stderr}
    except subprocess.TimeoutExpired as error:
        reading = {"timeout_seconds": 5,
                   "stdout": (error.stdout or b"").decode(),
                   "stderr": (error.stderr or b"").decode()}
    (HERE / "original_execution.json").write_text(json.dumps(reading, indent=2) + "\n")


if __name__ == "__main__":
    main()
