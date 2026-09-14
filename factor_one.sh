#!/usr/bin/env bash
# Bake N into the program, compile, and run it once.
# usage: ./factor_one.sh <decimal N>
set -euo pipefail
[ $# -eq 1 ] || { echo "usage: $0 <decimal N>" >&2; exit 1; }
cd "$(dirname "$0")"
# FORCE a rebuild of just this bin so the new N is compiled in.
touch src/bin/factor_one.rs
FACTOR_N="$1" cargo build --release --bin factor_one >/dev/null 2>&1
exec ./target/release/factor_one
