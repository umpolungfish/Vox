"""Exact dyadic channel work bounds for the retained joint source family."""
from fractions import Fraction as Q
from math import isqrt
import copy
import json
from pathlib import Path
import sys


def admit(source, proposed):
    sites = source['spatial_sites']['positions']
    current = source['glut_work']
    candidate = copy.deepcopy(proposed)
    if len(current['cells']) != len(candidate['cells']):
        raise ValueError('bank candidate changes the source cell family')
    records = []; accepted = 0
    scale = 1 << 128
    for index, (old, new) in enumerate(zip(current['cells'], candidate['cells'])):
        for field in ('source', 'multiplicity', 'positive', 'negative'):
            if old[field] != new[field]:
                raise ValueError('bank candidate changes source ancestry or ranks')
        i,j,a,b = old['source']
        square = sum((Q(x)-Q(y))**2 for x,y in zip(sites[i][a], sites[j][b]))
        radicand = square.numerator*scale*scale//square.denominator
        root = isqrt(radicand)
        low = Q(root, scale)
        high = low if low*low == square else Q(root+1,scale)
        p,n = old['positive'],old['negative']
        q0,q1 = Q(old['live_preferred_distance']),Q(new['live_preferred_distance'])
        z0,z1 = Q(old['release_coordinate']),Q(new['release_coordinate'])
        constant = Q(p+n,2)*(q1*q1-q0*q0)+Q(p,2)*(z1*z1-z0*z0)
        coefficient = -(p+n)*(q1-q0)
        endpoints = [constant+coefficient*low,constant+coefficient*high]
        lower,upper = min(endpoints),max(endpoints)
        permitted = upper <= 0
        if permitted:
            accepted += old['release_coordinate'] != new['release_coordinate']
        else:
            candidate['cells'][index] = copy.deepcopy(old)
        records.append({'source':old['source'],'admitted':permitted,
            'per_branch_work_lower':str(lower),'per_branch_work_upper':str(upper)})
    cells = candidate['cells']
    candidate['coordinate_work'] = sum(c['coordinate_work'] for c in cells)
    candidate['remaining_accessible_energy'] = sum(c['multiplicity']*c['positive']*
        c['release_coordinate']**2/2 for c in cells)
    candidate['remaining_stored_energy'] = sum(c['banked_second_preference_energy']+
        c['multiplicity']*c['positive']*c['release_coordinate']**2/2 for c in cells)
    return {'schema':'joint-source-channel-admission/v1',
        'scope':'all joint edge assignments: each orbital branch selects a subset of the same nonincreasing source channels',
        'distance_arithmetic':'outward 128-bit dyadic square root of exact binary64 site coordinates',
        'bank_cells':records,'changed_cells':accepted,
        'held_cells':sum(not row['admitted'] for row in records),
        'returned_bank':candidate,'source_ancestry_preserved':True,
        'full_chain_lattice_fold_closed':False}


if __name__ == '__main__':
    source_path, candidate_path, output = map(Path, sys.argv[1:4])
    source=json.loads(source_path.read_text())
    proposal=json.loads(candidate_path.read_text())['conditional_bank_candidate']
    result=admit(source,proposal)
    output.write_text(json.dumps(result,indent=2)+'\n')
    print('changed',result['changed_cells'],'held',result['held_cells'])
