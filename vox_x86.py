"""Which x86 backend V⊙x reads through.

V⊙x's own decoder is the default, so the tool has no dependency it cannot
satisfy from this directory. Setting VOX_CAPSTONE=1 swaps in capstone instead,
which is how the two are cross-checked against each other: the same corpus,
the same verify.py, two independent decoders.
"""
import os

USING_CAPSTONE = bool(os.environ.get("VOX_CAPSTONE"))

if USING_CAPSTONE:
    import capstone as backend
    from capstone.x86 import X86_OP_IMM, X86_OP_MEM, X86_OP_REG  # noqa: F401
else:
    import vox_decode as backend
    from vox_decode import X86_OP_IMM, X86_OP_MEM, X86_OP_REG  # noqa: F401

CS_ARCH_X86 = backend.CS_ARCH_X86
CS_MODE_32 = backend.CS_MODE_32
CS_MODE_64 = backend.CS_MODE_64
Cs = backend.Cs


def name():
    return "capstone" if USING_CAPSTONE else "vox_decode"
