# Hard-Target Descent Bottleneck

The 224-bit timeout came from dispatching the hard target to the bounded resident Lehman descent. `run_integrated.py` calls `build_case(n, 'dialectic-lean')`; that binary runs `ResidentDialectic`, not the source-carrier MPQS relation producer that returned the verified 224-bit factors in the prior session.

The resident object starts with a 64-cell short frontier, crosses a 4,032-cell extended Fermat span, then enters the Lehman lattice. Lehman tests 64 consecutive `a` values per multiplier `k` and advances `k` once per unsuccessful descent. At the 90-second timeout, the recorded run had 632,078 descents, 40,456,960 cells, 13,905,916 full-width square tests, and 26,551,044 low-residue rejections. The last progress sample was at 85 seconds. This is a multiplier walk, not MPQS relation closure.

Vox disassembled the exact timeout executable to `byte_fresh_balanced_224.disasm` and `byte_fresh_balanced_224.wiring.tsv`. The binary rewirer emitted `byte_fresh_balanced_224.rewired.imasm` and a passing report: all 355,966 bytes were decoded, with no opaque bytes or unresolved transfers. The rewritten graph contains 88,285 instructions and 4,138 functions.

A bounded Vox native profile sampled the same executable for about ten seconds. The instruction samples place mapped hot loops in `morphism_factor::fold` and `morphism_factor::unfold`, which convert the full-width bit tapes to and from limbs around arithmetic. 291 of 974 samples were outside the main ELF image. Hardware-counter collection was denied by the host, so those samples do not identify the external frames or provide cycle counts.

The route-selection defect is in the experiment harness: the general hard-target campaign selected `dialectic-lean` directly and omitted the successful source-carrier/MPQS arm. To close these targets on the successful path, dispatch must preserve the shared resident relation while admitting that relation producer, rather than repeatedly increasing the Lehman multiplier.
