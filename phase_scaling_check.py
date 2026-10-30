"""Measure sparse phase construction on the fixed 175-bit balanced control."""
import hashlib
import json
from pathlib import Path
import subprocess
import time
from baked_case import build_case

root = Path(__file__).resolve().parent
n = '34708385599522211756186926321308977827402135941491069'
with (root / 'phase-sparse-scaling-baked.jsonl').open('x') as output:
    for qubits in [16,18,20]:
        binary, words = build_case(n, 'sparse', qubits)
        digest = hashlib.sha256(binary.read_bytes()).hexdigest()
        started = time.monotonic()
        result = subprocess.run([str(binary)], env={}, input='', capture_output=True, text=True, check=True)
        row = dict(n=n, bits=int(n).bit_length(), qubits=qubits, binary_sha256=digest,
                   binary=str(binary), imasm_inputs=words,
                   elapsed=time.monotonic()-started, stdout=result.stdout, stderr=result.stderr)
        output.write(json.dumps(row)+'\n')
        output.flush()
        print(json.dumps(row), flush=True)
