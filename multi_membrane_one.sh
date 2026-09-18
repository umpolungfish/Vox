#!/usr/bin/env bash
# Bake a, N and register width as IMASM numerals, compile them in, run once.
# No runtime input: the computation is already in the word.
# usage: ./multi_membrane_one.sh <decimal N> [a] [qubits]
set -euo pipefail
cd "$(dirname "$0")"
N="${1:?usage: $0 <decimal N> [a] [qubits]}"
a="${2:-2}"; q="${3:-13}"
cargo build --release --bin vox >/dev/null 2>&1
WORD="$(./target/release/vox numeral "$a") $(./target/release/vox numeral "$N") $(./target/release/vox numeral "$q")"
touch src/bin/multi_quantum_membrane.rs
MEMBRANE_WORDS="$WORD" cargo build --release --bin multi_quantum_membrane
exec ./target/release/multi_quantum_membrane
