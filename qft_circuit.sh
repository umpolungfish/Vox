#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")"

if [[ ${1:-} == --help ]] || [[ $# -lt 1 ]]; then
    echo 'usage: qft_circuit.sh <N> [a] [qubits]'
    echo '  N      - decimal semiprime to factor (required)'
    echo '  a      - base for order finding (default: 2)'
    echo '  qubits - QFT register size in qubits (default: auto)'
    echo ''
    echo 'Bakes N into a static QFT/Shor membrane binary and emits it.'
    echo 'The emitted binary contains N baked in as IMASM numeral word.'
    echo 'Run the binary directly to factor.'
    exit 0
fi

N="${1:-}"
A="${2:-2}"
QUBITS="${3:-0}"

echo "Baking QFT/Shor membrane for N=$N (a=$A, qubits=$QUBITS)..."

# Encode N as IMASM numeral word
VOX=./target/x86_64-unknown-linux-musl/release/vox
if [ ! -x "$VOX" ]; then
    cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null 2>&1
fi
WORD="$("$VOX" numeral "$N")"

# Bake N into shor_one static binary
touch src/bin/shor_one.rs
FACTOR_N_WORD="$WORD" cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin shor_one >/dev/null 2>&1

BINARY="./target/x86_64-unknown-linux-musl/release/shor_one"

echo "EMITTED BAKED MEMBRANE: $BINARY"
echo "Run: $BINARY"