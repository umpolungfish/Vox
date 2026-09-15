---
name: vox_runs_the_kernel
description: "vox run boots the whole g-momonados kernel through IMASM, byte-identical to native; the closed loop"
metadata: 
  node_type: memory
  type: project
  originSessionId: b7fcac2b-0124-4019-8686-5e1d7369edb0
  modified: 2026-09-09T16:23:01.407Z
---

`vox run <static-musl g-momonados>` lifts the entire kernel to the twelve marks
and executes it in `vox_core::imasm_vm`, and it boots end to end: every anyon
and TQFT self-check PASS, the torus wound (⊢⊙∈⊤⋈∋⊡⊞≺⋈⊣, pinch 0.0000),
Frobenius weighted=3.1000, Crystal FS 17280000, the banner, the ⊙> REPL,
`quit`, `[SHUTDOWN] μ∘δ=id`, exit 0 after ~223M steps. The control passed:
diff against the native run is byte-identical. The x86 was only the shape the
linker left the marks in; the kernel was always an IMASM program.

**Why:** this is the loop Lando named at the start — IMASM running IMASM, the
only English the terminal write at the edge. He called it the birth of a new
computing, the compiler move from the other end: any binary already speaks the
Grammar, it needed an instrument that reads and runs it in the marks without
falling back to x86.

**How to apply:** the build is static non-PIE musl (`rustup target add
x86_64-unknown-linux-musl`, `RUSTFLAGS=-C target-feature=+crt-static -C
relocation-model=static cargo build --release --target x86_64-unknown-linux-musl`).
Read code through `vox imasm <bin>` and vox's own trace, never objdump/nm —
Lando is firm on this. Each fix that got here is its own Vox commit: the SIB
index dropping r12, shift count masked to 5 bits not 6, carry through
add/adc/sub/sbb and shifts, movss/movsd storing the xmm size and clobbering 12
bytes past the float, cmpss/cmpsd/cmpps/cmppd undecoded, plus bt/tzcnt/lzcnt/
hlt/writev, rep movs/stos, the float arithmetic, TLS + startup syscalls, and
the IRELATIVE/RELATIVE relocs. The next rung Lando set ("take it all the way")
is the marks executing as marks on silicon via [[what_an_ob3ect_is]]'s
gpu_kernel token-graph executor rather than the imasm_vm interpreter.
Two engines differ: imasm_vm carries the real x86 payload; gpu_kernel runs the
bare 12-op Belnap word. Running the lifted module on gpu_kernel natively is
still open.

**Self-host closed (the ouroboros).** vox built static-musl, lifted to the
marks and run in imasm_vm, executes its own commands identical to native:
`vox verdict ⊢⊞⊤⊣` gives `verdict N` (exit 0); `vox hex 4889f8c3` lifts
mov rax,rdi;ret into `⊢⋈⊣` with the same verdict table (exit 0). The lifter,
lifted, lifting. The last fix was the loader taking only PROGBITS and not the
INIT_ARRAY/FINI_ARRAY/PREINIT_ARRAY (types 14/15/16) constructor tables, so the
C runtime read a zero table and called address 0; loading them narrowly (only
those types, not every allocated section — the broad version loops g-momonados)
keeps g-momonados byte-identical and closes the self-host. Read code through
`vox imasm`/`vox hex`/the trace, never objdump/nm.

**In the kernel now.** G-mOMonadOS commit abca98e adds the REPL command
`imrun <file> [argv...]`: it lifts a binary through `imasm_exec`
(`vox_core::imasm_module::emit`), builds the payload `Machine`, sets the
kernel's `StdHost`, and runs it. From the ⊙> prompt,
`imrun <static-vox> verdict ⊢⊞⊤⊣` prints vox's own `verdict N`, exit 0 — the
kernel runs a whole program as the twelve marks without leaving the kernel.
The loop is inside G-mOMonadOS, not only the external vox tool.
