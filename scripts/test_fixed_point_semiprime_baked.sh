#!/usr/bin/env bash
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run_case() {
  local label="$1"
  local case_file="$2"
  local expect_p="$3"
  local expect_q="$4"
  local target_dir="$repo/target/baked-fixed-point-semiprime-$label"

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

  if printf '%s\n' "$output" | grep -Eq '^(n|winding|period) '; then
    echo "compiled membrane leaked a consumed intermediate for $label" >&2
    exit 1
  fi

  local p q
  p="$(printf '%s\n' "$output" | awk '$1 == "p" {print $2}')"
  q="$(printf '%s\n' "$output" | awk '$1 == "q" {print $2}')"

  if ! { [[ "$p" == "$expect_p" && "$q" == "$expect_q" ]] || [[ "$p" == "$expect_q" && "$q" == "$expect_p" ]]; }; then
    echo "unexpected $label semiprime factor readout: p=$p q=$q" >&2
    exit 1
  fi
}

run_case 31 "$repo/tests/baked/fixed_point_semiprime_31.word" 54559 37243
run_case 40 "$repo/tests/baked/fixed_point_semiprime_40.word" 972221 976231
run_case 47 "$repo/tests/baked/fixed_point_semiprime_47.word" 8424287 13575097
run_case 56 "$repo/tests/baked/fixed_point_semiprime_56.word" 243325237 200504561
run_case 60 "$repo/tests/baked/fixed_point_semiprime_60.word" 893585183 909907849
