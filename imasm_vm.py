"""A machine that runs an IMASM module.

Dispatch is on the glyph and nothing else. The twelve axes are the opcode set;
what an instruction *was* in x86 survives only as payload the glyph knows how to
read. ⊞ engages the ALU, ⋈ links two slots, ◻ commits to memory, ⊤ produces a
truth and ⊥ consumes one, ∈ splits and ∋ fuses, > calls and ⊣ terminates, <
transfers, and ⊙ transfers through data.

This is the executable end of the recompile: the module emitted by
imasm_module.emit runs here with no reference to the original binary.
"""
import re

_64 = ["rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi"] + \
      [f"r{k}" for k in range(8, 16)]
# name -> (canonical 64-bit register, byte width, byte offset within it)
_REG = {}
for _i, _r in enumerate(_64):
    _REG[_r] = (_r, 8, 0)
    if _i < 8:
        e = "e" + _r[1:] if _r[0] == "r" else _r
        w = _r[1:]
        _REG["e" + _r[1:]] = (_r, 4, 0)
        _REG[w] = (_r, 2, 0)
        if _r[-1] == "x":
            _REG[w[0] + "l"] = (_r, 1, 0)
            _REG[w[0] + "h"] = (_r, 1, 1)
        else:
            _REG[w + "l"] = (_r, 1, 0)
    else:
        _REG[_r + "d"] = (_r, 4, 0)
        _REG[_r + "w"] = (_r, 2, 0)
        _REG[_r + "b"] = (_r, 1, 0)

_MASK = {1: 0xFF, 2: 0xFFFF, 4: 0xFFFFFFFF, 8: (1 << 64) - 1,
         16: (1 << 128) - 1}
_XMM = {f"xmm{k}": (f"xmm{k}", 16, 0) for k in range(16)}
_REG.update(_XMM)
# rip is a slot like any other, and must be: a rip-relative reference reads the
# address of the NEXT instruction, so the machine has to carry the program
# counter where the effective-address unit can see it.
_REG["rip"] = ("rip", 8, 0)


def _lanes(v, width, n):
    """A 128-bit register read as n lanes of `width` bytes, low lane first."""
    m = _MASK[width]
    return [(v >> (k * width * 8)) & m for k in range(n)]


def _pack(ls, width):
    return sum((v & _MASK[width]) << (k * width * 8) for k, v in enumerate(ls))


def _sign(v, size):
    m = _MASK[size]
    v &= m
    return v - (m + 1) if v > m >> 1 else v


_SIMD = {"movdqa", "movdqu", "movaps", "movups", "movd", "movq", "pxor",
         "pand", "por", "paddd", "paddq", "paddw", "paddb", "psubd", "psubq",
         "psubw", "psubb", "pmulld", "pmuludq", "psrlq", "psllq", "psrldq",
         "pshufd", "punpckldq", "punpcklqdq", "pmuldq", "movdqu"}


class Halt(Exception):
    """A ⊣ reached with the stack unwound past where the run started."""


class SysExit(Exception):
    """exit/exit_group: a managed stop, not the machine running off the end.
    Carries the process's own exit code, same as the real syscall would."""
    def __init__(self, code):
        self.code = code
        super().__init__(f"exit({code})")


