#!/usr/bin/env bash
# Bake N in as its IMASM numeral AND the carrier stack, compile, run once.
# The decimal is consumed here by `vox numeral`; the numeral word and the
# carrier sequence are baked into hyperstack_one at compile, so the run is pure
# execution of a heterogeneous membrane that already contains N.
# usage: ./hyperstack_one.sh <decimal N> [carrier types...]   (default: phase shor fib)
set -euo pipefail
[ $# -ge 1 ] || { echo "usage: $0 <decimal N> [carrier types...]" >&2; exit 1; }
cd "$(dirname "$0")"
cargo build --release --bin vox >/dev/null 2>&1
N="$1"; shift
TYPES="${*:-phase shor fib}"
WORD="$(./target/release/vox numeral "$N")"
touch src/bin/hyperstack_one.rs
FACTOR_N_WORD="$WORD" HYPERSTACK_TYPES="$TYPES" cargo build --release --bin hyperstack_one >/dev/null 2>&1
exec ./target/release/hyperstack_one
