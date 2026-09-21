#!/usr/bin/env bash
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run_case() {
  local label="$1"
  local case_file="$2"
  local expected_n="$3"
  local expected_p="$4"
  local expected_q="$5"
  local target_dir="$repo/target/baked-fixed-point-$label"

  rm -rf "$target_dir"
  VOX_BAKED_INPUT_FILE="$case_file" \
  CARGO_TARGET_DIR="$target_dir" \
  cargo build \
    --manifest-path "$repo/Cargo.toml" \
    --target x86_64-unknown-linux-gnu \
    --release \
    --bin fixed_point_baked

  local binary="$target_dir/x86_64-unknown-linux-gnu/release/fixed_point_baked"
  local output
  output="$(cd /tmp && env -u VOX_BAKED_INPUT_FILE "$binary")"
  printf '%s\n' "$output"

  local n p q
  n="$(printf '%s\n' "$output" | awk '$1 == "n" {print $2}')"
  p="$(printf '%s\n' "$output" | awk '$1 == "p" {print $2}')"
  q="$(printf '%s\n' "$output" | awk '$1 == "q" {print $2}')"

  [[ "$n" == "$expected_n" ]]
  if ! { [[ "$p" == "$expected_p" && "$q" == "$expected_q" ]] || [[ "$p" == "$expected_q" && "$q" == "$expected_p" ]]; }; then
    echo "unexpected baked factor readout for $label: p=$p q=$q" >&2
    exit 1
  fi
}

# N=15 closes inside the repaired boundary at winding 4.
run_case n15 "$repo/tests/baked/fixed_point_n15.word" 15 3 5

# N=21 has ord_21(2)=6, so the compiled artifact must pass through TANCH and
# execute two scheduler re-entry cycles before the existing Hadamard descent can
# expose 3 and 7. The executable still receives no runtime input or re-entry count.
run_case n21 "$repo/tests/baked/fixed_point_n21.word" 21 3 7