class Machine:
    def __init__(self, module_text: str, stack_top=0x7FFF0000):
        self.code = {}                 # address -> [(glyph, fields), ...]
        self.entry = None
        self.reg = dict.fromkeys(_64, 0)
        self.reg.update({f"xmm{k}": 0 for k in range(16)})
        self.reg["rip"] = 0
        self.mem = {}
        self.flags = (0, 0, 1)         # (a, b, size) of the last ⊤, lazily read
        self.kind = "cmp"
        self.reg["rsp"] = stack_top
        self.steps = 0
        self._parse(module_text)

    # ── the module ────────────────────────────────────────────────────────
    def _parse(self, text):
        addr = None
        for line in text.splitlines():
            if line.startswith(";"):
                m = re.match(r"; entry (0x[0-9a-f]+)", line)
                if m:
                    self.entry = int(m.group(1), 16)
            elif line.startswith("="):
                at, blob = line[1:].split("\t")
                at = int(at, 16)
                for k, b in enumerate(bytes.fromhex(blob)):
                    if b:
                        self.mem[at + k] = b
            elif line.startswith("@"):
                addr = int(line[1:], 16)
                self.code[addr] = []
            elif line and addr is not None:
                g, *fields = line.split("\t")
                self.code[addr].append((g, fields))
        self.addrs = sorted(self.code)
        self.next_of = {a: b for a, b in zip(self.addrs, self.addrs[1:])}

    # ── slots ─────────────────────────────────────────────────────────────
    def get_reg(self, name):
        base, size, off = _REG[name]
        return (self.reg[base] >> (off * 8)) & _MASK[size]

    def set_reg(self, name, val):
        base, size, off = _REG[name]
        if size == 8:
            self.reg[base] = val & _MASK[8]
        elif size == 4:
            self.reg[base] = val & _MASK[4]      # 32-bit writes zero the top
        else:
            m = _MASK[size] << (off * 8)
            self.reg[base] = (self.reg[base] & ~m) | ((val << (off * 8)) & m)

    def load(self, addr, size):
        return int.from_bytes(bytes(self.mem.get(addr + k, 0)
                                    for k in range(size)), "little")

    def store(self, addr, val, size):
        for k, b in enumerate(val.to_bytes(size, "little", signed=False)
                              if val >= 0 else
                              (val & _MASK[size]).to_bytes(size, "little")):
            self.mem[addr + k] = b

    def ea(self, field):
        _, base, index, scale, disp, size = field.split(":")
        a = self.get_reg(base) if base else 0
        if index:
            a += self.get_reg(index) * int(scale)
        return (a + int(disp, 16)) & _MASK[8], int(size)

    def read(self, field, size_hint=8):
        kind = field[0]
        if kind == "r":
            return self.get_reg(field[2:]), _REG[field[2:]][1]
        if kind == "i":
            s = field[2:]
            return (int(s, 16) if not s.startswith("-")
                    else -int(s[1:], 16)) & _MASK[8], size_hint
        addr, size = self.ea(field)
        return self.load(addr, size), size

    def write(self, field, val):
        if field[0] == "r":
            self.set_reg(field[2:], val)
        else:
            addr, size = self.ea(field)
            self.store(addr, val, size)

    def width(self, field):
        if field[0] == "r":
            return _REG[field[2:]][1]
        if field[0] == "m":
            return int(field.rsplit(":", 1)[1])
        return 8

    # ── truth ─────────────────────────────────────────────────────────────
    def cc(self, name):
        a, b, size = self.flags
        if self.kind == "test":
            r = (a & b) & _MASK[size]
            zf, sf, cf, of = r == 0, _sign(r, size) < 0, False, False
        else:
            r = (a - b) & _MASK[size]
            zf, sf = r == 0, _sign(r, size) < 0
            cf = (a & _MASK[size]) < (b & _MASK[size])
            of = (_sign(a, size) - _sign(b, size)) != _sign(r, size)
        return {
            "e": zf, "z": zf, "ne": not zf, "nz": not zf,
            "s": sf, "ns": not sf, "b": cf, "nae": cf, "c": cf,
            "ae": not cf, "nb": not cf, "nc": not cf,
            "be": cf or zf, "na": cf or zf, "a": not (cf or zf), "nbe": not (cf or zf),
            "l": sf != of, "nge": sf != of, "ge": sf == of, "nl": sf == of,
            "le": zf or (sf != of), "ng": zf or (sf != of),
            "g": not zf and sf == of, "nle": not zf and sf == of,
            "o": of, "no": not of, "p": False, "np": True,
        }[name]

    def set_flags(self, a, b, size, kind="cmp"):
        self.flags, self.kind = (a, b, size), kind

    # ── the syscall boundary, reached only through ⊙ ───────────────────────
    # A deliberately small, honest subset — this is a function tester, not an
    # OS. exit/exit_group stop the run cleanly with the real exit code; write
    # actually writes the requested bytes out of the machine's own memory, to
    # a real fd, so a program that prints is not silently wrong; anything else
    # returns -ENOSYS in rax, the kernel's own answer for "not implemented,"
    # rather than crashing or pretending to have done something it has not.
    _ENOSYS = -38

    def do_syscall(self):
        import os
        num = _sign(self.get_reg("rax"), 8)
        a0, a1, a2 = self.get_reg("rdi"), self.get_reg("rsi"), self.get_reg("rdx")
        if num in (60, 231):                     # exit, exit_group
            raise SysExit(_sign(a0, 8) & 0xFF)
        if num == 1:                             # write(fd, buf, count)
            data = bytes(self.mem.get(a1 + k, 0) for k in range(a2))
            try:
                n = os.write(a0, data)
            except OSError:
                n = -1
            self.set_reg("rax", n & _MASK[8])
            return
        self.set_reg("rax", self._ENOSYS & _MASK[8])

    # ── external calls, reached only through ⊙'s "external" form ──────────
    # A PLT stub is itself an indirect jump through a GOT slot the machine
    # never loaded (nothing loaded the binary, so nothing bound it) — the
    # recompiler names the stub instead of emitting that dead jump, and this
    # is where the name is read. A small, honest subset of libc, byte
    # operations only, entirely on the machine's own memory: nothing here
    # reaches outside self.mem. Everything else raises rather than silently
    # returning zero, which would look like a real answer and is not one.
    _EXTERNAL = ("memcpy", "memmove", "memset", "strlen", "strcpy", "strcmp")

    def do_external(self, name):
        rdi, rsi, rdx = self.get_reg("rdi"), self.get_reg("rsi"), self.get_reg("rdx")
        if name in ("memcpy", "memmove"):
            src = bytes(self.mem.get(rsi + k, 0) for k in range(rdx))
            for k, b in enumerate(src):
                self.mem[rdi + k] = b
            self.set_reg("rax", rdi)
        elif name == "memset":
            b = rsi & 0xFF
            for k in range(rdx):
                self.mem[rdi + k] = b
            self.set_reg("rax", rdi)
        elif name == "strlen":
            n = 0
            while self.mem.get(rdi + n, 0):
                n += 1
            self.set_reg("rax", n)
        elif name == "strcpy":
            k = 0
            while True:
                b = self.mem.get(rsi + k, 0)
                self.mem[rdi + k] = b
                if b == 0:
                    break
                k += 1
            self.set_reg("rax", rdi)
        elif name == "strcmp":
            k = 0
            while True:
                a, b = self.mem.get(rdi + k, 0), self.mem.get(rsi + k, 0)
                if a != b or a == 0:
                    self.set_reg("rax", _sign(a - b, 4) & _MASK[4])
                    break
                k += 1
        else:
            raise Halt(f"external call to '{name}' is not one of {self._EXTERNAL}"
                       f" — unresolved, not guessed")
        ret = self.load(self.reg["rsp"], 8)      # pop the return address the
        self.reg["rsp"] += 8                     # caller's own call already pushed
        return ret

    # ── the ALU, reached only through ⊞ and ◻ ─────────────────────────────
    def alu(self, op, fields):
        if op in ("nop", "endbr64", "endbr32"):
            return
        if op == "lea":
            addr, _ = self.ea(fields[1])
            self.write(fields[0], addr)
            return
        if op in ("cdq", "cltd"):
            self.set_reg("edx", _MASK[4] if _sign(self.get_reg("eax"), 4) < 0 else 0)
            return
        if op in ("cqo",):
            self.set_reg("rdx", _MASK[8] if _sign(self.get_reg("rax"), 8) < 0 else 0)
            return
        if op in ("cdqe", "cltq"):
            self.set_reg("rax", _sign(self.get_reg("eax"), 4) & _MASK[8])
            return
        if op in ("idiv", "div"):
            size = self.width(fields[0])
            d, _ = self.read(fields[0])
            lo = self.get_reg({8: "rax", 4: "eax", 2: "ax", 1: "al"}[size])
            hi = self.get_reg({8: "rdx", 4: "edx", 2: "dx", 1: "ah"}[size])
            if op == "idiv":
                n = _sign((hi << (size * 8)) | lo, size * 2 if size < 8 else 8)
                d = _sign(d, size)
                q = abs(n) // abs(d) * (1 if (n < 0) == (d < 0) else -1)
                r = n - q * d
            else:
                n = (hi << (size * 8)) | lo
                q, r = divmod(n, d)
            self.set_reg({8: "rax", 4: "eax", 2: "ax", 1: "al"}[size], q & _MASK[size])
            self.set_reg({8: "rdx", 4: "edx", 2: "dx", 1: "ah"}[size], r & _MASK[size])
            return

        if op in _SIMD:
            self.simd(op, fields)
            return

        size = self.width(fields[0])
        a, _ = self.read(fields[0], size)
        if op in ("not", "neg", "inc", "dec"):
            r = {"not": ~a, "neg": -a, "inc": a + 1, "dec": a - 1}[op]
            self.write(fields[0], r & _MASK[size])
            if op != "not":
                self.set_flags(r & _MASK[size], 0, size)
            return
        if op == "imul" and len(fields) == 1:
            r = _sign(self.get_reg({8: "rax", 4: "eax"}[size]), size) * _sign(a, size)
            self.set_reg({8: "rax", 4: "eax"}[size], r & _MASK[size])
            return
        if op == "imul" and len(fields) == 3:
            # three-operand imul multiplies the two sources, not the
            # destination: dst may hold anything at all when it is reached.
            x, _ = self.read(fields[1], size)
            y, _ = self.read(fields[2], size)
            r = _sign(x, size) * _sign(y, size)
            self.write(fields[0], r & _MASK[size])
            self.set_flags(r & _MASK[size], 0, size)
            return
        b, _ = self.read(fields[-1], size)
        r = {
            "add": a + b, "sub": a - b, "adc": a + b, "sbb": a - b,
            "and": a & b, "or": a | b, "xor": a ^ b,
            "imul": _sign(a, size) * _sign(b, size),
            "shl": a << (b & 63), "sal": a << (b & 63),
            "shr": (a & _MASK[size]) >> (b & 63),
            "sar": _sign(a, size) >> (b & 63),
        }[op]
        self.write(fields[0], r & _MASK[size])
        self.set_flags(r & _MASK[size], 0, size)

    # ── the vector unit, likewise reached only through ⊞ and ◻ ────────────
    def simd(self, op, fields):
        dst = fields[0]
        if op in ("movdqa", "movdqu", "movaps", "movups"):
            v, _ = self.read(fields[1], 16)
            self.write(dst, v & _MASK[16])
            return
        if op in ("movd", "movq"):
            w = 4 if op == "movd" else 8
            src_is_x = fields[1][0] == "r" and fields[1][2:].startswith("xmm")
            v, _ = self.read(fields[1], w)
            if dst[0] == "r" and dst[2:].startswith("xmm"):
                self.write(dst, v & _MASK[w])          # zero-extends into xmm
            else:
                self.write(dst, v & _MASK[w] if src_is_x else v)
            return
        if op == "psrldq":                              # whole-register byte shift
            a, _ = self.read(dst, 16)
            n = int(fields[1][2:], 16)
            self.write(dst, (a >> (n * 8)) & _MASK[16])
            return
        if op in ("psrlq", "psllq"):
            a, _ = self.read(dst, 16)
            n = int(fields[1][2:], 16) if fields[1][0] == "i" else self.read(fields[1])[0]
            ls = _lanes(a, 8, 2)
            ls = [(x >> n) if op == "psrlq" else (x << n) for x in ls]
            self.write(dst, _pack(ls, 8))
            return
        if op == "pshufd":
            a, _ = self.read(fields[1], 16)
            sel = int(fields[2][2:], 16)
            ls = _lanes(a, 4, 4)
            self.write(dst, _pack([ls[(sel >> (2 * k)) & 3] for k in range(4)], 4))
            return
        if op == "punpcklqdq":
            a, _ = self.read(dst, 16)
            b, _ = self.read(fields[1], 16)
            self.write(dst, (a & _MASK[8]) | ((b & _MASK[8]) << 64))
            return
        if op == "punpckldq":
            a, _ = self.read(dst, 16)
            b, _ = self.read(fields[1], 16)
            x, y = _lanes(a, 4, 4), _lanes(b, 4, 4)
            self.write(dst, _pack([x[0], y[0], x[1], y[1]], 4))
            return
        if op == "pmuludq":                             # even 32-bit lanes → 64
            a, _ = self.read(dst, 16)
            b, _ = self.read(fields[1], 16)
            x, y = _lanes(a, 4, 4), _lanes(b, 4, 4)
            self.write(dst, _pack([x[0] * y[0], x[2] * y[2]], 8))
            return
        a, _ = self.read(dst, 16)
        b, _ = self.read(fields[1], 16)
        if op == "pxor":
            self.write(dst, a ^ b)
        elif op == "pand":
            self.write(dst, a & b)
        elif op == "por":
            self.write(dst, a | b)
        else:                                           # lane-wise add/sub/mul
            w = {"b": 1, "w": 2, "d": 4, "q": 8}[op[-1]]
            n = 16 // w
            x, y = _lanes(a, w, n), _lanes(b, w, n)
            f = {"padd": lambda p, q: p + q, "psub": lambda p, q: p - q,
                 "pmull": lambda p, q: p * q}[op[:-1]]
            self.write(dst, _pack([f(p, q) for p, q in zip(x, y)], w))

    # ── one step, dispatched on the glyph ─────────────────────────────────
    def step(self, addr):
        self.reg["rip"] = self.next_of.get(addr, 0)
        for glyph, f in self.code[addr]:
            if glyph == "∋":
                continue
            if glyph == "⊣":
                if f[0] == "leave":
                    self.reg["rsp"] = self.reg["rbp"]
                    self.reg["rbp"] = self.load(self.reg["rsp"], 8)
                    self.reg["rsp"] += 8
                    continue
                ret = self.load(self.reg["rsp"], 8)
                self.reg["rsp"] += 8
                return ret
            if glyph == "⊤":
                size = self.width(f[1])
                a, _ = self.read(f[1], size)
                b, _ = self.read(f[2], size)
                self.set_flags(a, b, size, f[0])
            elif glyph == "∈":
                if self.cc(f[0]):
                    return int(f[1][2:], 16)
            elif glyph == "≺":
                return int(f[1][2:], 16)
            elif glyph == "⊙":
                if f[0] == "syscall":
                    self.do_syscall()
                    continue
                if f[0] == "external":
                    return self.do_external(f[1])
                # ⊙ is both indirect forms. A call still has to leave its
                # return address on the stack; only a jmp does not.
                tgt, _ = self.read(f[1])
                if f[0] == "call":
                    self.reg["rsp"] -= 8
                    self.store(self.reg["rsp"], self.next_of[addr], 8)
                return tgt
            elif glyph == "≻":
                self.reg["rsp"] -= 8
                self.store(self.reg["rsp"], self.next_of[addr], 8)
                return int(f[1][2:], 16)
            elif glyph in ("⋈", "◻"):
                op = f[0]
                if op == "push":
                    v, _ = self.read(f[1])
                    self.reg["rsp"] -= 8
                    self.store(self.reg["rsp"], v, 8)
                elif op == "pop":
                    self.write(f[1], self.load(self.reg["rsp"], 8))
                    self.reg["rsp"] += 8
                elif op == "leave":
                    self.reg["rsp"] = self.reg["rbp"]
                    self.reg["rbp"] = self.load(self.reg["rsp"], 8)
                    self.reg["rsp"] += 8
                elif op == "xchg":
                    x, _ = self.read(f[1]); y, _ = self.read(f[2])
                    self.write(f[1], y); self.write(f[2], x)
                elif op in ("mov", "movabs"):
                    v, _ = self.read(f[2], self.width(f[1]))
                    self.write(f[1], v & _MASK[self.width(f[1])])
                elif op == "movzx":
                    v, _ = self.read(f[2])
                    self.write(f[1], v & _MASK[self.width(f[2])])
                elif op in ("movsx", "movsxd"):
                    v, _ = self.read(f[2])
                    self.write(f[1], _sign(v, self.width(f[2])) & _MASK[self.width(f[1])])
                else:
                    self.alu(op, f[1:])
            elif glyph == "⊥":
                if f[1] == "set":
                    self.write(f[2], 1 if self.cc(f[0]) else 0)
                elif self.cc(f[0]):
                    v, _ = self.read(f[3], self.width(f[2]))
                    self.write(f[2], v)
            elif glyph == "⊞":
                self.alu(f[0], f[1:])
        return self.next_of.get(addr)

    def call(self, addr, *args, limit=50_000_000):
        """Run one function to its ⊣, System V integer arguments."""
        for name, v in zip(("rdi", "rsi", "rdx", "rcx", "r8", "r9"), args):
            self.set_reg(name, v & _MASK[8])
        sentinel = 0xDEAD0000
        self.reg["rsp"] -= 8
        self.store(self.reg["rsp"], sentinel, 8)
        pc = addr
        self.steps = 0
        while pc != sentinel:
            if pc not in self.code:
                raise Halt(f"no instruction at 0x{pc:x}")
            pc = self.step(pc)
            self.steps += 1
            if pc is None or self.steps > limit:
                raise Halt(f"ran off the end after {self.steps} steps")
        return _sign(self.get_reg("eax"), 4)
