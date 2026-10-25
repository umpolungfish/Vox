#!/usr/bin/env bash
# Select an explicit operation order before the wrapper's original argument parser.
for ig_order_arg in "$@"; do
    if [[ "$ig_order_arg" == --ordering || "$ig_order_arg" == --orderings ]]; then
        exec python3 "$(dirname -- "${BASH_SOURCE[0]}")/ordering.py" "$0" "$@"
    fi
done
# Bake N in as its IMASM numeral, compile, and emit its static executable — same shape as factor_one.sh.
# The decimal is consumed here by `vox numeral` (the encode-before-compile step);
# the numeral word it prints is baked into `membrane` at compile, so the run is
# pure execution of a word that already contains N.
# usage: ./membrane_one.sh --number DECIMAL_N [--run]
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
unset MEMBRANE_RUN
N=""
RUN=0
while (($#)); do
    case "$1" in
        --number) (($# >= 2)) || { echo "--number requires DECIMAL_N" >&2; exit 2; }; N="$2"; shift 2 ;;
        --run) RUN=1; shift ;;
        --help|-h) echo "usage: $0 --number DECIMAL_N [--run]"; exit 0 ;;
        *) echo "unknown flag: $1" >&2; exit 2 ;;
    esac
done
[[ -n "$N" ]] || { echo "usage: $0 --number DECIMAL_N [--run]" >&2; exit 2; }
cd "$(dirname "$0")"
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null 2>&1
AWORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral 2)"
NWORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$N")"
QWORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral 0)"
touch src/bin/membrane.rs
MEMBRANE_WORDS="$AWORD $NWORD $QWORD" cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin membrane >/dev/null 2>&1
# The default output is a verified static executable.
# MEMBRANE_RUN=1 also executes it.
if (( RUN )); then MEMBRANE_RUN=1 finish_static_membrane "$PWD"/target/x86_64-unknown-linux-musl/release/membrane
else finish_static_membrane "$PWD"/target/x86_64-unknown-linux-musl/release/membrane
fi
