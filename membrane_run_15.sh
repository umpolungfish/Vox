#!/usr/bin/env bash
# PRE-ENCODED IMASM numerals baked in. No runtime encoding, no runtime input.
set -euo pipefail
cd "$(dirname "$0")"
A='⊢≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣'
N='⊢≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋⊙⊡⊣'
Q='⊢≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋⊙⊡⊣'
echo "a-word: $A"
echo "N-word: ${#N} glyphs"
echo "q-word: $Q"
export MEMBRANE_WORDS="$A $N $Q"
touch src/bin/multi_quantum_membrane.rs
cargo build --release --bin multi_quantum_membrane
exec ./target/release/multi_quantum_membrane
