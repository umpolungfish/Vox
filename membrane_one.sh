#!/usr/bin/env bash
# Bake N in as its IMASM numeral, compile, and emit its static executable — same shape as factor_one.sh.
# The decimal is consumed here by `vox numeral` (the encode-before-compile step);
# the numeral word it prints is baked into `membrane` at compile, so the run is
# pure execution of a word that already contains N.
# usage: ./membrane_one.sh <decimal N>
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
[ $# -eq 1 ] || { echo "usage: $0 <decimal N>" >&2; exit 1; }
cd "$(dirname "$0")"
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null 2>&1
AWORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral 2)"
NWORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral "$1")"
QWORD="$(./target/x86_64-unknown-linux-musl/release/vox numeral 0)"
touch src/bin/membrane.rs
MEMBRANE_WORDS="$AWORD $NWORD $QWORD" cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin membrane >/dev/null 2>&1
# The default output is a verified static executable.
# MEMBRANE_RUN=1 also executes it.
finish_static_membrane "$PWD"/target/x86_64-unknown-linux-musl/release/membrane
