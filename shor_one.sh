#!/usr/bin/env bash
# shor_one.sh — Bake N into a standalone Shor Order-Finding Membrane binary,
# then EMIT that binary into membranes/shor_factor_<N>.
#
# The emitted artifact already contains N in its static data/word, takes no runtime
# inputs or arguments, and preserves the Shor δ/μ bracket execution.
#
# usage:
#   ./shor_one.sh <decimal N> [--run]                 bake (and optionally run) one membrane
#   ./shor_one.sh --all [--bound B] [--bake] [--out FILE]
#       vox every semiprime <= B (default B=1000) through the Shor path

set -euo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/scripts/static_membrane.sh"

cd "$(dirname "$0")"
VOX=./target/x86_64-unknown-linux-musl/release/vox

ensure_vox() { [ -x "$VOX" ] || cargo build --target-dir target --release --target x86_64-unknown-linux-musl --bin vox >/dev/null 2>&1; }

# semiprimes BOUND — one N per line, ascending: every p*q <= BOUND, p,q prime, p<=q
semiprimes() {
python3 - "$1" <<'PY'
import sys
B = int(sys.argv[1])
sieve = bytearray([1]) * (B + 1)
if B >= 0: sieve[0] = 0
if B >= 1: sieve[1] = 0
for i in range(2, int(B ** 0.5) + 1):
    if sieve[i]:
        for j in range(i * i, B + 1, i):
            sieve[j] = 0
primes = [i for i in range(2, B + 1) if sieve[i]]
out = set()
for i, p in enumerate(primes):
    for q in primes[i:]:
        n = p * q
        if n > B:
            break
        out.add(n)
for n in sorted(out):
    print(n)
PY
}

bake_one() { # $1 = N, $2 = run (0/1)
  local N="$1" RUN="$2" WORD OUT FRAME_NAME FRAME_MEM BUILD_KEY
  echo "🌀 [1/2] Ensuring vox compiler is built..."
  ensure_vox
  echo "🌀 [2/2] Encoding decimal N=$N into IMASM numeral word..."
  WORD="$("$VOX" numeral "$N")"
  echo "         Numeral Word: $WORD"

  FRAME_NAME=$(echo -n "$N" | sha256sum | cut -c1-16)
  OUT="membranes/shor_factor_${FRAME_NAME}"
  BUILD_KEY=$(
    {
      printf '%s\n' "$WORD" 'target=x86_64-unknown-linux-musl'
      rustc -vV
      sha256sum Cargo.toml Cargo.lock build.rs
      rg --files src -g '*.rs' -g '*.cuh' -g '*.imasm' | sort | xargs -d '\n' sha256sum
    } | sha256sum | cut -d ' ' -f 1
  )
  if [ -x "$OUT" ] && [ -f "$OUT.build-key" ] && [ "$(cat "$OUT.build-key")" = "$BUILD_KEY" ] &&
     [ -f "$OUT.sha256" ] && sha256sum --check --status "$OUT.sha256"; then
    echo "🌀 Membrane already baked: $OUT"
  else
    FACTOR_N_WORD="$WORD" cargo build --target-dir target --release --bin shor_one --target x86_64-unknown-linux-musl >/dev/null 2>&1
    mkdir -p membranes
    require_static_membrane "./target/x86_64-unknown-linux-musl/release/shor_one"
    cp -f ./target/x86_64-unknown-linux-musl/release/shor_one "$OUT"
    chmod +x "$OUT"
    printf '%s\n' "$BUILD_KEY" > "$OUT.build-key"
    sha256sum "$OUT" > "$OUT.sha256"
    echo "🌀 EMITTED BAKED SHOR MEMBRANE: $OUT"
  fi
  require_static_membrane "$OUT"
  echo ""

  if [ "$RUN" -eq 1 ]; then
    echo "==================== EXECUTING EMITTED SHOR MEMBRANE ===================="
    "$OUT"
  fi
}

# ---- arg parsing ----------------------------------------------------------
MODE=one; N=""; BOUND=1000; BAKE=0; DO_RUN=0; OUT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --all)   MODE=all ;;
    --bake)  BAKE=1 ;;
    --run)   BAKE=1; DO_RUN=1 ;;
    --bound) shift; BOUND="${1:-1000}" ;;
    --out)   shift; OUT="${1:-}" ;;
    *)  if [ -z "$N" ]; then N="$1"; else echo "unexpected argument: $1" >&2; exit 1; fi ;;
  esac
  shift
done

if [ "$MODE" = all ]; then
  ensure_vox
  mkdir -p membranes
  [ -n "$OUT" ] || OUT="membranes/shor_semiprimes_${BOUND}.tsv"
  : > "$OUT"
  total="$(semiprimes "$BOUND" | wc -l | tr -d ' ')"
  i=0; ok=0; fail=0
  echo "🌀 Sweeping ALL semiprimes <= $BOUND ($total total) through the Shor membrane..."
  while read -r N; do
    i=$((i + 1))
    # Run the Shor membrane for this N
    line="$( "$VOX" shor_factor "$N" 2>/dev/null | grep -F '= ' | head -1 )" || true
    if [ -n "$line" ] && [[ "$line" =~ =\ ([0-9]+)\ x\ ([0-9]+) ]]; then
      p="${BASH_REMATCH[1]}"; q="${BASH_REMATCH[2]}"
      shape="[shor]"
    else
      fail=$((fail + 1))
      printf '%s\t?\t?\t?\n' "$N" >> "$OUT"
      echo "  [$i/$total] $N : NO FACTOR READOUT (shor)"
      if [ "$BAKE" -eq 1 ]; then bake_one "$N" 0; fi
      continue
    fi
    if [ $(( p * q )) -eq "$N" ]; then
      ok=$((ok + 1))
      printf '%s\t%s\t%s\t%s\n' "$N" "$p" "$q" "$shape" >> "$OUT"
      echo "  [$i/$total] $N = $p x $q  $shape"
    else
      fail=$((fail + 1))
      printf '%s\t?\t?\t?\n' "$N" >> "$OUT"
      echo "  [$i/$total] $N : FACTOR PRODUCT CHECK FAILED ($line)"
    fi
    if [ "$BAKE" -eq 1 ]; then
      bake_one "$N" 0
    fi
  done < <(semiprimes "$BOUND")
  echo ""
  echo "✅ SHOR VOXED ALL SEMIPRIMES: $ok/$total factored, $fail failed"
  echo "   Manifest: $OUT"
else
  [ -n "$N" ] || { echo "usage: $0 <decimal N> [--run] | $0 --all [--bound B] [--bake] [--out FILE]" >&2; exit 1; }
  bake_one "$N" "$DO_RUN"
fi
