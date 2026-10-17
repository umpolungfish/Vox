#!/usr/bin/env bash
# Generate standalone glyph sequences, recover exact modules, execute controls.
set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"
cd "$(dirname "$0")"
cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox
mkdir -p membranes/glyphs
for glyph_spec in \
  'binomial binomial_one/0_1_2_8_9_12_25_27_30_81_120_81' \
  'lcm lcm_one/0_1_10_20_40_80_100_10' \
  'divisor divisor_one/1_2_97_360_65536' \
  'landau landau_one/0_1_10_15_30' \
  'schutte schutte_one/7_1_2_3' \
  'shor shor_one/2_21_12'; do
    read -r glyph_name glyph_case <<< "$glyph_spec"
    glyph_source="membranes/$glyph_case"
    glyph_dest="membranes/glyphs/$glyph_name"
    require_static_membrane "$glyph_source/payload.elf"
    if ! test -e "$glyph_dest.elf"; then
        cp "$glyph_source/payload.elf" "$glyph_dest.elf"
    fi
    require_static_membrane "$glyph_dest.elf"
    cmp "$glyph_source/payload.elf" "$glyph_dest.elf"
    # Create-new CLI output prevents replacement of an existing word. On a
    # repeat invocation, verify the existing artifact rather than overwriting it.
    if ! test -e "$glyph_dest.glyphs"; then
        ./target/x86_64-unknown-linux-musl/release/vox glyphs "$glyph_source/payload.elf.imasm" "$glyph_dest.glyphs"
    fi
    if ! test -e "$glyph_dest.recovered.imasm"; then
        ./target/x86_64-unknown-linux-musl/release/vox unglyphs "$glyph_dest.glyphs" "$glyph_dest.recovered.imasm"
    fi
    cmp "$glyph_source/payload.elf.imasm" "$glyph_dest.recovered.imasm"
    /usr/bin/time -o "$glyph_dest.time" -f '%e seconds elapsed' \
        ./target/x86_64-unknown-linux-musl/release/vox run "$glyph_dest.glyphs" > "$glyph_dest.stdout" 2> "$glyph_dest.stderr"
    rg -q '^entry\(\.\.\.\) exited\(0\)' "$glyph_dest.stdout"
    diff -u "$glyph_source/native.stdout" <(sed '/^entry(\.\.\.) exited(0) /d' "$glyph_dest.stdout")
    diff -u "$glyph_source/native.stderr" "$glyph_dest.stderr"
    printf '%s: exact module recovery and native-output agreement\n' "$glyph_name"
    tail -1 "$glyph_dest.stdout"
    cat "$glyph_dest.time"
done
