#!/usr/bin/env bash
set -euo pipefail
cd "${1:-.}"

python3 - <<'PY'
from pathlib import Path
p = Path('src/divisor_membrane.rs')
s = p.read_text()

if 'let ql_v = tape_to_u64(ql);' in s and 'exact completion in the narrowed interval' in s:
    print('divisor bridge exact-completion fix already present')
else:
    old = '''pub fn bridge_extendable_tape(n: &[char], m: u32, pl: &[char], _ql: &[char], k: u32, ph: &[char], qh: &[char], l: u32) -> bool {
    under(OP_BRIDGE, || {
        if k == 0 && l == 0 { return true; }
        if m > 31 { return false; }               // u64 window; NOT the old m<=10 gate
        let nv = tape_to_u64(n);
        let pl_v = tape_to_u64(pl);
        let ph_v = tape_to_u64(ph);
        let qh_v = tape_to_u64(qh);
        let shift = m - l;
        // top-l interval carried by the high partials
        let p_lo = ph_v << shift;
        let p_hi = ((ph_v + 1) << shift) - 1;
        let q_lo = qh_v << shift;
        let q_hi = ((qh_v + 1) << shift) - 1;
        // q = N/p lands in q's interval  <=>  p in [ceil(N/q_hi), floor(N/q_lo)]
        let lo = p_lo.max(div_ceil_u64(nv, q_hi.max(1)));
        let hi = p_hi.min(nv / q_lo.max(1));
        if lo > hi { return false; }
        // low congruence p = pl (mod 2^k): does the AP meet [lo, hi]?
        if k == 0 { return true; }
        let step = 1u64 << k;
        let rem = pl_v % step;
        let x0 = lo + ((rem + step - (lo % step)) % step);
        x0 <= hi
    })
}
'''
    new = '''pub fn bridge_extendable_tape(n: &[char], m: u32, pl: &[char], ql: &[char], k: u32, ph: &[char], qh: &[char], l: u32) -> bool {
    under(OP_BRIDGE, || {
        if k == 0 && l == 0 { return true; }
        if m > 31 { return false; }               // u64 window; NOT the old m<=10 gate
        let nv = tape_to_u64(n);
        let pl_v = tape_to_u64(pl);
        let ql_v = tape_to_u64(ql);
        let ph_v = tape_to_u64(ph);
        let qh_v = tape_to_u64(qh);
        let shift = m - l;
        // top-l interval carried by the high partials
        let p_lo = ph_v << shift;
        let p_hi = ((ph_v + 1) << shift) - 1;
        let q_lo = qh_v << shift;
        let q_hi = ((qh_v + 1) << shift) - 1;
        // q = N/p lands in q's interval  <=>  p in [ceil(N/q_hi), floor(N/q_lo)]
        let lo = p_lo.max(div_ceil_u64(nv, q_hi.max(1)));
        let hi = p_hi.min(nv / q_lo.max(1));
        if lo > hi { return false; }

        // Exact completion in the narrowed interval.  The old bridge stopped at
        // "the p-residue AP intersects [lo,hi]", which admitted interval-only
        // false positives (for N=143, k=l=2 it admitted p=15 although 15 !| 143).
        // Preserve the solution-blind search, but require BOTH low residues and
        // the exact product before reporting the partial state extendable.
        let step = if k == 0 { 1 } else { 1u64 << k };
        let mut p = if k == 0 {
            lo
        } else {
            let rem = pl_v % step;
            lo + ((rem + step - (lo % step)) % step)
        };

        while p <= hi {
            if p != 0 && nv % p == 0 {
                let q = nv / p;
                let q_low_ok = k == 0 || q % step == ql_v % step;
                if q_low_ok && q >= q_lo && q <= q_hi {
                    return true;
                }
            }
            match p.checked_add(step) {
                Some(next) if next > p => p = next,
                _ => break,
            }
        }
        false
    })
}
'''
    if old not in s:
        raise SystemExit('ERROR: expected bridge_extendable_tape body not found; current file differs from known source')
    p.write_text(s.replace(old, new, 1))
    print('tightened bridge_extendable_tape to exact completion semantics')
PY

echo
echo 'running divisor membrane tests...'
cargo test --release divisor_membrane::tests::membrane_143_closes_over_tapes --lib
cargo test --release divisor_membrane::tests::membrane_35_closes_over_tapes --lib
