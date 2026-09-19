#!/usr/bin/env bash
set -euo pipefail

cd "${1:-.}"

[[ -f Cargo.toml && -d src ]] || { echo 'run from Vox repo root (or pass it as arg)' >&2; exit 2; }

# The previous patch must already have installed these two semantic pieces.
[[ -f src/factor_extract.rs ]] || {
  echo 'src/factor_extract.rs is missing; use the supplied factor_extract_complete.rs first' >&2
  exit 3
}

grep -q 'pub fn relaxed_equivalent_with_witness' src/trace_algebra.rs || {
  echo 'trace_algebra.rs does not contain the relaxed witness relation yet' >&2
  echo 'Do not reapply the whole patch; patch only trace_algebra.rs against your current file.' >&2
  exit 4
}

# Export the module exactly once.
if ! grep -q '^pub mod factor_extract;$' src/lib.rs; then
  python3 - <<'PY'
from pathlib import Path
p = Path('src/lib.rs')
s = p.read_text()
anchor = 'pub mod trace_algebra;\n'
if anchor not in s:
    raise SystemExit('could not find pub mod trace_algebra; in src/lib.rs')
s = s.replace(anchor, anchor + 'pub mod factor_extract;\n', 1)
p.write_text(s)
PY
  echo 'added pub mod factor_extract; to src/lib.rs'
else
  echo 'src/lib.rs already exports factor_extract'
fi

# Add CLI exposure only if the previous patch did not already do it.
if ! grep -q 'Some("extract-factor")' src/main.rs; then
  python3 - <<'PY'
from pathlib import Path
p = Path('src/main.rs')
s = p.read_text()
usage_anchor = '    eprintln!("  vox morphism-factor <native-numeral-word>   factor entirely over IMASM tapes");\n'
usage_line = '    eprintln!("  vox extract-factor <factor-carrier-word>    passive ≡c extraction from an already factor-bearing trace");\n'
if usage_anchor not in s:
    raise SystemExit('could not locate morphism-factor usage line in src/main.rs')
s = s.replace(usage_anchor, usage_anchor + usage_line, 1)

arm_anchor = '''        Some("verify") => {\n'''
arm = '''        Some("extract-factor") => {\n            if args.len() != 2 {\n                eprintln!("vox extract-factor <factor-carrier-word>");\n                1\n            } else {\n                let word: Vec<char> = args[1].chars().collect();\n                match ::vox::factor_extract::extract_word(&word) {\n                    Ok(readout) => {\n                        println!("{} x {}", readout.p, readout.q);\n                        println!("normal-form {}", readout.normal_form.iter().collect::<String>());\n                        println!("transforms {}", readout.transforms);\n                        0\n                    }\n                    Err(e) => { eprintln!("{}", e); 2 }\n                }\n            }\n        }\n'''
if arm_anchor not in s:
    raise SystemExit('could not locate verify command arm in src/main.rs')
s = s.replace(arm_anchor, arm + arm_anchor, 1)
p.write_text(s)
PY
  echo 'added extract-factor CLI arm to src/main.rs'
else
  echo 'src/main.rs already contains extract-factor CLI arm'
fi

echo
echo 'integration state:'
grep -n '^pub mod factor_extract;$' src/lib.rs
grep -n 'pub fn relaxed_equivalent_with_witness' src/trace_algebra.rs
grep -n 'Some("extract-factor")' src/main.rs

echo
echo 'building...'
cargo build --release
cargo test --release factor_extract
