#!/usr/bin/env bash
# toroidal_one.sh — Bake N into a standalone Toroidal Resident Membrane binary,
# then EMIT that binary into membranes/toroidal_factor_<N>.
#
# The emitted artifact already contains N in its static data/word, takes no runtime
# inputs or arguments, and preserves quantum phase preparation.
#
# Sweep mode (--all) voxes ALL semiprimes p*q (p,q prime, p<=q) with p*q <= bound
# through the toroidal path the baked membrane runs: the short-frontier scout
# (scout_semiprime) first, and when the scout verdicts HARD (factors far apart)
# the shape-routed membrane (vox factor: rho fused with the sieve). Emits a TSV
# manifest N -> p x q [shape]. No per-N cargo build is needed for the sweep;
# add --bake to also EMIT each per-N baked membrane (slow: one cargo build per N).
#
# usage:
#   ./toroidal_one.sh <decimal N> [--run]                 bake (and optionally run) one membrane
#   ./toroidal_one.sh --all [--bound B] [--bake] [--out FILE]
#       vox every semiprime <= B (default B=1000)
set -euo pipefail

cd "$(dirname "$0")"
VOX=./target/release/vox

ensure_vox() { [ -x "$VOX" ] || cargo build --release --bin vox >/dev/null 2>&1; }

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
  local N="$1" RUN="$2" WORD OUT FRAME_NAME FRAME_MEM
  echo "🌀 [1/2] Ensuring vox compiler is built..."
  ensure_vox
  echo "🌀 [2/2] Encoding decimal N=$N into IMASM numeral word..."
  WORD="$("$VOX" numeral "$N")"
  echo "         Numeral Word: $WORD"

  FRAME_NAME=$(echo -n "$N" | sha256sum | cut -c1-16)
  FRAME_MEM="membranes/frame_factor_${FRAME_NAME}"
  OUT="membranes/toroidal_factor_${FRAME_NAME}"
  if [ -f "$OUT" ] && [ -x "$OUT" ] && [ -f "$FRAME_MEM" ] && [ -x "$FRAME_MEM" ]; then
    echo "🌀 Membrane already baked: $OUT"
  else
    touch src/bin/toroidal_one.rs
    FACTOR_N_WORD="$WORD" cargo build --release --bin toroidal_one --target x86_64-unknown-linux-musl >/dev/null 2>&1
    touch src/bin/frame_factor_one.rs
    FRAME_FACTOR_N_WORD="$WORD" FRAME_FACTOR_WIDTH=2 cargo build --release --bin frame_factor_one --target x86_64-unknown-linux-musl >/dev/null 2>&1
    cp -f ./target/x86_64-unknown-linux-musl/release/frame_factor_one "$FRAME_MEM"
    chmod +x "$FRAME_MEM"
    mkdir -p membranes
    cp -f ./target/x86_64-unknown-linux-musl/release/toroidal_one "$OUT"
    chmod +x "$OUT"
    echo "🌀 EMITTED BAKED MEMBRANE: $OUT"
  fi
  echo ""

  if [ "$RUN" -eq 1 ]; then
    echo "==================== EXECUTING EMITTED MEMBRANE ===================="
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
  [ -n "$OUT" ] || OUT="membranes/toroidal_semiprimes_${BOUND}.tsv"
  : > "$OUT"
  total="$(semiprimes "$BOUND" | wc -l | tr -d ' ')"
  i=0; ok=0; fail=0
  echo "🌀 Sweeping ALL semiprimes <= $BOUND ($total total) through the toroidal scout membrane..."
  while read -r N; do
    i=$((i + 1))
    # 1) toroidal scout: the short-frontier shape probe
    line="$( "$VOX" scout "$N" 2>/dev/null | grep -F '= ' | head -1 )" || true
    if [ -n "$line" ] && [[ "$line" =~ =\ ([0-9]+)\ x\ ([0-9]+) ]]; then
      p="${BASH_REMATCH[1]}"; q="${BASH_REMATCH[2]}"
      shape="${line#* x }"; shape="${shape##* }"
    else
      # 2) scout verdict HARD -> hand to the membrane (rho fused with the sieve)
      line="$( "$VOX" factor "$N" 2>/dev/null | grep -F '= ' | head -1 )" || true
      if [ -n "$line" ] && [[ "$line" =~ =\ ([0-9]+)\ x\ ([0-9]+) ]]; then
        p="${BASH_REMATCH[1]}"; q="${BASH_REMATCH[2]}"
        shape="[membrane]"
      else
        fail=$((fail + 1))
        printf '%s\t?\t?\t?\n' "$N" >> "$OUT"
        echo "  [$i/$total] $N : NO FACTOR READOUT (scout + membrane)"
        if [ "$BAKE" -eq 1 ]; then bake_one "$N" 0; fi
        continue
      fi
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
  echo "✅ VOXED ALL SEMIPRIMES: $ok/$total factored, $fail failed"
  echo "   Manifest: $OUT"
else
  [ -n "$N" ] || { echo "usage: $0 <decimal N> [--run] | $0 --all [--bound B] [--bake] [--out FILE]" >&2; exit 1; }
  bake_one "$N" "$DO_RUN"
fi
