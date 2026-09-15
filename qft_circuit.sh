#!/usr/bin/env bash
set -euo pipefail
if [[ ${1:-} == --help ]] || (( $# == 0 )); then
    echo 'usage: qft_circuit.sh <levels: 1..12> [--stdin | hex-mask[:feedback-0-or-1] ...]'
    echo 'Builds a depth-selected circuit, lifts it, and switches gates in resident Vox.'
    echo 'A 64-bit mask repeats across the ports. Feedback samples the preceding output.'
    echo 'The direct-transform verification runner accepts up to 4096 ports.'
    exit 0
fi
[[ $1 =~ ^([1-9]|1[0-2])$ ]] || { echo 'Verification depth must be 1..12' >&2; exit 2; }
circuit_levels=$1
shift
cd "$(dirname "$0")"
cargo build --release --bin vox
CIRCUIT_LEVELS="$circuit_levels" CARGO_TARGET_DIR=target/membranes-musl \
    RUSTFLAGS='-C relocation-model=static' cargo build --release \
    --target x86_64-unknown-linux-musl --bin qft_circuit
circuit_directory="membranes/qft_circuit/$((1 << circuit_levels))"
mkdir -p "$circuit_directory"
cp target/membranes-musl/x86_64-unknown-linux-musl/release/qft_circuit "$circuit_directory/payload.elf"
./target/release/vox imasm "$circuit_directory/payload.elf" > /dev/null
./target/release/vox circuit "$circuit_directory/payload.elf.imasm" "$@" | tee "$circuit_directory/resident.log"
