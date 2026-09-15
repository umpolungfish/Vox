---
name: context_window_conductor
description: "Alibaba's published context/output lengths sit on the 2^12 conductor lattice with odd part 1 or 5; the measurement that would settle it is pending"
metadata:
  type: project
---

Noticed by Lando 2026-08-19 while setting the local model's window from Qwen's
recommendations: their two output lengths are 32768 and 81920, and a line in the
SIC material reads the index-2048 fixed subfield at degree 2^16 = 65536 with real
subfield 2^15 = 32768.

The arithmetic, computed: 32768 = 2^15 (odd part 1), 81920 = 5·2^14 = 65536+16384
= (5/4)·2^16, ratio 81920/32768 = 5/2. Also 40960 = 5·2^13, 65536 = 2^16,
131072 = 2^17, 262144 = 2^18. As multiples of the conductor 2^12 = 4096 the
quotients are 8, 10, 16, 20, 32, 64. Every odd part is 1 or 5, and 5 is one of the
four radicals of the d=2048 Stark unit discriminant, 4190205 = 3·5·409·683
(factorization verified), the level whose Gauss sum returns its own radical.

Held against it: round decimal multiples carry 5s for free, and Alibaba's own
Qwen2.5-1M length 1,010,000 = 2^4·5^4·101 is not a multiple of 4096 at all
(quotient 246.58). The decimal numbers in the stack trace to RoPE's base 10000;
the binary ones to the address width.

**The measurement, still to run.** Take `max_position_embeddings` from every
`config.json` under ~/.modelz and ~/models rather than from recalled figures, take
the odd part of each, and compare the distribution across vendors. Odd parts
staying in {1,5} for Alibaba while spreading over {1,3,5,7,...} elsewhere is the
conductor doing work; everyone in {1,5} is decimal rounding on a binary substrate.

See [[landscape]] (SIC: conductor 4096, degree 2^27/Q), [[d2048_bypass_is_the_answer]].
