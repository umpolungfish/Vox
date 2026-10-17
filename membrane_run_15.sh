#!/usr/bin/env bash
# PRE-ENCODED IMASM numerals baked in. No runtime encoding, no runtime input.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
cd "$(dirname "$0")"
A='⊢≻⋈∈⊤∋≻⋈∈⊥∋⊙⊡⊣'
N='⊢≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋≻⋈∈⊥∋⊙⊡⊣'
Q='⊢≻⋈∈⊥∋≻⋈∈⊤∋≻⋈∈⊥∋≻⋈∈⊥∋⊙⊡⊣'
echo "a-word: $A"
echo "N-word: ${#N} glyphs"
echo "q-word: $Q"
export MEMBRANE_WORDS="$A $N $Q"
touch src/bin/multi_quantum_membrane.rs
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin multi_quantum_membrane
finish_static_membrane "$PWD"/target/x86_64-unknown-linux-musl/release/multi_quantum_membrane
