#!/usr/bin/env bash
set -euo pipefail

f=src/morphism_factor.rs

if [[ ! -f "$f" ]]; then
  echo "missing $f (run this from the Vox repository root)" >&2
  exit 2
fi

python3 - "$f" <<'PY'
from pathlib import Path
import sys

path = Path(sys.argv[1])
s = path.read_text()

helper = r'''/// Marks which the carrier grammar permits outside a dispatch motif.  They are
/// carried across the resident object but deliberately emit no morphism from
/// `construct_carrier`.
fn non_dispatch_carry(g: char) -> bool {
    g == AREV || g == '⊞' || g == IMSCRIB
}

/// The morphism tower is a dispatch projection, not a byte-for-byte copy of
/// the walk: standalone carry marks are intentionally absent from it.  Check
/// that the tower work can be obtained from the preserved walk work by deleting
/// only those legal non-dispatch carries.
///
/// A small DP is used rather than a greedy subsequence walk because the same
/// glyphs (AREV/ENGAGR/IMSCRIB) may also occur *inside* a real morphism motif;
/// at such a position the mark may either be consumed by the tower or be a
/// standalone carry, and the later context decides which reading is valid.
fn dispatch_projection_holds(walk_work: &[char], tower_work: &[char]) -> bool {
    let mut reachable = vec![false; tower_work.len() + 1];
    reachable[0] = true;

    for &g in walk_work {
        let mut next = vec![false; tower_work.len() + 1];
        for j in 0..=tower_work.len() {
            if !reachable[j] { continue; }

            // The carrier constructor explicitly permits this mark to pass
            // without dispatching a morphism.
            if non_dispatch_carry(g) { next[j] = true; }

            // Or this occurrence can belong to the next dispatched morphism.
            if j < tower_work.len() && g == tower_work[j] { next[j + 1] = true; }
        }
        reachable = next;
    }

    reachable[tower_work.len()]
}

'''

if 'fn dispatch_projection_holds(' not in s:
    anchor = 'pub fn audit_execution_projection(\n'
    if anchor not in s:
        raise SystemExit('audit_execution_projection anchor not found; refusing unknown source state')
    s = s.replace(anchor, helper + anchor, 1)
    print('added carry-aware dispatch projection helper')
else:
    print('carry-aware dispatch projection helper already present')

old = '    if walk_work != tower_work {\n'
new = '    if !dispatch_projection_holds(&walk_work, &tower_work) {\n'
if old in s:
    s = s.replace(old, new, 1)
    print('changed tower audit to carry-aware projection check')
elif new in s:
    print('tower audit already uses carry-aware projection check')
else:
    raise SystemExit('expected tower-work comparison not found; refusing unknown source state')

path.write_text(s)
PY

echo
echo 'running focused morphism tests...'
cargo test --release morphism_factor::tests::instant_extractor_decomposes_and_factors --lib
cargo test --release morphism_factor::tests::frontier_arm_clears_a_balanced_semiprime_fast --lib
cargo test --release morphism_factor::tests::changed_work_is_rejected_even_when_a_type_is_factoring_complete --lib
