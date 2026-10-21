"""Prepare canonical Rebis residue/form records for the native Vox compiler."""
import hashlib
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT.parent/'red-hot_rebis'))
from rhr_p4rky.protein_glut import self_marks, _NAMES

def literal(value):
    return json.dumps(value, ensure_ascii=False)

rows = []
print('cargo:rerun-if-changed='+str(ROOT.parent/'red-hot_rebis/rhr_p4rky/protein_glut.py'))
for aa in _NAMES:
    for mark in self_marks(aa):
        print('cargo:rerun-if-changed='+str(ROOT.parent/mark['source']))
        rows.append('MarkSpec { aa: %s, record: %s, word: %s, tuple: %s, artifact: %s, sha256: %s },' % tuple(
            literal(x) for x in (aa, json.dumps(mark, ensure_ascii=False), mark['word'], mark['tuple'], mark['artifact_id'], mark['sha256'])))
bindings = {name: hashlib.sha256((ROOT/'src'/name).read_bytes()).hexdigest()
            for name in ('phase_word.rs','fold3d.rs','peptide_geometry.rs','peptide_closure.rs')}
for name in bindings:
    print('cargo:rerun-if-changed='+str(ROOT/'src'/name))
output = 'const MARKS: &[MarkSpec] = &[\n'+'\n'.join(rows)+'\n];\n'
output += 'const SOURCE_BINDING: &str = '+literal(json.dumps(bindings))+';\n'
Path(sys.argv[1]).write_text(output)
