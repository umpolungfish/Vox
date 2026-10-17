#!/usr/bin/env bash
# Bake a dynamic IMASM numeral into an executable. Execution is a separate step.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
usage() {
    printf 'usage: %s DECIMAL_N [OUTPUT_DIRECTORY]\n       %s --input SINGLE_CASE.jsonl [OUTPUT_DIRECTORY]\n' "$0" "$0"
}
if [[ ${1:-} == --help || ${1:-} == -h ]]; then usage; exit 0; fi
if [[ ${1:-} == --input ]]; then
    [[ $# -ge 2 && $# -le 3 ]] || { usage >&2; exit 2; }
    bake_n=$(jq -er -s 'if length == 1 then .[0].n else error("Expected exactly one case") end' "$2")
    bake_output=${3:-}
else
    [[ $# -ge 1 && $# -le 2 ]] || { usage >&2; exit 2; }
    bake_n=$1
    bake_output=${2:-}
fi
[[ $bake_n =~ ^[1-9][0-9]*$ && $bake_n != 1 ]] || {
    printf 'N must be a canonical decimal integer greater than one.\n' >&2; exit 2;
}
bake_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
cd -- "$bake_root"
if [[ -z $bake_output ]]; then
    mkdir -p membranes/baked_factor
    bake_output=$(mktemp -d "$bake_root/membranes/baked_factor/factor.XXXXXXXX")
else
    [[ $bake_output == /* ]] || bake_output="$bake_root/$bake_output"
    mkdir -p -- "$(dirname -- "$bake_output")"
    mkdir -- "$bake_output"
fi
bake_output=$(cd -- "$bake_output" && pwd)
bake_fail() {
    printf 'Bake failed; build diagnostics are in %s\n' "$bake_output" >&2
}
trap bake_fail ERR
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox \
    >"$bake_output/vox_build.stdout" 2>"$bake_output/vox_build.stderr"
./target/x86_64-unknown-linux-musl/release/vox numeral "$bake_n" >"$bake_output/input.imasm"
bake_word=$(<"$bake_output/input.imasm")
FACTOR_N_WORD="$bake_word" cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin factor_one \
    >"$bake_output/build.stdout" 2>"$bake_output/build.stderr"
require_static_membrane "target/x86_64-unknown-linux-musl/release/factor_one"
cp -- target/x86_64-unknown-linux-musl/release/factor_one "$bake_output/factor_one"
printf '%s\n' "$bake_n" >"$bake_output/input.decimal"
sha256sum "$bake_output/factor_one" >"$bake_output/factor_one.sha256"
printf '%s\n' "$bake_output/factor_one"
