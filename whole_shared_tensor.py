"""Exact variable elimination for occurrence-bound pair factors on shared ports."""
from fractions import Fraction
import hashlib
import json
from math import prod


def project_joint_source_events(events, n):
    """Project executed joint rows without inferring any missing coupling.

    Each row carries the complete residue-state tuple, an exact integer mass,
    and the source edge ancestry that produced that tuple. Equal tuples retain
    their separate ancestry rows through projection.
    """
    pairs = [(i, j) for i in range(n) for j in range(i + 1, n)]
    projected = []
    for event in events:
        if not event.get('address') or not event.get('rows'):
            raise ValueError('joint source event lacks its address or executed rows')
        distributions = {pair: [0] * 256 for pair in pairs}
        retained = []
        for row in event['rows']:
            states, mass, ancestry = row['states'], row['mass'], row['ancestry']
            if len(states) != n or any(type(s) is not int or not 0 <= s < 16 for s in states):
                raise ValueError('joint source row lacks a state at every residue')
            if type(mass) is not int or mass <= 0 or len(ancestry) != n:
                raise ValueError('joint source row lacks positive exact mass or residue ancestry')
            if any(not port for port in ancestry):
                raise ValueError('joint source row has an unbound residue port')
            for i, j in pairs:
                distributions[i, j][16 * states[i] + states[j]] += mass
            retained.append(row)
        projected.append({'address': event['address'], 'rows': retained,
            'pair_projections': distributions})
    if not projected:
        raise ValueError('joint source contraction has no executed events')
    return projected


def sixteen3_contexts(left,right):
    true,false,all_states=0b1010,0b1100,0b1111
    join,meet=left|right,left&right
    return [join,meet,(join&true)|(meet&(all_states^true)),
        (meet&true)|(join&(all_states^true)),
        (join&false)|(meet&(all_states^false)),
        (meet&false)|(join&(all_states^false))]


def bind_joint_occurrence_tensors(events, n, occurrence_tensors):
    """Require the joint lift to recover every supplied occurrence projection.

    Compare normalized integer masses exactly because complete joint rows can
    carry a different common multiplicity from a two-port projection.
    """
    projected = project_joint_source_events(events, n)
    bindings = []
    for event in projected:
        address = tuple(event['address'])
        for pair, actual in event['pair_projections'].items():
            key = (address, pair)
            if key not in occurrence_tensors:
                raise ValueError(f'joint lift lacks source occurrence tensor {key}')
            expected = occurrence_tensors[key]
            if len(expected) != 256 or any(type(v) is not int or v < 0 for v in expected):
                raise ValueError('source occurrence is not a reconstructed integer pair tensor')
            a, b = sum(actual), sum(expected)
            if b <= 0 or any(x * b != y * a for x, y in zip(actual, expected)):
                raise ValueError(f'joint lift changes source occurrence tensor {key}')
            bindings.append({'event_address': list(address), 'residue_pair': list(pair),
                'source_mass': b, 'joint_mass': a})
    if len(bindings) != len(occurrence_tensors):
        raise ValueError('joint lift leaves source occurrence tensors unconsumed')
    return bindings


