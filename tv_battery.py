"""Run the baked dialectic factor producer over TV.txt values."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import time
from sympy import isprime
from baked_case import build_case

ROOT = Path(__file__).resolve().parent

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--input', default='/home/mrnob0dy666/imsgct/TV.txt')
    parser.add_argument('--output', required=True)
    args = parser.parse_args()
    values = [int(x) for x in Path(args.input).read_text().split()]
    revision = subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
    with Path(args.output).open('x') as report:
        for index,n in enumerate(values):
            # TV values use the arbitrary-width braid phase readout. The
            # dialectic producer is reserved for its lattice-family controls.
            binary,words = build_case(n,'phaseB')
            started=time.monotonic()
            result=subprocess.run([str(binary)],cwd=ROOT,env={},input='',capture_output=True,text=True)
            elapsed=time.monotonic()-started
            factors=[]
            for line in result.stdout.splitlines():
                if line.startswith('factors '):
                    parts=line.split()  # ['factors', p, 'x', q, '[time]']
                    factors=[int(parts[1]),int(parts[3])]
                    break
            valid=(result.returncode==0 and len(factors)==2 and all(x>1 for x in factors)
                   and factors[0]*factors[1]==n and all(isprime(x) for x in factors))
            row=dict(index=index,bits=n.bit_length(),n=str(n),producer='phaseB_fac',
                     words=words,binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),
                     revision=revision,elapsed=elapsed,returncode=result.returncode,
                     factors=[str(x) for x in factors],status='success' if valid else 'failure',
                     stdout=result.stdout,stderr=result.stderr,binary=str(binary))
            report.write(json.dumps(row)+'\n'); report.flush()
            print(index,n.bit_length(),'success' if valid else 'failure',round(elapsed,3),flush=True)
            if not valid:
                raise SystemExit(f'TV case {index} failed; evidence retained in {args.output}')

if __name__=='__main__':
    main()
