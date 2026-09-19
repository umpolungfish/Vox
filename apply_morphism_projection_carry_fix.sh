#!/usr/bin/env bash
set -euo pipefail

f=src/morphism_factor.rs

if grep -q 'fn dispatch_projection_holds' "$f"; then
  echo 'carry-aware dispatch projection already present'
elif grep -q 'if walk_work != tower_work {' "$f"; then
  git apply --check morphism_projection_carry_fix.patch
  git apply morphism_projection_carry_fix.patch
  echo 'applied carry-aware execution-projection fix'
else
  echo 'expected audit block not found; refusing to edit an unknown source state' >&2
  exit 2
fi

echo
echo 'running focused morphism tests...'
cargo test --release morphism_factor::tests::instant_extractor_decomposes_and_factors --lib
cargo test --release morphism_factor::tests::frontier_arm_clears_a_balanced_semiprime_fast --lib
cargo test --release morphism_factor::tests::changed_work_is_rejected_even_when_a_type_is_factoring_complete --lib
