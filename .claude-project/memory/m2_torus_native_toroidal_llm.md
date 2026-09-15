---
name: m2_torus_native_toroidal_llm
description: m2_torus — the trunk trains now; candle's fused rms_norm was dropping the gradient, and the corpus is the ob3ect opcode sequences
metadata:
  type: project
---

A Grammar-native toroidal LM in `daydr33m/m2_torus`, trained as a masked-ring
IMASM denoiser. As of 2026-08-13 it works: **held-out denoise 60.4% against 8.3%
chance, 44 of 64 filled val rings Closed**, loss 0.87 (ln 12 = 2.485).

**The trunk was frozen for every run before that.** `candle_nn::rms_norm` carries
no gradient back to what feeds it. Every layer parameter sits behind a norm —
fork and fuse behind `norm_attn`, both ENGAGR arms and `ff_fuse` behind
`norm_ff` — so all 48 held their initial value to eight decimals while the
residual `x + f(norm(x))` still carried gradient to the embedding through its
`x` term. Loss fell, the trunk looked trained, and six layers of random
projection sat in the middle. Every number before the fix (15.1%, 17.1%, 21.3%,
23.3%) was an embedding and a tied head doing unigram statistics. The fix is an
explicit `Rms` module in `model.rs`: `x * rsqrt(mean(x²)+eps) * weight`.

Diagnosis that found it: with `--gate-lambda 0` the printed gate never moved at
all, because `gate_penalty` — which measures ‖WᵀW−I‖² on fork and fuse, NOT on
the gate — was the only gradient those two ever got. Printing ‖W‖² per parameter
showed embed moving and every layer weight constant. If a metric is bit-identical
across steps, the parameter is not being trained; check before reading the curve.

**The corpus is the ob3ect opcode sequences.** `ob3ect/digital/*/​*_ob3ect.json`
carries `phases.phase_4.steps`, an ordered opcode list per ob3ect; the twelve
opcodes are the twelve marks (VINIT ⊢, TANCH ⊣, AFWD >, AREV <, CLINK ⋈,
EVALT ⊤, FSPLIT ∈, FFUSE ∋, IMSCRIB ⊙, EVALF ⊥, ENGAGR ⊞, IFIX ⊡). 1703
sequences, 1277 distinct, lengths 3..114 → `ob3ect_words.txt`. The old
`imasm_words.txt` (1502 rings of unrecorded provenance) was the ceiling: it
plateaued at 17% no matter the steps. Do NOT look for words in the ob3ect SVGs —
only 26 of 1340 carry one, and `grounded_tuple` in the JSON is a 12-VALUE
Shavian tuple, a different object from the word.

`train.rs` takes every knob on the command line now (`--lr --ring --batch
--mask-frac --gate-lambda --seed --d-model --layers --heads --ff-mult --sigma0
--causal --val-frac --log-every`), and accepts variable-length rings: longer
than the window wraps, shorter pads with the padding excluded from masking and
from the loss. Best so far: `--lr 9e-4 --ring 16`, 10k steps.

Open: the gate penalty now RISES after step ~2600 (0.0333 → 0.0534) — the trunk
trades μ∘δ=id deviation for task performance once it can actually learn. That
exchange rate is what `--gate-lambda` sets. See [[the_forty_nine]].
