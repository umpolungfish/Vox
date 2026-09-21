#!/usr/bin/env bash
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
case_file="$repo/tests/baked/fixed_point_semiprime_31.word"
target_dir="$repo/target/baked-fixed-point-semiprime-31"

rm -rf "$target_dir"
VOX_BAKED_INPUT_FILE="$case_file" \
CARGO_TARGET_DIR="$target_dir" \
cargo build \
  --manifest-path "$repo/Cargo.toml" \
  --target x86_64-unknown-linux-gnu \
  --release \
  --bin fixed_point_baked

binary="$target_dir/x86_64-unknown-linux-gnu/release/fixed_point_baked"
output="$(cd /tmp && env -u VOX_BAKED_INPUT_FILE "$binary")"
printf '%s\n' "$output"

if printf '%s\n' "$output" | grep -Eq '^(n|winding|period) '; then
  echo "compiled membrane leaked a consumed intermediate" >&2
  exit 1
fi

p="$(printf '%s\n' "$output" | awk '$1 == "p" {print $2}')"
q="$(printf '%s\n' "$output" | awk '$1 == "q" {print $2}')"

if ! { [[ "$p" == "54559" && "$q" == "37243" ]] || [[ "$p" == "37243" && "$q" == "54559" ]]; }; then
  echo "unexpected 31-bit semiprime factor readout: p=$p q=$q" >&2
  exit 1
fi
