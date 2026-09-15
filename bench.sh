#!/usr/bin/env bash
# Bake a batch of values as their IMASM numerals into one sealed binary, compile,
# and run to completion. The binary prints the table only when the whole batch is
# done: sealed, no peeking. usage: ./bench.sh <file of "label decimal" lines>
set -euo pipefail
[ $# -eq 1 ] || { echo "usage: $0 <file of 'label decimal' lines>" >&2; exit 1; }
cd "$(dirname "$0")"
cargo build --release --bin vox >/dev/null 2>&1
WORDS=""; LABELS=""
while read -r tot N; do
  [ -z "$N" ] && continue
  w="$(./target/release/vox numeral "$N")"
  WORDS="$WORDS $w"; LABELS="$LABELS $tot"
done < "$1"
touch src/bin/factor_bench.rs
BENCH_WORDS="$WORDS" BENCH_LABELS="$LABELS" cargo build --release --bin factor_bench >/dev/null 2>&1
exec ./target/release/factor_bench
