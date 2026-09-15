#!/usr/bin/env bash
# Convert all numeric parameters to IMASM numerals before compiling.
set -euo pipefail
usage() {
    echo 'usage: membrane_one.sh abc <epsilon numerator> <denominator> <cutoff>...'
    echo '       membrane_one.sh divisor <N>...'
    echo '       membrane_one.sh shor <a> <N> <qubits>'
    echo '       membrane_one.sh schutte <vertices> <subset size>...'
    echo '       membrane_one.sh landau <N>...'
    echo '       membrane_one.sh tripsum <limit>...'
    echo '       membrane_one.sh factor <N>'
    echo 'Builds a static executable, lifts its complete module, and runs it in Vox.'
}
if [[ ${1:-} == --help ]]; then usage; exit 0; fi
if (( $# < 2 )); then usage >&2; exit 2; fi
case "$1" in
    abc) membrane_binary=abc_one; minimum=3 ;;
    divisor) membrane_binary=divisor_one; minimum=1 ;;
    shor) membrane_binary=shor_one; minimum=3 ;;
    schutte) membrane_binary=schutte_one; minimum=2 ;;
    landau) membrane_binary=landau_one; minimum=1 ;;
    tripsum) membrane_binary=tripsum_one; minimum=1 ;;
    factor) membrane_binary=factor_one; minimum=1 ;;
    *) usage >&2; exit 2 ;;
esac
shift
if (( $# < minimum )); then usage >&2; exit 2; fi
if [[ $membrane_binary == factor_one && $# != 1 ]]; then usage >&2; exit 2; fi
cd "$(dirname "$0")"
cargo build --release --bin vox
membrane_words=()
for parameter in "$@"; do
    [[ $parameter =~ ^[0-9]+$ ]] || { echo 'Parameters must be unsigned decimal integers' >&2; exit 2; }
    membrane_words+=("$(./target/release/vox numeral "$parameter")")
done
membrane_case=$(IFS=_; echo "$*")
membrane_output="membranes/${membrane_binary}/${membrane_case}"
mkdir -p "$membrane_output"
FACTOR_N_WORD="${membrane_words[0]}" MEMBRANE_WORDS="${membrane_words[*]}" CARGO_TARGET_DIR=target/membranes-musl \
    RUSTFLAGS='-C relocation-model=static' cargo build --release \
    --target x86_64-unknown-linux-musl --bin "$membrane_binary"
membrane_executable="target/membranes-musl/x86_64-unknown-linux-musl/release/$membrane_binary"
cp "$membrane_executable" "$membrane_output/payload.elf"
"$membrane_output/payload.elf" > "$membrane_output/native.stdout" 2> "$membrane_output/native.stderr"
./target/release/vox imasm "$membrane_output/payload.elf" > /dev/null
/usr/bin/time -o "$membrane_output/vox.time" -f '%e seconds elapsed' \
    ./target/release/vox run "$membrane_output/payload.elf.imasm" \
    > "$membrane_output/vox.stdout" 2> "$membrane_output/vox.stderr"
cat "$membrane_output/vox.stdout" "$membrane_output/vox.stderr"
# The CLI can return zero after a VM halt, so require the process-exit marker.
if ! rg -q 'entry\(\.\.\.\) exited\(0\)' "$membrane_output/vox.stdout" "$membrane_output/vox.stderr"; then
    echo "Vox execution failed; see $membrane_output" >&2
    exit 1
fi
sed '/^entry(\.\.\.) exited(0) /d' "$membrane_output/vox.stdout" > "$membrane_output/checked.stdout"
diff -u "$membrane_output/native.stdout" "$membrane_output/checked.stdout" > "$membrane_output/output.diff"
diff -u "$membrane_output/native.stderr" "$membrane_output/vox.stderr" >> "$membrane_output/output.diff"
echo "Native control matches Vox output. Complete module: $membrane_output/payload.elf.imasm"
