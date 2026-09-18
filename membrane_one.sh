#!/usr/bin/env bash
# Bake N in as its IMASM numeral, compile, and run once — same shape as factor_one.sh.
# The decimal is consumed here by `vox numeral` (the encode-before-compile step);
# the numeral word it prints is baked into `membrane` at compile, so the run is
# pure execution of a word that already contains N.
# usage: ./membrane_one.sh <decimal N>
set -euo pipefail
[ $# -eq 1 ] || { echo "usage: $0 <decimal N>" >&2; exit 1; }
cd "$(dirname "$0")"
cargo build --release --bin vox >/dev/null 2>&1
WORD="$(./target/release/vox numeral "$1")"
# FORCE a rebuild of just this bin so the new numeral is compiled in.
touch src/bin/membrane.rs
MEMBRANE_WORDS="$WORD" cargo build --release --bin membrane >/dev/null 2>&1
exec ./target/release/membrane
