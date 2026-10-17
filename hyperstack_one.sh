#!/usr/bin/env bash
# Bake N as its IMASM numeral AND the carrier stack into a standalone membrane
# binary, then EMIT that binary at a named path. It is not run here: the built
# artifact already contains N and the stack, takes no input, and is executed
# separately. usage: ./hyperstack_one.sh <decimal N> [carrier types...]
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
[ $# -ge 1 ] || { echo "usage: $0 <decimal N> [carrier types...]" >&2; exit 1; }
cd "$(dirname "$0")"
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null 2>&1
N="$1"; shift
TYPES="${*:-phase shor fib}"
WORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$N")"
FACTOR_N_WORD="$WORD" HYPERSTACK_TYPES="$TYPES" cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin hyperstack_one >/dev/null 2>&1
mkdir -p membranes
SLUG="$(printf '%s' "$TYPES" | tr ' ' '-')"
OUT="membranes/factor_${N}_${SLUG}"
require_static_membrane "./target/x86_64-unknown-linux-musl/release/hyperstack_one"
cp -f ./target/x86_64-unknown-linux-musl/release/hyperstack_one "$OUT"
echo "emitted baked membrane: $OUT"
echo "  N=$N through [$TYPES]; run it with: $(dirname "$0")/$OUT"
