#!/usr/bin/env bash
# Build a layered phase sandwich (chipdip): N's numeral nested as the core inside
# `layers` stacked factor-lane work frames. The free-lunch rule makes each work
# frame transform the whole nest, and each layer banks its reversal, so the word
# does its work in one pass. usage: ./chipdip.sh <decimal N> [layers]
set -euo pipefail
[ $# -ge 1 ] || { echo "usage: $0 <decimal N> [layers]" >&2; exit 1; }
cd "$(dirname "$0")"
cargo build --release --bin vox >/dev/null 2>&1
NUM="$(./target/release/vox numeral "$1")"
LAYERS="${2:-7}"
python3 -c "
s='$NUM'; body=s[1:-3]          # strip the numeral's own seed+commit+close
W='≻⊤≺⊥⊞⋈'                       # the factor-lane work run: two poles, reversal, hold, link
L=int('$LAYERS')
opn='⊢⊙'+('∈'+W)*(L-1)+'∈'
cls=(W+'∋')*L+'⊡⊣'
print(opn+body+cls)"
