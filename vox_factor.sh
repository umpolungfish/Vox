#!/usr/bin/env bash
# Baked FDE-QC membrane factoriser: the pre-encoded IMASM numerals are read from
# the compiled-in word file and CONSUMED by vox, which PRODUCES the factors.
# No runtime input, no decimals.
set -euo pipefail
cd "$(dirname "$0")"
read -r A N Q < membrane_words.imasm
echo "membrane baked: a=${#A} N=${#N} q=${#Q} marks"
exec ./target/release/vox morphism-factor "$N"
