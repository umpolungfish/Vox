#!/usr/bin/env bash
# sweep_binaries.sh — lift a tree of binaries into IMASM, one module per file.
#
# Written because the hand-pasted path list did not survive being pasted: a
# wrapped terminal had injected a space into `Windows/ SystemApps` and friends,
# and every one of those became a panic with the path nowhere in the message.
# A list you generate cannot be line-wrapped; a list you copy can.
#
# Usage: ./sweep_binaries.sh OUT.bin ROOT [ROOT...]
#        ./sweep_binaries.sh MicrosoftMega.bin /mnt/c/Windows/SystemApps
set -uo pipefail
VOX="$(cd "$(dirname "$0")" && pwd)/target/release/vox"
OUT="${1:?usage: sweep_binaries.sh OUT.bin ROOT [ROOT...]}"; shift
[ -x "$VOX" ] || { echo "no vox at $VOX — cargo build --release first" >&2; exit 1; }

: > "$OUT"
ok=0; skip=0; fail=0
while IFS= read -r -d '' f; do
  if [ ! -r "$f" ]; then skip=$((skip+1)); continue; fi
  if "$VOX" imasm "$f" >> "$OUT" 2>/dev/null; then
    ok=$((ok+1))
  else
    fail=$((fail+1)); printf 'unlifted: %s\n' "$f" >> "${OUT%.bin}.unlifted"
  fi
  # progress without a line per file
  if (( (ok+fail+skip) % 50 == 0 )); then printf '\r  %d lifted, %d unreadable, %d failed' "$ok" "$skip" "$fail" >&2; fi
done < <(find "$@" -type f \( -iname '*.exe' -o -iname '*.dll' \) -print0 2>/dev/null)

printf '\r  %d lifted, %d unreadable, %d failed\n' "$ok" "$skip" "$fail" >&2
echo "  -> $OUT ($(du -h "$OUT" | cut -f1))" >&2
[ -s "${OUT%.bin}.unlifted" ] && echo "  -> $(wc -l < "${OUT%.bin}.unlifted") paths listed in ${OUT%.bin}.unlifted" >&2
exit 0
