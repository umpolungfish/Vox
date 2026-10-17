#!/usr/bin/env bash
# Compile a self-contained factor membrane with N and its evaluation-frame
# width imscribed into the executable. The resulting binary takes no input.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
[ "$#" -ge 1 ] && [ "$#" -le 2 ] || {
  echo "usage: ./frame_factor_build.sh <decimal-N> [frame-width]" >&2
  exit 1
}
cd "$(dirname "$0")"
N="$1"
WIDTH="${2:-2}"
[[ "$N" =~ ^[0-9]+$ ]] || { echo "N must be a nonnegative decimal integer" >&2; exit 1; }
[[ "$WIDTH" =~ ^[0-9]+$ ]] && [ "$WIDTH" -ge 2 ] || {
  echo "frame width must be a decimal integer of at least 2" >&2
  exit 1
}
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null 2>&1
WORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$N")"
IDENTITY="$(printf '%s:%s' "$WORD" "$WIDTH" | sha256sum | cut -c1-20)"
mkdir -p membranes
touch src/bin/frame_factor_one.rs
FRAME_FACTOR_N_WORD="$WORD" FRAME_FACTOR_WIDTH="$WIDTH" \
  cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin frame_factor_one >/dev/null 2>&1
require_static_membrane "target/x86_64-unknown-linux-musl/release/frame_factor_one"
cp target/x86_64-unknown-linux-musl/release/frame_factor_one "membranes/frame_factor_${IDENTITY}"
chmod +x "membranes/frame_factor_${IDENTITY}"
printf 'compiled membrane: %s\n' "membranes/frame_factor_${IDENTITY}"
