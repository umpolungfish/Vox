#!/usr/bin/env bash
# Bake a, N and register width as IMASM numerals, compile them in, and emit the static executable.
# No runtime input: the computation is already in the word.
# usage: ./multi_membrane_one.sh <decimal N> [a] [qubits]
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
cd "$(dirname "$0")"
N="${1:?usage: $0 <decimal N> [a] [qubits]}"
a="${2:-2}"; q="${3:-13}"
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null 2>&1
WORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$a") $(./target/x86_64-unknown-linux-musl/release/vox numeral "$N") $(./target/x86_64-unknown-linux-musl/release/vox numeral "$q")"
printf '%s\n' "$WORD" > membrane_words.imasm
touch src/bin/multi_quantum_membrane.rs
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin multi_quantum_membrane
# The default output is a verified static executable.
# MEMBRANE_RUN=1 also executes it.
finish_static_membrane "$PWD"/target/x86_64-unknown-linux-musl/release/multi_quantum_membrane
