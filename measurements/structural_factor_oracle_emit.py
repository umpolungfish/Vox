"""Execute baked factor-phase controls and verify complete IMASM recovery."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import time
ROOT = Path(__file__).resolve().parents[1]
FOLDER = ROOT / 'membranes/structural_factor_phase_2026-09-30_verified'
FOLDER.mkdir(exist_ok=False)
for name in ['src/structural_factor_oracle.rs', 'src/structural_quantum_executor.rs',
             'examples/factor_phase_baked_control.rs']:
    shutil.copy2(ROOT / name, FOLDER / Path(name).name)
elf = FOLDER / 'payload.elf'
shutil.copy2(ROOT / 'target/x86_64-unknown-linux-musl/release/examples/factor_phase_baked_control', elf)
start = time.monotonic()
with (FOLDER / 'native.stdout').open('w') as out, (FOLDER / 'native.stderr').open('w') as err:
    subprocess.run([str(elf)], stdout=out, stderr=err, timeout=60, check=True)
elapsed = time.monotonic() - start
assert not (FOLDER / 'native.stderr').read_bytes()
vox = ROOT / 'target/release/vox'
expected = [subprocess.check_output([str(vox), 'numeral', str(n)], text=True, timeout=60).strip()
            for n in [15, 1, 0, 0]]
assert (FOLDER / 'native.stdout').read_text().splitlines() == expected
module = Path(str(elf) + '.imasm')
glyphs = FOLDER / 'payload.glyphs'
recovered = FOLDER / 'recovered.imasm'
with (FOLDER / 'emission.stdout').open('w') as log:
    subprocess.run([str(vox), 'imasm', str(elf)], stdout=subprocess.DEVNULL,
                   stderr=log, check=True, timeout=60)
    subprocess.run([str(vox), 'glyphs', str(module), str(glyphs)], stdout=log,
                   stderr=log, check=True, timeout=60)
    subprocess.run([str(vox), 'unglyphs', str(glyphs), str(recovered)], stdout=log,
                   stderr=log, check=True, timeout=60)
assert module.read_bytes() == recovered.read_bytes()
alphabet = set('⊢⊣≻≺⋈⊤∈∋⊙⊥⊞⊡')
assert set(glyphs.read_text()) <= alphabet
assert set((FOLDER / 'native.stdout').read_text().replace('\n', '')) <= alphabet
row = dict(instrument='baked factor-phase basis controls', source_n='15',
           branches=['3*5 marked', '3*4 unmarked', '1*15 trivial unmarked'],
           factor_search_performed=False, phase_and_workspace_round_trip=True,
           native_seconds=elapsed, complete_imasm_recovery=True, glyph_alphabet_only=True,
           binary_sha256=hashlib.sha256(elf.read_bytes()).hexdigest())
(FOLDER / 'verification.json').write_text(json.dumps(row, indent=2) + '\n')
print(json.dumps(row))
