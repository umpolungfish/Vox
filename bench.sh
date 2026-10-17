#!/usr/bin/env bash
# Bake a batch of values as their IMASM numerals into one sealed binary, compile,
# and emit its static executable. The binary prints the table only when the whole batch is
# done: sealed, no peeking. usage: ./bench.sh <file of "label decimal" lines>
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
[ $# -eq 1 ] || { echo "usage: $0 <file of 'label decimal' lines>" >&2; exit 1; }
cd "$(dirname "$0")"
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null 2>&1
WORDS=""; LABELS=""
while read -r tot N; do
  [ -z "$N" ] && continue
  w="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$N")"
  WORDS="$WORDS $w"; LABELS="$LABELS $tot"
done < "$1"
touch src/bin/factor_bench.rs
BENCH_WORDS="$WORDS" BENCH_LABELS="$LABELS" cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin factor_bench >/dev/null 2>&1
# The default output is a verified static executable.
# MEMBRANE_RUN=1 also executes it.
finish_static_membrane "$PWD"/target/x86_64-unknown-linux-musl/release/factor_bench
