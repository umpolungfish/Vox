#!/usr/bin/env bash
# Bake N as its IMASM numeral into a factoring membrane, compile, and run once.
# The transformation rides the matched delta/mu circuit; the run takes no input.
# usage: ./factor_one.sh <decimal N>
set -euo pipefail
[ $# -ge 1 ] || { echo "usage: $0 <decimal N>" >&2; exit 1; }
cd "$(dirname "$0")"
cargo build --release --bin vox >/dev/null 2>&1
WORD="$(./target/release/vox numeral "$1")"
touch src/bin/factor_one.rs
FACTOR_N_WORD="$WORD" cargo build --release --bin factor_one >/dev/null 2>&1
exec ./target/release/factor_one
