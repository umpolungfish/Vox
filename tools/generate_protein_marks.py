"""Prepare canonical Rebis residue/form records for the native Vox compiler."""
import hashlib
import json
import os
import re
import subprocess
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT.parent/'red-hot_rebis'))
from rhr_p4rky.protein_glut import self_marks, _NAMES

def native_type_programs():
    mill=ROOT.parent/'p4rakernel/p4ramill'
    alphabet=ROOT.parent/'MoDoT/ask_native/src/imasm.rs'
    block=alphabet.read_text().split('const GLYPH_TYPE_NAME:',1)[1].split('];',1)[0]
    names=dict(re.findall(r'\("([^"]+)",\s*"([^"]+)"\)',block))
    sources=[alphabet,mill/'_scripts/export_primitive_programs.lean',
             mill/'Imscribing/Primitives/Core.lean',
             *sorted((mill/'Imscribing/Primitives/Types').glob('PrimitiveType*.lean'))]
    for path in sources:print('cargo:rerun-if-changed='+str(path))
    binding={str(p.relative_to(ROOT.parent)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sources}
    cache=ROOT/'measurements/native_primitive_programs';cache.mkdir(parents=True,exist_ok=True)
    path=cache/'current.json'
    snapshot=os.environ.get('VOX_PRIMITIVE_PROGRAMS_FILE')
    if snapshot:
        snapshot=Path(snapshot).resolve()
        prior=json.loads(snapshot.read_text())
        print('cargo:rerun-if-changed='+str(snapshot))
    else:
        prior=json.loads(path.read_text()) if path.exists() else {}
    prior_sources=prior.get('sources',{})
    fresh=(all(prior_sources.get(k)==v for k,v in binding.items()) and
           all((ROOT.parent/k).exists() and hashlib.sha256((ROOT.parent/k).read_bytes()).hexdigest()==v
               for k,v in prior_sources.items()))
    if snapshot or fresh:
        exported=prior['export']
    else:
        # Rust consumes the native programs in the existing Lean build. Lean
        # source edits do not authorize Cargo to rebuild its dependency graph.
        # Record compiled modules separately from the source text being read.
        run=subprocess.run(['lake','env','lean','--run','_scripts/export_primitive_programs.lean'],
                           cwd=mill,capture_output=True,text=True)
        if run.returncode:
            raise RuntimeError('Cannot export the existing compiled Lean programs. '
                'Build the affected Lean module explicitly before rebuilding Vox.\n'+run.stdout+run.stderr)
        exported=json.loads(run.stdout)
    programs={p['name']:p for p in exported['programs']}
    assert len(programs)==len(names)==49
    for glyph,name in names.items():
        program=programs[name]
        assert program['word'] and set(program['word'])<=set('⊢⊣≻≺⋈⊙∈∋⊤⊥⊞⊡')
        for source in [program['source'],*program.get('dependencies',[])]:
            native=mill/source
            binding[str(native.relative_to(ROOT.parent))]=hashlib.sha256(native.read_bytes()).hexdigest()
            print('cargo:rerun-if-changed='+str(native))
    compiled={}
    for relative in ([] if snapshot else list(binding)):
        native=ROOT.parent/relative
        if native.suffix!='.lean':continue
        module=mill/'.lake/build/lib/lean'/native.relative_to(mill).with_suffix('.olean')
        if module.exists():
            key=str(module.relative_to(ROOT.parent))
            compiled[key]=hashlib.sha256(module.read_bytes()).hexdigest()
            print('cargo:rerun-if-changed='+str(module))
    binding.update(compiled)
    if snapshot:
        # This immutable instrument return supplies the programs. Current Lean
        # text is separately observed above; its rebuild cannot replace them.
        binding[str(snapshot.relative_to(ROOT.parent))]=hashlib.sha256(snapshot.read_bytes()).hexdigest()
    else:
        path.write_text(json.dumps(dict(sources=binding,export=exported,
            export_origin='existing compiled Lean declarations; source text and compiled modules bound separately'),
            ensure_ascii=False)+'\n')
    return {glyph:programs[name] for glyph,name in names.items()},binding

programs,type_bindings=native_type_programs()

def literal(value):
    return json.dumps(value, ensure_ascii=False)

rows = []
print('cargo:rerun-if-changed='+str(ROOT.parent/'red-hot_rebis/rhr_p4rky/protein_glut.py'))
for aa in _NAMES:
    for mark in self_marks(aa):
        print('cargo:rerun-if-changed='+str(ROOT.parent/mark['source']))
        components=[dict(axis='self-mark',word=mark['word'],source=mark['source'])]
        values=mark['tuple'].strip('⟨⟩')
        assert len(values)==len('⊢⊣≻≺⋈⊤∈∋⊙⊥⊞⊡')
        for axis,value in zip('⊢⊣≻≺⋈⊤∈∋⊙⊥⊞⊡',values):
            program=programs[value]
            components.append(dict(axis=axis,value=value,word=program['word'],source=program['source'],
                                   pairs=program['pairs'],program=program['name']))
        mark['native_components']=components
        parts='&['+','.join(literal(p['word']) for p in components)+']'
        pairs='&['+','.join('Some(&['+','.join('(%d,%d)'%tuple(pair) for pair in p['pairs'])+'])'
                            if 'pairs' in p else 'None' for p in components)+']'
        fields=tuple(literal(x) for x in (aa,json.dumps(mark,ensure_ascii=False),mark['word'],
                                         mark['tuple'],mark['artifact_id'],mark['sha256']))
        rows.append(('MarkSpec { aa: %s, record: %s, word: %s, tuple: %s, artifact: %s, sha256: %s, native_parts: %s, native_pairs: %s },') % (*fields,parts,pairs))
bindings = {name: hashlib.sha256((ROOT/'src'/name).read_bytes()).hexdigest()
    for name in ('protein_compile.rs','protein_glut.rs','protein.rs','genetic.rs','genetic_table.rs',
                         'belnap.rs','belnap_relation.rs','phase_word.rs',
                         'fold3d.rs','peptide_geometry.rs','peptide_closure.rs',
                         'protein_relax.rs','protein_lattice.rs','protein_native.rs','protein_records.rs','protein_sites.rs','protein_orbital.rs',
                         'protein_site_geometry.rs','protein_glut_work.rs','sic.rs','protein_electronic.rs',
                         'protein_electronic_sector.rs','protein_atom_graph.rs','protein_pair_density.rs')}
for name in bindings:
    print('cargo:rerun-if-changed='+str(ROOT/'src'/name))
for relative in ('src/Lean/Sixteen3.lean',
                 'p4ramill/Imscribing/GeneticCode.lean',
                 'p4ramill/Imscribing/Primitives/Crystal.lean',
                 'p4ramill/Imscribing/Biochem/PrimaLogicaMechanicalGlut.lean',
                 'p4ramill/Imscribing/Biochem/NativeOrbitalTransfer.lean',
                 'p4ramill/Imscribing/Biochem/NativeShellComposition.lean',
                 'p4ramill/Imscribing/Biochem/NativeEnsembleRelease.lean',
                 'p4ramill/Imscribing/Biochem/NativeEnsembleSIC.lean',
                 'p4ramill/Imscribing/Biochem/NativeEnsembleSuccessor.lean',
                 'p4ramill/Imscribing/Paraconsistent/OrbitalBelnap.lean'):
    path = ROOT.parent/'p4rakernel'/relative
    print('cargo:rerun-if-changed='+str(path))
    bindings['p4rakernel/'+relative] = hashlib.sha256(path.read_bytes()).hexdigest()
bindings.update(type_bindings)
for name in ('Cargo.toml','Cargo.lock'):
    path=ROOT/name
    bindings['Vox/'+name]=hashlib.sha256(path.read_bytes()).hexdigest()
    print('cargo:rerun-if-changed='+str(path))
genetic_generator=ROOT/'gen_genetic_table.py'
bindings['Vox/gen_genetic_table.py']=hashlib.sha256(genetic_generator.read_bytes()).hexdigest()
print('cargo:rerun-if-changed='+str(genetic_generator))
for name in ('frame.rs','wh.rs','carrier.rs'):
    relative='G-mOMonadOS/src/sic/'+name
    path=ROOT.parent/relative
    bindings[relative]=hashlib.sha256(path.read_bytes()).hexdigest()
    print('cargo:rerun-if-changed='+str(path))
for name in ('check.rs','flow.rs','classic.rs','imasm16_3.rs'):
    relative='MoDoT/imasm_core/src/'+name
    path=ROOT.parent/relative
    bindings[relative]=hashlib.sha256(path.read_bytes()).hexdigest()
    print('cargo:rerun-if-changed='+str(path))
output = 'const MARKS: &[MarkSpec] = &[\n'+'\n'.join(rows)+'\n];\n'
output += 'const SOURCE_BINDING: &str = '+literal(json.dumps(bindings))+';\n'
Path(sys.argv[1]).write_text(output)
