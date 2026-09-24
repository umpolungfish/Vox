#!/usr/bin/env bash
# Compile a self-contained factor membrane with N and its evaluation-frame
# width imscribed into the executable. The resulting binary takes no input.
set -euo pipefail
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
cargo build --release --bin vox >/dev/null 2>&1
WORD="$(./target/release/vox numeral "$N")"
IDENTITY="$(printf '%s:%s' "$WORD" "$WIDTH" | sha256sum | cut -c1-20)"
mkdir -p membranes
touch src/bin/frame_factor_one.rs
FRAME_FACTOR_N_WORD="$WORD" FRAME_FACTOR_WIDTH="$WIDTH" \
  cargo build --release --bin frame_factor_one >/dev/null 2>&1
cp target/release/frame_factor_one "membranes/frame_factor_${IDENTITY}"
chmod +x "membranes/frame_factor_${IDENTITY}"
printf 'compiled membrane: %s\n' "membranes/frame_factor_${IDENTITY}"
