#!/usr/bin/env bash
# Bake N in as its IMASM numeral, compile, and run once — no runtime input.
# The decimal is consumed here by `vox numeral` (the encode-before-compile step);
# the numeral word it prints is baked into factor_one at compile, so the run is
# pure execution of a word that already contains N.
# usage: ./factor_one.sh <decimal N>
set -euo pipefail
[ $# -eq 1 ] || { echo "usage: $0 <decimal N>" >&2; exit 1; }
cd "$(dirname "$0")"
# Ensure vox (the encoder) is built.
cargo build --release --bin vox >/dev/null 2>&1
WORD="$(./target/release/vox numeral "$1")"
# FORCE a rebuild of just this bin so the new numeral is compiled in.
touch src/bin/factor_one.rs
FACTOR_N_WORD="$WORD" cargo build --release --bin factor_one >/dev/null 2>&1
exec ./target/release/factor_one
