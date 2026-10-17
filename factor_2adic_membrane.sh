#!/usr/bin/env bash
# Convert N to its IMASM numeral, bake it into a contained membrane binary,
# and print the path of that binary. The produced binary takes no input.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
cd "$(dirname "$0")"

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'USAGE'
Usage: ./factor_2adic_membrane.sh <decimal-N> [output-binary] [phase-base] [lift-radix]

Convert N, phase base, and lift radix to IMASM numerals, bake them into a native executable, and
print the executable path. Both defaults are 2. The phase base drives winding;
the lift radix drives the prefix modulus radix^k. Lift into IMASM to inspect or
execute the contained module.

Example:
  ./factor_2adic_membrane.sh 105
  vox imasm ./membranes/factor_2adic_<hash>
  vox run ./membranes/factor_2adic_<hash>.imasm
  ./factor_2adic_membrane.sh 1000036000099 membranes/case 7 10
USAGE
  exit 0
fi

N="${1:?usage: ./factor_2adic_membrane.sh <N> [output-binary]}"
if [[ ! "$N" =~ ^[0-9]+$ ]]; then
  echo "N must be a nonnegative decimal integer" >&2
  exit 2
fi
BASE="${3:-2}"
if [[ ! "$BASE" =~ ^[0-9]+$ ]]; then
  echo "base must be a nonnegative decimal integer" >&2
  exit 2
fi
RADIX="${4:-2}"
if [[ ! "$RADIX" =~ ^[0-9]+$ ]]; then
  echo "lift radix must be a nonnegative decimal integer" >&2
  exit 2
fi

cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null
N_WORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$N")"
BASE_WORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$BASE")"
RADIX_WORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$RADIX")"
if [[ -z "$N_WORD" || -z "$BASE_WORD" || -z "$RADIX_WORD" ]]; then
  echo "vox numeral returned an empty word" >&2
  exit 2
fi
./target/x86_64-unknown-linux-musl/release/vox coprime "$BASE" "$N" >/dev/null
FINGERPRINT="$(printf '%s\n%s\n%s' "$N_WORD" "$BASE_WORD" "$RADIX_WORD" | sha256sum | cut -d' ' -f1)"
OUTPUT="${2:-membranes/factor_2adic_${FINGERPRINT}}"
if [[ -e "$OUTPUT" ]]; then
  echo "output already exists: $OUTPUT" >&2
  exit 2
fi

# A value-specific target directory keeps Cargo's compile-time environment
# attached to this membrane when another N is baked later.
BUILD_DIR="target/factor_2adic_membranes/${FINGERPRINT}"
mkdir -p "$BUILD_DIR"
printf '%s\n%s\n%s\nunit\n' "$N_WORD" "$BASE_WORD" "$RADIX_WORD" > "$BUILD_DIR/baked_inputs.txt"
VOX_BAKED_INPUT_FILE="$BUILD_DIR/baked_inputs.txt" \
cargo build --release --bin factor_2adic --target x86_64-unknown-linux-musl --target-dir "$BUILD_DIR"

mkdir -p "$(dirname "$OUTPUT")"
require_static_membrane "$BUILD_DIR/x86_64-unknown-linux-musl/release/factor_2adic"
cp "$BUILD_DIR/x86_64-unknown-linux-musl/release/factor_2adic" "$OUTPUT"
chmod +x "$OUTPUT"
./target/x86_64-unknown-linux-musl/release/vox imasm "$OUTPUT" >/dev/null 2>&1
printf '%s\n' "$OUTPUT"
