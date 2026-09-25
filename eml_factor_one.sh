#!/usr/bin/env bash
# Bake N's IMASM numeral into the EML-first full carrier and execute it once.
set -euo pipefail
[ "$#" -ge 1 ] && [ "$#" -le 2 ] || { echo "usage: $0 <decimal N> [decimal phase-base]" >&2; exit 1; }
cd "$(dirname "$0")"
N="$1"
PHASE_BASE="${2:-2}"
[[ "$N" =~ ^[0-9]+$ ]] || { echo "N must be a nonnegative decimal integer" >&2; exit 1; }
[[ "$PHASE_BASE" =~ ^[0-9]+$ ]] || { echo "phase base must be a nonnegative decimal integer" >&2; exit 1; }
RUSTFLAGS='-C target-cpu=native -D warnings' cargo build --release --bin vox >/dev/null 2>&1
WORD="$(./target/release/vox numeral "$N")"
BASE_WORD="$(./target/release/vox numeral "$PHASE_BASE")"
IDENTITY="$(printf '%s%s' "$WORD" "$BASE_WORD" | sha256sum | cut -c1-24)"
DEST="membranes/baked_eml/factor_${IDENTITY}"
mkdir -p "$DEST"
printf '%s\n%s\n' "$WORD" "$BASE_WORD" > "$DEST/input.imasm"
RUSTFLAGS='-C target-cpu=native -D warnings' \
    VOX_BAKED_INPUT_FILE="$DEST/input.imasm" cargo build --release --bin factor_eml_one >/dev/null 2>&1
cp target/release/factor_eml_one "$DEST/factor_eml_one"
chmod +x "$DEST/factor_eml_one"
exec "$DEST/factor_eml_one"
