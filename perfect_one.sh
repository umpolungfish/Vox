#!/usr/bin/env bash
# Bake a value as its IMASM numeral into a depth-n perfect membrane, compile, and
# emit its static executable. The transform rides the matched delta/mu circuit; the run takes no
# input. usage: ./perfect_one.sh <decimal value> [depth]
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
[ $# -ge 1 ] || { echo "usage: $0 <decimal value> [depth]" >&2; exit 1; }
cd "$(dirname "$0")"
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null 2>&1
WORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$1")"
DEPTH="${2:-2}"
touch src/bin/perfect_one.rs
PERFECT_N_WORD="$WORD" PERFECT_DEPTH="$DEPTH" cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin perfect_one >/dev/null 2>&1
# The default output is a verified static executable.
# MEMBRANE_RUN=1 also executes it.
finish_static_membrane "$PWD"/target/x86_64-unknown-linux-musl/release/perfect_one
