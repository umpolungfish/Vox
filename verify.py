#!/usr/bin/env python3
"""The recompile is executable, or it is not. This decides it.

Every function in a shared object is run twice — once natively through ctypes,
once as an IMASM module in the machine — over the same inputs. Agreement on
every input at every optimisation level is the claim; anything else is printed
as a mismatch with the arguments that produced it.
"""
import ctypes
import random
import subprocess
import sys

import imasm_module
import imasm_vm


def symbols(path):
    out = subprocess.run(["nm", "-D", "--defined-only", path],
                         capture_output=True, text=True).stdout
    syms = {}
    for line in out.splitlines():
        parts = line.split()
        if len(parts) == 3 and parts[1] in "Tt":
            syms[parts[2]] = int(parts[0], 16)
    return syms


def check(path, cases, verbose=True):
    lib = ctypes.CDLL(path)
    text = imasm_module.emit(path)
    syms = symbols(path)
    ok = bad = 0
    for name, args_list in cases.items():
        if name not in syms:
            continue
        fn = getattr(lib, name)
        fn.restype = ctypes.c_int
        fn.argtypes = [ctypes.c_int] * len(args_list[0])
        for args in args_list:
            want = fn(*args)
            m = imasm_vm.Machine(text)
            try:
                got = m.call(syms[name], *args)
            except imasm_vm.Halt as e:
                got = f"halt: {e}"
            if got == want:
                ok += 1
            else:
                bad += 1
                if verbose and bad <= 5:
                    print(f"    MISMATCH {name}{args}  native={want}  imasm={got}")
    return ok, bad


def main():
    paths = sys.argv[1:]
    r = random.Random(1729)
    cases = {
        "gcd": [(r.randint(1, 10**6), r.randint(1, 10**6)) for _ in range(40)],
        "fib": [(k,) for k in range(0, 30)],
        "sumsq": [(k,) for k in range(0, 40)],
        "clamp": [(r.randint(-99, 99), -20, 20) for _ in range(40)],
        "popcnt": [(r.randint(0, 2**31),) for _ in range(40)],
        "collatz": [(k,) for k in range(1, 60)],
        # the shapes that exercise the transfer glyphs: recursion and calls
        # (>, ⊣), a jump table and a function pointer (⊙), stack memory (◻)
        "fact": [(k,) for k in range(0, 13)],
        "ack": [(m, n) for m in range(0, 3) for n in range(0, 6)],
        "viadd": [(r.randint(-500, 500), r.randint(-500, 500)) for _ in range(30)],
        "arr": [(k,) for k in range(0, 64)],
        "sw": [(k, r.randint(-999, 999)) for k in range(0, 40)],
        "fptr": [(k, r.randint(-99, 99), r.randint(-99, 99)) for k in range(0, 30)],
        "sumdigits": [(r.randint(0, 10**7),) for _ in range(30)],
    }
    total_ok = total_bad = 0
    for p in paths:
        ok, bad = check(p, cases)
        total_ok += ok
        total_bad += bad
        print(f"{p:<28}{ok:>6} agree{('   ' + str(bad) + ' MISMATCH') if bad else ''}")
    print(f"\n{total_ok} agreements, {total_bad} mismatches")
    return 1 if total_bad else 0


if __name__ == "__main__":
    raise SystemExit(main())
