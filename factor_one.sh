#!/usr/bin/env bash
# Bake N in as its IMASM numeral, compile, and emit its static executable — no runtime input.
# The decimal is consumed here by `vox numeral` (the encode-before-compile step);
# the numeral word it prints is baked into factor_one at compile, so the run is
# pure execution of a word that already contains N.
# usage: ./factor_one.sh <decimal N> [carrier word]
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
[ $# -ge 1 ] && [ $# -le 2 ] || { echo "usage: $0 <decimal N> [carrier word]" >&2; exit 1; }
cd "$(dirname "$0")"
# Ensure vox (the encoder) is built.
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null 2>&1
WORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$1")"
# FORCE a rebuild of just this bin so the new numeral is compiled in.
touch src/bin/factor_one.rs
FACTOR_N_WORD="$WORD" FACTOR_CARRIER_WORD="${2-}" cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin factor_one >/dev/null 2>&1
# The default output is a verified static executable.
# MEMBRANE_RUN=1 also executes it.
finish_static_membrane "$PWD"/target/x86_64-unknown-linux-musl/release/factor_one
