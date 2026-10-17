#!/usr/bin/env bash
# Bake N's IMASM numeral into the EML-first full carrier and emit its static executable.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
[ "$#" -ge 1 ] && [ "$#" -le 2 ] || { echo "usage: $0 <decimal N> [decimal phase-base]" >&2; exit 1; }
cd "$(dirname "$0")"
N="$1"
PHASE_BASE="${2:-2}"
[[ "$N" =~ ^[0-9]+$ ]] || { echo "N must be a nonnegative decimal integer" >&2; exit 1; }
[[ "$PHASE_BASE" =~ ^[0-9]+$ ]] || { echo "phase base must be a nonnegative decimal integer" >&2; exit 1; }
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null 2>&1
WORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$N")"
BASE_WORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$PHASE_BASE")"
IDENTITY="$(printf '%s%s' "$WORD" "$BASE_WORD" | sha256sum | cut -c1-24)"
DEST="membranes/baked_eml/factor_${IDENTITY}"
mkdir -p "$DEST"
printf '%s\n%s\n' "$WORD" "$BASE_WORD" > "$DEST/input.imasm"
VOX_BAKED_INPUT_FILE="$DEST/input.imasm" cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin factor_eml_one >/dev/null 2>&1
require_static_membrane "target/x86_64-unknown-linux-musl/release/factor_eml_one"
cp target/x86_64-unknown-linux-musl/release/factor_eml_one "$DEST/factor_eml_one"
chmod +x "$DEST/factor_eml_one"
require_static_membrane "$DEST/factor_eml_one"
finish_static_membrane "$PWD/$DEST/factor_eml_one"
