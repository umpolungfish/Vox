"""Apply the reviewed coherent factor interface to G-mOMonadOS."""
from pathlib import Path
import hashlib
import json
ROOT = Path(__file__).resolve().parents[1]
KERNEL = ROOT.parent / 'G-mOMonadOS'
changes = {}
def replace(path, old, new):
    text = path.read_text()
    if text.count(old) != 1:
        raise RuntimeError(f'Expected one interface seam in {path}')
    changes[path] = text.replace(old, new)
replace(KERNEL / 'Cargo.toml', 'vox_core = { path = "vendor/vox",',
        'vox_core = { path = "../Vox",')
replace(KERNEL / 'src/lib.rs', 'pub mod phase_unbraid;',
        'pub mod phase_unbraid;\npub mod factor_phase;')
replace(KERNEL / 'src/main.rs', 'mod shor_qft;', 'mod shor_qft;\nmod factor_phase;')
replace(KERNEL / 'src/repl.rs', '            "shor" => {', '''            "factor_phase" => {
                match crate::factor_phase::execute_baked() {
                    Ok(output) => sprint!("{}", output),
                    Err(error) => sprintln!("factor_phase: {}", error),
                }
            }
            "shor" => {''')
replace(KERNEL / 'src/menu.rs', '    MenuItem { name: "fibqc",',
        '    MenuItem { name: "factor_phase", cmd: "factor_phase", desc: "Exact folded coherent CPU simulation and pair measurement over baked IMASM source and quantile", example: "factor_phase", submenu: None },\n    MenuItem { name: "fibqc",')
new = KERNEL / 'src/factor_phase.rs'
if new.exists():
    raise RuntimeError('Preserve the existing kernel factor-phase source')
changes[new] = (ROOT / 'measurements/gm_factor_phase.rs').read_text()
example = KERNEL / 'examples/factor_phase_baked.rs'
if example.exists():
    raise RuntimeError('Preserve the existing baked kernel example')
changes[example] = '''//! Silent prepared coherent pair measurement using the kernel interface.
#![deny(warnings)]
fn main() {
    let output=g_momonados::factor_phase::execute_baked()
        .expect("prepared coherent factor measurement");
    use std::io::{self,Write};
    io::stdout().lock().write_all(output.as_bytes()).unwrap();
}
'''
backup = ROOT / 'membranes/folded_quantum_pair_2026-09-30/kernel_before'
backup.mkdir(exist_ok=False)
for path in changes:
    if path.exists():
        saved=backup / path.relative_to(KERNEL)
        saved.parent.mkdir(parents=True,exist_ok=True)
        saved.write_bytes(path.read_bytes())
(backup / 'manifest.json').write_text(json.dumps({str(p):hashlib.sha256(p.read_bytes()).hexdigest()
    for p in changes if p.exists()},indent=2)+'\n')
for path,text in changes.items():
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(text)
(KERNEL / 'commit.txt').write_text('I connect prepared coherent factor-pair measurement to the canonical Vox register and oracle.\n\nI expose the baked factor_phase interface through the kernel, its quantum menu and a silent prepared example.\n')
print('Applied the reviewed kernel dependency, source, menu and baked execution seams.')
