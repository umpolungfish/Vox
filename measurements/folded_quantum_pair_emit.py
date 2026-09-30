"""Recover the complete carrier of the verified coherent CPU execution."""
from pathlib import Path
import hashlib
import json
import shutil
import subprocess
ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'membranes/folded_quantum_pair_2026-09-30'
elf = FOLDER / 'payload.elf'
module = Path(str(elf) + '.imasm')
glyphs = FOLDER / 'payload.glyphs'
recovered = FOLDER / 'recovered.imasm'
if glyphs.exists():
    raise RuntimeError('Preserve the preceding carrier emission')
vox = ROOT / 'target/release/vox'
with (FOLDER / 'emission.stdout').open('w') as log:
    subprocess.run([str(vox), 'imasm', str(elf)], stdout=subprocess.DEVNULL,
                   stderr=log, check=True, timeout=60)
    subprocess.run([str(vox), 'glyphs', str(module), str(glyphs)], stdout=log,
                   stderr=log, check=True, timeout=60)
    subprocess.run([str(vox), 'unglyphs', str(glyphs), str(recovered)], stdout=log,
                   stderr=log, check=True, timeout=60)
assert module.read_bytes() == recovered.read_bytes()
assert set(glyphs.read_text()) <= set('⊢⊣≻≺⋈⊤∈∋⊙⊥⊞⊡')
source = FOLDER / 'source'
source.mkdir(exist_ok=False)
paths = [ROOT / 'src/folded_quantum_register.rs', ROOT / 'src/structural_factor_oracle.rs',
         ROOT / 'src/structural_quantum_executor.rs', ROOT / 'src/fixed_point_quantum_phase.rs',
         ROOT.parent / 'G-mOMonadOS/src/factor_phase.rs']
for path in paths:
    shutil.copy2(path, source / path.name)
row = dict(complete_module_recovery=True, glyph_alphabet_only=True,
           binary_sha256=hashlib.sha256(elf.read_bytes()).hexdigest(),
           source_sha256={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths},
           vm_execution_performed=False)
(FOLDER / 'emission_verification.json').write_text(json.dumps(row, indent=2) + '\n')
print('Complete coherent factor carrier emission and byte-identical recovery pass.')
