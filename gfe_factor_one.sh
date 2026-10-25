#!/usr/bin/env bash
# Select an explicit operation order before the wrapper's original argument parser.
for ig_order_arg in "$@"; do
    if [[ "$ig_order_arg" == --ordering || "$ig_order_arg" == --orderings ]]; then
        exec python3 "$(dirname -- "${BASH_SOURCE[0]}")/ordering.py" "$0" "$@"
    fi
done
# gfe_factor_one.sh — Bake N into a standalone Geometer Factor Extractor (GFE)
# Resident Membrane binary, then EMIT that binary into membranes/gfe_factor_<ID>.
#
# Follows the toroidal_one.sh ez-bake pattern:
# - Accepts decimal N, IMASM numeral file, or raw IMASM word.
# - Bakes the N-only IMASM word, the 8-monomer catalog chain, and the small-prime
#   orbit mask directly into the Cargo static image at compile time.
# - Emits a self-contained static executable with zero runtime arguments.
# - Checks build-key and SHA256 cache to avoid redundant builds.
# - Rewires the complete image, round-trips its executable glyph module, and
#   executes that module through Vox if --run or invoked in default mode.
#
# usage:
#   ./gfe_factor_one.sh <N or file.imasm> [--bake] [--run] [--out <path>]
set -euo pipefail

orig_pwd=$(pwd)
gfe_root=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
source "$gfe_root/scripts/static_membrane.sh"

VOX="$gfe_root/target/x86_64-unknown-linux-musl/release/vox"

ensure_vox() {
    cargo build --offline --target-dir target --release --target x86_64-unknown-linux-musl -j1 --bin vox \
        >"$gfe_root/target/gfe_vox_build.stdout" 2>"$gfe_root/target/gfe_vox_build.stderr"
}

# ---- arg parsing ----------------------------------------------------------
DO_RUN=1
DO_BAKE=1
INPUT=""
OUT_OVERRIDE=""

while [ $# -gt 0 ]; do
    case "$1" in
        --bake) DO_RUN=0; DO_BAKE=1 ;;
        --run)  DO_RUN=1; DO_BAKE=1 ;;
        --out)  shift; OUT_OVERRIDE="${1:-}" ;;
        *)
            if [ -z "$INPUT" ]; then
                INPUT="$1"
            else
                echo "unexpected argument: $1" >&2
                exit 1
            fi
            ;;
    esac
    shift
done

if [ -z "$INPUT" ]; then
    printf 'usage: %s <DECIMAL_N | FILE.imasm | IMASM_WORD> [--bake] [--run] [--out <path>]\n' "$0" >&2
    exit 2
fi

# Resolve file paths before changing directory
if [ -f "$orig_pwd/$INPUT" ]; then
    INPUT="$orig_pwd/$INPUT"
elif [ -f "$gfe_root/$INPUT" ]; then
    INPUT="$gfe_root/$INPUT"
fi

cd -- "$gfe_root"

# ---- resolve input to IMASM word and optional decimal ---------------------
WORD=""
DECIMAL=""

if [ -f "$INPUT" ]; then
    RAW=$(tr -d '[:space:]' < "$INPUT")
    if [[ "$RAW" =~ ^[1-9][0-9]*$ ]]; then
        DECIMAL="$RAW"
        ensure_vox
        WORD="$("$VOX" numeral "$RAW")"
    elif [[ "$RAW" == ⊢* ]]; then
        WORD="$RAW"
    else
        echo "Unrecognized format in input file: $INPUT" >&2
        exit 1
    fi
elif [[ "$INPUT" =~ ^[1-9][0-9]*$ ]]; then
    DECIMAL="$INPUT"
    ensure_vox
    WORD="$("$VOX" numeral "$INPUT")"
elif [[ "$INPUT" == ⊢* ]]; then
    WORD="$INPUT"
else
    echo "Input must be a decimal integer, an .imasm file, or an IMASM numeral word: $INPUT" >&2
    exit 1
