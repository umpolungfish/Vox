#!/usr/bin/env bash
set -euo pipefail

python3 "$(dirname "$0")/fix_rustdoc_pseudocode.py"

cargo test --release --doc
cargo test --release

echo
echo "rustdoc pseudocode repair validated."