def contract_joint_source_events(events, n, occurrence_tensors=None):
    """Apply all pair contexts to the same executed joint rows.

    The source rows and their ancestry remain the carried object. Pair tensors
    are its exact projections, rather than inputs used to invent a joint law.
    """
    bindings = (bind_joint_occurrence_tensors(events, n, occurrence_tensors)
        if occurrence_tensors is not None else None)
    projected = project_joint_source_events(events, n)
    pair_records = []
    four = [[0, 0, 0] for _ in range(n)]
    total_mass = 0
    for event in projected:
        for row in event['rows']:
            total_mass += row['mass']
            for residue, state in enumerate(row['states']):
                for vertex in range(4):
                    if state & (1 << vertex):
                        x, y = 1 - 2 * (vertex & 1), 1 - 2 * (vertex >> 1)
                        for axis, value in enumerate((x, y, x * y)):
                            four[residue][axis] += row['mass'] * value
        for (i, j), distribution in event['pair_projections'].items():
            signed = distribution[:]
            for bit in range(8):
                for state in range(256):
                    if state & (1 << bit):
                        signed[state] -= signed[state ^ (1 << bit)]
            recovered = signed[:]
            for bit in range(8):
                for state in range(256):
                    if state & (1 << bit):
                        recovered[state] += recovered[state ^ (1 << bit)]
            if recovered != distribution:
                raise ValueError('joint projection lost its signed Mobius tensor')
            contexts = [[0] * 16 for _ in range(6)]
            for state, mass in enumerate(recovered):
                for order, result in enumerate(sixteen3_contexts(state // 16, state % 16)):
                    contexts[order][result] += mass
            pair_records.append({'event_address': event['address'], 'residue_pair': [i, j],
                'signed_terms': [[state, mass] for state, mass in enumerate(signed) if mass],
                'joint_counts': [[state, mass] for state, mass in enumerate(recovered) if mass],
                'sixteen3_context_counts': contexts})
    return {'kind': 'exact projection and SIXTEEN3 contraction of executed joint source rows',
        'events': events, 'residue_ports': n, 'total_mass': total_mass,
        'source_occurrence_bindings': bindings,
        'source_occurrence_lift_verified': bindings is not None,
        'pair_tensors': pair_records, 'four_character_numerators': four,
        'four_character_denominator': total_mass}


def contract(factors,pairs,n,pair_tensor_uses,source_events,decoded_pairs):
    """Contract the rank-one K_n tensor without materializing its Cartesian support.

    Each source-addressed pair factor is the exact product of its two common
    normalized residue marginals. Across the complete graph, every port factor
    therefore appears n-1 times and variable elimination yields c(s)^(n-1).
    """
    transitions={}
    for left in range(16):
        for right in range(16):
            contexts=sixteen3_contexts(left,right);rows=[]
            for a,b in (contexts[0:2],contexts[2:4],contexts[4:6]):
                terms=[]
                for mask in (a,b):
                    positive=((mask>>1)&1)+((mask>>3)&1)
                    negative=((mask>>2)&1)+((mask>>3)&1)
                    terms.append((positive+negative,negative*(positive+negative),
                        negative*(positive+negative)**2))
                if sum(term[0] for term in terms):
                    rows.append(((a or 15,b or 15),
                        tuple(sum(term[axis] for term in terms) for axis in range(3))))
            transitions[left,right]=rows
    if not factors or len(source_events)!=len(factors):raise ValueError("shared-port contraction has an incomplete source event family")
    event_weights=[];factor_groups={}
    for event_index,(factor_event,event) in enumerate(zip(factors,source_events)):
        if len(event)!=5 or len(event[4])!=len(pairs):raise ValueError("event lacks one occurrence address per pair factor")
        port_norms=[sum(int(count)**(n-1) for _,count in port[1]) for port in factor_event]
        port_denominator=prod(int(port[0])**(n-1) for port in factor_event)
        raw_pair_mass=prod(sum(int(value) for value in decoded_pairs[pair_index][ordinal][1])
            for pair_index,ordinal in enumerate(event[4]))
        partition_mass=Fraction(raw_pair_mass*prod(port_norms),port_denominator)
        signature=json.dumps(factor_event,separators=(',',':'))
        group=factor_groups.setdefault(signature,[factor_event,Fraction(0)])
        group[1]+=partition_mass
        event_weights.append([event_index,partition_mass])
    partition_total=sum((weight for _,weight in event_weights),Fraction(0))
    if partition_total<=0:raise ValueError("shared-port contraction has zero total partition mass")
    event_weights=[(index,weight/partition_total) for index,weight in event_weights]
    if not factors:raise ValueError("shared-port contraction has no source events")
    contracted_four=[[Fraction(0) for _ in range(3)] for _ in range(n)]
    pair_marginals={tuple(pair['residues']):[Fraction(0) for _ in range(256)] for pair in pairs}
    event_count=len(factors)
    for factor_event,group_mass in factor_groups.values():
        if len(factor_event)!=n:raise ValueError("shared source event lost a residue port")
        powered=[];normalizers=[];probabilities=[]
        group_probability=group_mass/partition_total
        for residue,(_,states,_) in enumerate(factor_event):
            weights={int(state):int(count)**(n-1) for state,count in states}
            normalizer=sum(weights.values())
            if normalizer<=0:raise ValueError("shared-port contraction has an empty residue factor")
            powered.append(weights);normalizers.append(normalizer)
            probabilities.append({state:Fraction(weight,normalizer) for state,weight in weights.items()})
            for axis in range(3):
                moment=0
                for state,weight in weights.items():
                    for vertex in range(4):
                        if state&(1<<vertex):
                            x,y=1-2*(vertex&1),1-2*(vertex>>1)
                            moment+=weight*(x,y,x*y)[axis]
                contracted_four[residue][axis]+=Fraction(moment,normalizer)*group_probability
        for pair in pairs:
            i,j=pair['residues'];joint=pair_marginals[(i,j)]
            for left,left_probability in probabilities[i].items():
                for right,right_probability in probabilities[j].items():
                    joint[16*left+right]+=group_probability*left_probability*right_probability
    response_totals={}
    for pair in pairs:
        i,j=pair['residues']
        for state,probability in enumerate(pair_marginals[(i,j)]):
            if not probability:continue
            left,right=state//16,state%16
            for key,values in transitions[left,right]:
                row=response_totals.setdefault((i,key[0],j,key[1]),[Fraction(0),Fraction(0),Fraction(0)])
                for axis,value in enumerate(values):row[axis]+=probability*value
    responses=[(*key,*values) for key,values in sorted(response_totals.items())]
    response_records=[[*key,[[str(value.numerator),str(value.denominator)] for value in values]]
        for key,values in sorted(response_totals.items())]
    factor_sha=hashlib.sha256(json.dumps(factors,separators=(',',':')).encode()).hexdigest()
    response_sha=hashlib.sha256(json.dumps(response_records,separators=(',',':')).encode()).hexdigest()
    scale=1<<128
    four_records=[]
    for row in contracted_four:
        four_row=[]
        for value in row:
            numerator=value.numerator*scale;denominator=value.denominator
            four_row.append({'lower_numerator':str(numerator//denominator),
                'upper_numerator':str(-((-numerator)//denominator)),
                'denominator_power_two':128})
        four_records.append(four_row)
    metadata={'kind':'exact rank-one variable elimination over the shared residue source tensor',
        'source_events':event_count,'distinct_factorized_events':len(factor_groups),
        'pair_factors_per_event':len(pairs),'shared_residue_ports':n,'incident_factor_power':n-1,
        'normalization':'exact integer powers, raw occurrence partition masses, and rational port marginals',
        'sixteen3':'join/meet contexts applied to pair marginals of the single contracted event tensor',
        'pair_tensor_occurrence_uses':pair_tensor_uses,'rank_one_occurrence_fibres_verified':True,
        'event_partition_weights':[[index,[str(weight.numerator),str(weight.denominator)]] for index,weight in event_weights],
        'factorized_event_tensor_sha256':factor_sha,'response_coefficients_sha256':response_sha,
        'response_coefficients':response_records,'contracted_source_four_coordinates':four_records}
    return responses,four_records,metadata
