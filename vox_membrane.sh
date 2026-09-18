#!/usr/bin/env bash
# Baked FDE-QC membrane: the word is pre-encoded IMASM, compiled by the imasm
# toolchain and run one-shot. No runtime input, no decimals, no read-back.
set -euo pipefail
cd "$(dirname "$0")"
W="$(cat membrane_word.imasm)"
echo "baked membrane word: ${#W} marks"
python3 - "$W" <<'PY'
import sys
sys.path.insert(0, '/home/mrnob0dy666/imsgct/G-mOMonadOS')
from imasm_emit import compile_word
compile_word(sys.argv[1].strip(), '/tmp/membrane.bin')
print('compiled -> /tmp/membrane.bin')
PY
chmod +x /tmp/membrane.bin
exec /tmp/membrane.bin
