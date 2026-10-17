#!/usr/bin/env bash
# Bake the saved IMASM word into one static native executable.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source scripts/static_membrane.sh
output=${1:-"$PWD/membranes/vox_membrane"}
[[ ! -e "$output" ]] || { printf 'output already exists: %s\n' "$output" >&2; exit 2; }
mkdir -p "$(dirname "$output")"
word_file=${VOX_MEMBRANE_WORD_FILE:-membrane_word.imasm}
if [[ ! -f "$word_file" && "$word_file" == membrane_word.imasm ]]; then
    word_file=membrane_word.imasm.ref
fi
python3 - "$PWD/../G-mOMonadOS" "$output" "$word_file" <<'PY'
import pathlib, sys
sys.path.insert(0, sys.argv[1])
from imasm_emit import compile_word
compile_word(pathlib.Path(sys.argv[3]).read_text().strip(), sys.argv[2])
PY
finish_static_membrane "$output"
