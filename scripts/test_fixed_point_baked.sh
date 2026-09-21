#!/usr/bin/env bash
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
case_file="$repo/tests/baked/fixed_point_n15.imasm"
target_dir="$repo/target/baked-fixed-point-n15"

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

n="$(printf '%s\n' "$output" | awk '$1 == "n" {print $2}')"
p="$(printf '%s\n' "$output" | awk '$1 == "p" {print $2}')"
q="$(printf '%s\n' "$output" | awk '$1 == "q" {print $2}')"

[[ "$n" == "15" ]]
if ! { [[ "$p" == "3" && "$q" == "5" ]] || [[ "$p" == "5" && "$q" == "3" ]]; }; then
  echo "unexpected baked factor readout: p=$p q=$q" >&2
  exit 1
fi
