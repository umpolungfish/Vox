#!/usr/bin/env bash
# Select an explicit operation order before the wrapper's original argument parser.
for ig_order_arg in "$@"; do
    if [[ "$ig_order_arg" == --ordering || "$ig_order_arg" == --orderings ]]; then
        exec python3 "$(dirname -- "${BASH_SOURCE[0]}")/ordering.py" "$0" "$@"
    fi
done
# Bake a, N and register width as IMASM numerals, compile them in, and emit the static executable.
# No runtime input: the computation is already in the word.
# usage: ./multi_membrane_one.sh --number DECIMAL_N [--base DECIMAL_A] [--qubits COUNT] [--run]
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
unset MEMBRANE_RUN
cd "$(dirname "$0")"
N=""; a=2; q=13; RUN=0
while (($#)); do
    case "$1" in
        --number) (($# >= 2)) || { echo "--number requires DECIMAL_N" >&2; exit 2; }; N="$2"; shift 2 ;;
        --base) (($# >= 2)) || { echo "--base requires DECIMAL_A" >&2; exit 2; }; a="$2"; shift 2 ;;
        --qubits) (($# >= 2)) || { echo "--qubits requires COUNT" >&2; exit 2; }; q="$2"; shift 2 ;;
        --run) RUN=1; shift ;;
        --help|-h) echo "usage: $0 --number DECIMAL_N [--base DECIMAL_A] [--qubits COUNT] [--run]"; exit 0 ;;
        *) echo "unknown flag: $1" >&2; exit 2 ;;
    esac
done
[[ -n "$N" ]] || { echo "usage: $0 --number DECIMAL_N [--base DECIMAL_A] [--qubits COUNT] [--run]" >&2; exit 2; }
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null 2>&1
WORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$a") $(./target/x86_64-unknown-linux-musl/release/vox numeral "$N") $(./target/x86_64-unknown-linux-musl/release/vox numeral "$q")"
printf '%s\n' "$WORD" > membrane_words.imasm
touch src/bin/multi_quantum_membrane.rs
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin multi_quantum_membrane
# The default output is a verified static executable.
# MEMBRANE_RUN=1 also executes it.
if (( RUN )); then MEMBRANE_RUN=1 finish_static_membrane "$PWD"/target/x86_64-unknown-linux-musl/release/multi_quantum_membrane
else finish_static_membrane "$PWD"/target/x86_64-unknown-linux-musl/release/multi_quantum_membrane
fi
