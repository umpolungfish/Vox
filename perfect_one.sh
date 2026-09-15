#!/usr/bin/env bash
# Bake a value as its IMASM numeral into a depth-n perfect membrane, compile, and
# run once. The transform rides the matched delta/mu circuit; the run takes no
# input. usage: ./perfect_one.sh <decimal value> [depth]
set -euo pipefail
[ $# -ge 1 ] || { echo "usage: $0 <decimal value> [depth]" >&2; exit 1; }
cd "$(dirname "$0")"
cargo build --release --bin vox >/dev/null 2>&1
WORD="$(./target/release/vox numeral "$1")"
DEPTH="${2:-2}"
touch src/bin/perfect_one.rs
PERFECT_N_WORD="$WORD" PERFECT_DEPTH="$DEPTH" cargo build --release --bin perfect_one >/dev/null 2>&1
exec ./target/release/perfect_one
