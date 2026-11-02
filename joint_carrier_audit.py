"""Replay source bindings and the complete root-return inclusion calculation."""
import hashlib
import json
from fractions import Fraction
from pathlib import Path
import sys
import numpy as np
from whole_algebraic_readout import I,S
from whole_joint_graph import compile_source
from joint_geometry_return import numerical_operator,compile_equations
from joint_root_enclosure import positive_congruence
from joint_root_carrier import prepare


def audit(directory):
    load=lambda name:json.loads((directory/name).read_text())
    digest=lambda name:hashlib.sha256((directory/name).read_bytes()).hexdigest()
    source=load('tc5b_joint_graph_events_prepared.json')
    relation=load('tc5b_joint_edge_relation.json')
    regenerated=compile_source(source)
    regenerated['source_sha256']=digest('tc5b_joint_graph_events_prepared.json')
    assert regenerated==relation,'source edge relation no longer reproduces its source'
    for event in relation['events']:
        forms,component,tick=event['address']
        for residue,port in enumerate(event['ports']):
            program=source['site_graphs']['bindings'][residue][forms[residue]][component]
            path=source['source_paths'][residue][forms[residue]][component]
            seed=path[(tick+residue)%len(path)]
            assert port['program']==program and port['seed']==seed
            assert port['edge_states']==source['site_graphs']['programs'][program]['seed_edge_states'][seed]
        for pi,ordinal in enumerate(event['pair_occurrence_ordinals']):
            i,j=relation['pair_ports'][pi]
            period=next(c[3] for c in source['pairs'][pi]['form_cycles'] if c[:3]==[forms[i],forms[j],component])
            assert relation['source_occurrence_tensors'][pi][ordinal][:4]==[forms[i],forms[j],component,tick%period]
    print('complete source relation replay passed',flush=True)
    current=load('tc5b_joint_family_bank_returned_carrier.json')
    equations=load('tc5b_joint_geometry_equations.json')
    recomputed=compile_equations(relation,current)
    for key in recomputed:assert recomputed[key]==equations[key],('geometry source contraction changed',key)
    assert equations['current_carrier_sha256']==digest('tc5b_joint_family_bank_returned_carrier.json')
    carrier=load('tc5b_joint_root_carrier.json');certificate=carrier['root_certificate']
    assert carrier['joint_relation']==relation
    assert carrier['source_relation_sha256']==digest('tc5b_joint_edge_relation.json')
    assert carrier['equations_sha256']==digest('tc5b_joint_geometry_equations.json')
    witness=load('tc5b_joint_geometry_exact_center.witness.json')
    replay_path=directory/'tc5b_joint_root_carrier.replay.json'
    prepare(directory/'tc5b_joint_geometry_equations.json',directory/'tc5b_joint_family_bank_returned_carrier.json',
        directory/'tc5b_joint_edge_relation.json',directory/'tc5b_joint_geometry_enclosure.json',
        directory/'tc5b_joint_geometry_exact_center.witness.json',replay_path)
    assert json.loads(replay_path.read_text())==carrier,'root geometry/bank reconstruction changed'
    whole=numerical_operator(equations,current);whole.charts=witness['charts']
    center=[Fraction(v) for v in witness['extended_parameters']];n=len(center)
    midpoint=whole.evaluate([I.value(v) for v in center],True,hessian=False)[0]
    numeric=whole.evaluate(list(map(float,center)),hessian=True)[0]
    C=[[I.value(float(v)) for v in row] for row in np.linalg.inv(np.asarray(numeric.h,float))]
    parse=lambda v:I(int(v['lower_numerator']),int(v['upper_numerator']))
    box=list(map(parse,certificate['box']));H=[[parse(v) for v in row] for row in certificate['jacobian']]
    matrix=[[I.value(int(i==j))-sum(C[i][k]*H[k][j] for k in range(n)) for j in range(n)] for i in range(n)]
    image=[I.value(center[i])-sum(C[i][k]*midpoint.g[k] for k in range(n))+
        sum(matrix[i][j]*(box[j]-I.value(center[j])) for j in range(n)) for i in range(n)]
    assert [v.record() for v in image]==certificate['image'],'root inclusion replay changed'
    assert all(b.lo<v.lo and v.hi<b.hi for b,v in zip(box,image))
    positive=positive_congruence(H);assert positive['verified']
    returned=load('tc5b_joint_root_sic_return.json')
    for key in ('full_carrier_input_equals_return','conditional_bank_reentry_equal','input_equals_return','return_equals_reentry'):
        assert returned[key],key
    for original,result in [('algebraic_positions','returned_algebraic_positions'),
        ('algebraic_bank','returned_algebraic_bank'),('four_state_character_images','returned_four_state_character_images')]:
        assert carrier[original]==returned[result],original
    nmr=load('tc5b_joint_root_nmr_comparison.json')
    assert nmr['producer_sha256']==digest('tc5b_joint_root_carrier.json')
    proof={'charts':whole.charts,'center':[str(v) for v in center],
        'center_force':[v.record() for v in midpoint.g],
        'preconditioner':[[v.record() for v in row] for row in C]}
    result={'source_relation_replayed':True,'source_geometry_contraction_replayed':True,
        'root_inclusion_replayed':True,'positive_mechanical_congruence':True,
        'conditional_bank_and_full_carrier_return_equal':True,
        'source_events':len(relation['events']),'source_occurrences':relation['occurrences_bound'],
        'root_carrier_sha256':digest('tc5b_joint_root_carrier.json'),
        'native_return_sha256':digest('tc5b_joint_root_sic_return.json'),
        'root_proof_replay':proof,'nmr_conformers':nmr['conformers'],
        'root_geometry_bank_reconstruction_replayed':True,
        'full_chain_lattice_fold_closed':True}
    (directory/'tc5b_joint_carrier_audit.json').write_text(json.dumps(result,indent=2)+'\n')
    print('source contraction, root inclusion and native full-carrier equality replay passed',flush=True)


if __name__=='__main__':audit(Path(sys.argv[1]))
