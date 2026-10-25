#!/usr/bin/env bash
# Select an explicit operation order before the wrapper's original argument parser.
for ig_order_arg in "$@"; do
    if [[ "$ig_order_arg" == --ordering || "$ig_order_arg" == --orderings ]]; then
        exec python3 "$(dirname -- "${BASH_SOURCE[0]}")/ordering.py" "$0" "$@"
    fi
done
# Bake the saved IMASM word into one static native executable.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source scripts/static_membrane.sh
unset MEMBRANE_RUN
output="$PWD/membranes/vox_membrane"
word_file=${VOX_MEMBRANE_WORD_FILE:-membrane_word.imasm}
run=0
while (($#)); do
    case "$1" in
        --input) (($# >= 2)) || { echo "--input requires an IMASM word file" >&2; exit 2; }; word_file="$2"; shift 2 ;;
        --output) (($# >= 2)) || { echo "--output requires a destination path" >&2; exit 2; }; output="$2"; shift 2 ;;
        --run) run=1; shift ;;
        --help|-h) echo "usage: $0 --input FILE.imasm [--output FILE] [--run]"; exit 0 ;;
        *) echo "unknown flag: $1" >&2; exit 2 ;;
    esac
done
[[ ! -e "$output" ]] || { printf 'output already exists: %s\n' "$output" >&2; exit 2; }
mkdir -p "$(dirname "$output")"
if [[ ! -f "$word_file" && "$word_file" == membrane_word.imasm ]]; then
    word_file=membrane_word.imasm.ref
fi
python3 - "$PWD/../G-mOMonadOS" "$output" "$word_file" <<'PY'
import pathlib, sys
sys.path.insert(0, sys.argv[1])
from imasm_emit import compile_word
compile_word(pathlib.Path(sys.argv[3]).read_text().strip(), sys.argv[2])
PY
if (( run )); then MEMBRANE_RUN=1 finish_static_membrane "$output"
else finish_static_membrane "$output"
fi
