#!/usr/bin/env bash
# Bake the saved numeric words; runtime needs only the emitted executable.
set -euo pipefail
cd "$(dirname "${BASH_SOURCE[0]}")"
source scripts/static_membrane.sh
MEMBRANE_WORDS=$(< membrane_words.imasm)
export MEMBRANE_WORDS
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin membrane
finish_static_membrane "$PWD/target/x86_64-unknown-linux-musl/release/membrane"
