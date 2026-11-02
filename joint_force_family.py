"""Native frame derivatives lifted over the shared joint source variables."""
import json
from pathlib import Path
import sys
import numpy as np
from collections import Counter
from fractions import Fraction
from math import prod


def contract_complete_edge_family(relation, forces):
    """Count concrete edge assignments once, retaining the source relation.

    Unobserved residue edge variables contribute their domain cardinalities.
    This is counting elimination on the explicit edge-domain relation, not an
    inference of a joint probability law from pair-state histograms.
    """
    if relation['source_sha256'] != forces['source_sha256']:
        raise ValueError('joint force contraction differs from its source')
    masses=[prod(len(p['edge_states']) for p in e['ports']) for e in relation['events']]
    total=sum(masses); cache={}; returned=np.zeros(20)
    tables=[np.asarray(t['state_torsion_response']) for t in forces['pair_force_tables']]
    unresolved=[{row[0] for row in t['unresolved_entries']} for t in forces['pair_force_tables']]
    event_rows=[]
    for event,mass in zip(relation['events'],masses):
        hist=[tuple(sorted(Counter(p['edge_states']).items())) for p in event['ports']]
        response=np.zeros(20)
        for pair_index,(i,j) in enumerate(relation['pair_ports']):
            key=(pair_index,hist[i],hist[j])
            if key not in cache:
                value=np.zeros(20);denominator=len(event['ports'][i]['edge_states'])*len(event['ports'][j]['edge_states'])
                for left,lc in hist[i]:
                    for right,rc in hist[j]:
                        if 16*left+right in unresolved[pair_index]:
                            raise ValueError('source-supported joint force remains unresolved')
                        value+=float(Fraction(lc*rc,denominator))*tables[pair_index][16*left+right]
                cache[key]=value
            response+=cache[key]
        returned+=float(Fraction(mass,total))*response
        event_rows.append({'address':event['address'],'joint_assignment_count':str(mass)})
    return {'schema':'joint-edge-counting-force-return/v1','source_sha256':relation['source_sha256'],
        'source_relation_retained':True,'joint_assignment_count':str(total),'events':event_rows,
        'normalized_counting_torsion_response':returned.tolist(),
        'contraction':'each concrete tuple of source edge addresses counted once',
        'arithmetic':'exact integer source multiplicities; binary64 force readout',
        'full_chain_lattice_fold_closed':False}


def compile_forces(relation, source, native):
    sites=np.asarray(source['spatial_sites']['positions'])
    jacobian=np.asarray(native['native_site_torsion_pullback'])
    bank={tuple(c['source']):c for c in source['glut_work']['cells']}
    centers=np.empty((len(sites),16,3)); derivatives=np.empty((len(sites),16,3,len(sites)))
    for i in range(len(sites)):
        for mask in range(16):
            chosen=[v for v in range(4) if mask&(1<<v)] or list(range(4))
            centers[i,mask]=sites[i,chosen].mean(axis=0)
            derivatives[i,mask]=jacobian[i,chosen].mean(axis=0)
    tables=[]
    for i,j in relation['pair_ports']:
        rows=[]; unresolved=[]
        for state,contexts in enumerate(relation['sixteen3_gate_table']):
            torque=np.zeros(len(sites))
            for order in range(3):
                a,b=contexts[2*order:2*order+2]; k=first=0
                for mask in (a,b):
                    p=((mask>>1)&1)+((mask>>3)&1); n=((mask>>2)&1)+((mask>>3)&1)
                    k+=p+n;first+=n*(p+n)
                delta=centers[i,a]-centers[j,b]; d=np.linalg.norm(delta)
                if d==0 and first:unresolved.append([state,'context',order]);continue
                torque+=(k-first/d if d else k)*delta@(derivatives[i,a]-derivatives[j,b])
            family=source['orbital_transfer']['source_and_closed_families'][state][1]
            for member in range(16):
                if not family&(1<<member):continue
                a,b=divmod(member,4); p=(a&1)+(b&1); n=(a>>1)+(b>>1)
                cell=bank.get((i,j,a,b));q=cell['live_preferred_distance'] if cell else n
                delta=sites[i,a]-sites[j,b];d=np.linalg.norm(delta)
                if d==0 and q:unresolved.append([state,'orbital',member]);continue
                torque+=((p+n)*(d-q)/d if d else p+n)*delta@(jacobian[i,a]-jacobian[j,b])
            rows.append(torque.tolist())
        tables.append({'residue_pair':[i,j],'state_torsion_response':rows,'unresolved_entries':unresolved})
    return {'schema':'joint-source-force-family/v1','source_sha256':relation['source_sha256'],
        'event_count':len(relation['events']),'pair_force_tables':tables,
        'operator':'sum native pair force functions on the same shared residue edge variables',
        'arithmetic':'binary64 native frame pullback','full_chain_lattice_fold_closed':False}


if __name__=='__main__':
    relation,source,native,output=map(Path,sys.argv[1:5])
    result=compile_forces(json.loads(relation.read_text()),json.loads(source.read_text()),json.loads(native.read_text()))
    output.write_text(json.dumps(result,separators=(',',':'))+'\n')
    print('pair force tables',len(result['pair_force_tables']))
