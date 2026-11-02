"""Shared geometry equations from counting the retained source edge relation."""
from collections import Counter, defaultdict
from fractions import Fraction
from math import prod
import json
import hashlib
from pathlib import Path
import sys
import numpy as np
from whole_algebraic_readout import Whole


def compile_equations(relation, source):
    masses=[prod(len(p['edge_states']) for p in e['ports']) for e in relation['events']]
    total=sum(masses)
    pair_counts=[[0]*256 for _ in relation['pair_ports']]
    for event,mass in zip(relation['events'],masses):
        hist=[Counter(p['edge_states']) for p in event['ports']]
        lengths=[len(p['edge_states']) for p in event['ports']]
        for pi,(i,j) in enumerate(relation['pair_ports']):
            outside=mass//(lengths[i]*lengths[j]);counts=pair_counts[pi]
            for left,lc in hist[i].items():
                for right,rc in hist[j].items():counts[16*left+right]+=outside*lc*rc
    responses=defaultdict(lambda:[Fraction(0)]*3);channels=defaultdict(Fraction)
    for (i,j),counts in zip(relation['pair_ports'],pair_counts):
        if sum(counts)!=total:raise ValueError('joint geometry contraction lost source mass')
        for state,count in enumerate(counts):
            if not count:continue
            weight=Fraction(count,total);contexts=relation['sixteen3_gate_table'][state]
            for order in range(3):
                a,b=contexts[2*order:2*order+2];k=first=second=0
                for mask in (a,b):
                    p=((mask>>1)&1)+((mask>>3)&1);n=((mask>>2)&1)+((mask>>3)&1)
                    k+=p+n;first+=n*(p+n);second+=n*(p+n)**2
                if k:
                    row=responses[i,a or 15,j,b or 15]
                    for axis,value in enumerate((k,first,second)):row[axis]+=weight*value
            family=source['orbital_transfer']['source_and_closed_families'][state][1]
            for member in range(16):
                if family&(1<<member):channels[i,j,member//4,member%4]+=weight
    return {'source_sha256':relation['source_sha256'],'source_relation':'tc5b_joint_edge_relation.json',
        'joint_assignment_count':str(total),
        'responses':[[*key,*[[str(x.numerator),str(x.denominator)] for x in values]] for key,values in responses.items()],
        'orbital_channels':[[*key,str(v.numerator),str(v.denominator)] for key,v in channels.items()],
        'full_chain_lattice_fold_closed':False}


def numerical_operator(equations, source):
    w=Whole.__new__(Whole);w.n=20;w.dimension=17
    angles=np.array(source['torsions'][3:]);w.charts=(np.cos(angles)<0).tolist()
    w.seed=(np.sin(angles)/(1+np.abs(np.cos(angles)))).tolist()
    w.responses=[(*r[:4],*(Fraction(int(v[0]),int(v[1])) for v in r[4:])) for r in equations['responses']]
    w.channels=[]
    for i,j,a,b,num,den in equations['orbital_channels']:
        p=(a&1)+(b&1);n=(a>>1)+(b>>1)
        if p+n:w.channels.append((i,a,j,b,Fraction(int(num),int(den)),p,n))
    keys={(i,a) for i,a,j,b,*_ in w.responses}|{(j,b) for i,a,j,b,*_ in w.responses}
    keys|={(i,1<<a) for i,a,j,b,*_ in w.channels}|{(j,1<<b) for i,a,j,b,*_ in w.channels}
    w.center_keys=sorted(keys);index={k:i for i,k in enumerate(w.center_keys)}
    w.response_left=np.array([index[i,a] for i,a,j,b,*_ in w.responses]);w.response_right=np.array([index[j,b] for i,a,j,b,*_ in w.responses])
    w.response_coefficients=np.asarray([r[4:] for r in w.responses],float)
    w.response_coefficients_extended=np.asarray([[np.longdouble(v.numerator)/np.longdouble(v.denominator)
        for v in row[4:]] for row in w.responses],dtype=np.longdouble)
    w.channel_left=np.array([index[i,1<<a] for i,a,j,b,*_ in w.channels]);w.channel_right=np.array([index[j,1<<b] for i,a,j,b,*_ in w.channels])
    w.channel_values=np.asarray([r[4:] for r in w.channels],float)
    caps={tuple(c['source']):c['live_preferred_distance'] for c in source['glut_work']['cells']}
    w.channel_preference_caps=np.asarray([caps.get((i,j,a,b),n) for i,a,j,b,count,p,n in w.channels])
    cells={tuple(c['source']):c for c in source['glut_work']['cells']}
    w.channel_preference_cap_squares=[]
    for i,a,j,b,count,p,n in w.channels:
        cell=cells.get((i,j,a,b))
        z=Fraction(cell['release_coordinate']) if cell else Fraction(n)
        w.channel_preference_cap_squares.append(Fraction(n,p+n)*(p*n+z*z))
    w.active_orbital_force_channels=w.channels
    w.carried_channels=[]
    return w


if __name__=='__main__':
    relation_path,source_path,output=map(Path,sys.argv[1:4])
    relation=json.loads(relation_path.read_text());source=json.loads(source_path.read_text())
    equations=compile_equations(relation,source)
    equations['carried_bank']=source['glut_work']
    equations['current_carrier_sha256']=hashlib.sha256(source_path.read_bytes()).hexdigest()
    output.write_text(json.dumps(equations,indent=2)+'\n')
    w=numerical_operator(equations,source)
    from scipy.optimize import root
    result=root(lambda t:w.float_energy_gradient(t,angular=True)[1],source['torsions'][3:],options={'maxfev':600})
    energy,gradient=w.float_energy_gradient(result.x,angular=True)
    charts=(np.cos(result.x)<0).tolist()
    parameters=np.sin(result.x)/(1+np.abs(np.cos(result.x)))
    witness={'parameters':parameters.tolist(),'charts':charts,'angles':result.x.tolist(),'energy':energy,
        'gradient_inf':float(np.max(np.abs(gradient))),'solver_success':bool(result.success),
        'solver_message':str(result.message),'source_sha256':equations['source_sha256'],
        'full_chain_lattice_fold_closed':False}
    output.with_suffix('.witness.json').write_text(json.dumps(witness,indent=2)+'\n')
    print(witness['solver_message'],'gradient',witness['gradient_inf'])