fi

FRAME_NAME=$(printf "%s\n" "$WORD" "${IG_ORDER_SITE:-}" "${IG_ORDER_SPEC:-}" | sha256sum | cut -c1-16)

if [ -n "$OUT_OVERRIDE" ]; then
    OUT="$OUT_OVERRIDE"
    [[ "$OUT" == /* ]] || OUT="$gfe_root/$OUT"
else
    OUT="$gfe_root/membranes/gfe_factor_${FRAME_NAME}"
fi

BAKE_DIR="$gfe_root/membranes/baked_gfe/gfe_${FRAME_NAME}"
mkdir -p "$gfe_root/membranes" "$BAKE_DIR"

trap 'printf "GFE bake failed; diagnostics are in %s\n" "$BAKE_DIR" >&2' ERR

# ---- 8-monomer telechelic polymer sequence --------------------------------
gfe_polymer="$BAKE_DIR/catalog_polymer_sequence.tsv"
gfe_imscribe=${IMSCRIBE_BIN:-/home/mrnob0dy666/imscribing_grammar/.venv/bin/imscribe}
gfe_catalog="$HOME/.imscrbgrmr/catalog.json"
[ -x "$gfe_imscribe" ] || { echo "Native imscribe is required: $gfe_imscribe" >&2; exit 1; }
[ -f "$gfe_catalog" ] || { echo "Main catalog is absent: $gfe_catalog" >&2; exit 1; }
gfe_polymer_pending="$BAKE_DIR/catalog_polymer_sequence.pending.tsv"
: >"$gfe_polymer_pending"
for gfe_monomer in torus_encoding microtubule grammar_enclosing_repair \
    zeta_sic_dual_frame dual_d12_inverse exact_product_terminal \
    sic_d12_gram_matrix gabor_torus_dual_frame; do
    gfe_tuple=$(IG_CATALOG="$gfe_catalog" "$gfe_imscribe" analyze "$gfe_monomer" --format notation)
    [[ "$gfe_tuple" == ⟨*⟩ ]] || { echo "Native catalog lookup failed for $gfe_monomer" >&2; exit 1; }
    printf '%s\t%s\n' "$gfe_monomer" "$gfe_tuple" >>"$gfe_polymer_pending"
done
mv "$gfe_polymer_pending" "$gfe_polymer"

# ---- small prime orbit mask ------------------------------------------------
gfe_mask_file="$BAKE_DIR/small_prime_orbit_mask.imasm"
if [ ! -f "$gfe_mask_file" ]; then
    if [ -f "$gfe_root/membranes/small_prime_orbit_mask.imasm" ]; then
        cp -f "$gfe_root/membranes/small_prime_orbit_mask.imasm" "$gfe_mask_file"
    else
        cargo build --offline --target-dir target --release --target x86_64-unknown-linux-musl -j1 --bin gfe_mask_prepare >/dev/null 2>&1
        ./target/x86_64-unknown-linux-musl/release/gfe_mask_prepare >"$gfe_mask_file"
    fi
fi

printf '%s\n' "$WORD" >"$BAKE_DIR/input.imasm"
[ -n "$DECIMAL" ] && printf '%s\n' "$DECIMAL" >"$BAKE_DIR/input.decimal"

# ---- canonical build key ---------------------------------------------------
gfe_build_key=$(
    {
        printf '%s\n' "$WORD" "$(cat "$gfe_polymer")" \
            "$(cat "$gfe_mask_file")" \
            "order_site=${IG_ORDER_SITE:-}" "order_spec=${IG_ORDER_SPEC:-}" \
            'target=x86_64-unknown-linux-musl' 'carrier=GFE-open-telechelic-v1' 'strip=none'
        rustc -vV
        sha256sum Cargo.toml Cargo.lock build.rs gfe_factor_one.sh scripts/static_membrane.sh tools/lift_gfe.py
        rg --files src -g '*.rs' -g '*.cuh' -g '*.imasm' | sort | xargs -d '\n' sha256sum
    } | sha256sum | cut -d ' ' -f 1
)

gfe_key_file="$OUT.build-key"
gfe_hash_file="$OUT.sha256"

if [ -x "$OUT" ] && [ -f "$gfe_key_file" ] && [ -f "$gfe_hash_file" ] && \
   [ "$(cat "$gfe_key_file" 2>/dev/null || true)" = "$gfe_build_key" ] && \
   sha256sum --check --status "$gfe_hash_file"; then
    echo "🌀 GFE Membrane already baked: $OUT"
else
    echo "🌀 Baking GFE static membrane for $FRAME_NAME..."
    VOX_FACTOR_N_FILE="$BAKE_DIR/input.imasm" \
    GFE_POLYMER_SEQUENCE_FILE="$gfe_polymer" \
    GFE_SMALL_PRIME_MASK_FILE="$gfe_mask_file" \
    CARGO_PROFILE_RELEASE_STRIP=none \
        cargo build --offline --target-dir target --release --target x86_64-unknown-linux-musl -j1 --bin gfe_factor_one \
        >"$BAKE_DIR/build.stdout" 2>"$BAKE_DIR/build.stderr"
    cp -f ./target/x86_64-unknown-linux-musl/release/gfe_factor_one "$OUT"
    chmod +x "$OUT"
    require_static_membrane "$OUT"
    printf '%s\n' "$gfe_build_key" >"$gfe_key_file"
    sha256sum "$OUT" >"$gfe_hash_file"
    echo "🌀 EMITTED BAKED GFE MEMBRANE: $OUT"
fi

require_static_membrane "$OUT"

{
    [ -n "$DECIMAL" ] && printf 'source_decimal=%s\n' "$DECIMAL"
    printf 'catalog_source=%s\n' "$gfe_catalog"
    printf 'catalog_sha256=%s\n' "$(sha256sum "$gfe_catalog" | cut -d ' ' -f 1)"
    printf 'source_imasm_sha256=%s\n' "$(sha256sum "$BAKE_DIR/input.imasm" | cut -d ' ' -f 1)"
    printf 'polymer_sequence_sha256=%s\n' "$(sha256sum "$gfe_polymer" | cut -d ' ' -f 1)"
    printf 'small_prime_mask_imasm_sha256=%s\n' "$(sha256sum "$gfe_mask_file" | cut -d ' ' -f 1)"
    printf 'order_site=%s\norder_spec=%s\n' "${IG_ORDER_SITE:-}" "${IG_ORDER_SPEC:-}"
    printf 'build_key=%s\n' "$gfe_build_key"
    cat "$gfe_hash_file"
} >"$BAKE_DIR/bake.manifest"

ensure_vox
gfe_lift_key=$(
    {
        sha256sum "$OUT" "$VOX" tools/lift_gfe.py render_carrier_diagram.py \
            ../G-mOMonadOS/tools/vox_binary_rewire.py ../IMSCRIBr/native_wiring.py \
            ../IMSCRIBr/native/wiring_probe.rs ../IMSCRIBr/symbolic_diagram.py \
            ../IMSCRIBr/wiring.py ../IMSCRIBr/tokens.py
        rg --files ../MoDoT/imasm_core/src -g '*.rs' | sort | xargs -d '\n' sha256sum
    } | sha256sum | cut -d ' ' -f 1
)
gfe_lift_dir="$BAKE_DIR/lift_${gfe_build_key}_$gfe_lift_key"
gfe_glyphs=$(python3 "$gfe_root/tools/lift_gfe.py" "$OUT" --vox "$VOX" --output "$gfe_lift_dir")
printf 'GFE executable IMASM: %s\n' "$gfe_glyphs"

if [ "$DO_RUN" -eq 1 ]; then
    echo "==================== EXECUTING EMITTED GFE MEMBRANE ===================="
    exec "$VOX" run "$gfe_glyphs"
fi
