"""Occurrence-bound pair gates over shared, addressed source edge variables."""
import base64
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import sys
import zlib
from whole_shared_tensor import sixteen3_contexts


def compile_source(source):
    contraction = source['one_shot_lattice_contraction']
    graph_events = contraction['joint_source_graph_events']
    references = contraction['shared_source_event_readout']
    pairs = source['pairs']
    tensors = []
    for pair in pairs:
        payload = pair['source_occurrence_mobius_interactions']
        tensors.append(json.loads(zlib.decompress(base64.b64decode(payload[2:]))))
    checked = set()
    events = []
    if len(graph_events) != len(references):
        raise ValueError('joint graph events do not cover source occurrences')
    for graph, reference in zip(graph_events, references):
        forms, component, tick, _, ordinals = reference
        if (graph['forms'], graph['component'], graph['tick']) != (forms, component, tick):
            raise ValueError('joint graph event address differs from occurrence address')
        ports = graph['ports']
        if len(ports) != 20 or len(ordinals) != 190:
            raise ValueError('incomplete shared residue ports or pair gates')
        for pair_index, (pair, ordinal) in enumerate(zip(pairs, ordinals)):
            i, j = pair['residues']
            occurrence = tensors[pair_index][ordinal]
            if occurrence[:3] != [forms[i], forms[j], component]:
                raise ValueError('pair occurrence does not bind to the shared event')
            key = (pair_index, ordinal)
            if key in checked:
                continue
            counts = [0] * 256
            # Project addressed edges solely to check the existing pair observer.
            # The full edge domains remain in the resulting joint relation.
            for left, lc in Counter(ports[i]['edge_states']).items():
                for right, rc in Counter(ports[j]['edge_states']).items():
                    counts[16 * left + right] += lc * rc
            stated = [0] * 256
            for state, mass in occurrence[4]:
                stated[state] = mass
            signed = [0] * 256
            for state, coefficient in occurrence[5]:
                signed[state] = coefficient
            for bit in range(8):
                for state in range(256):
                    if state & (1 << bit):
                        signed[state] += signed[state ^ (1 << bit)]
            if counts != stated or signed != stated:
                raise ValueError('shared edge ports fail exact source Mobius recovery')
            checked.add(key)
        events.append({'address': [forms, component, tick], 'ports': ports,
            'pair_occurrence_ordinals': ordinals})
    if len(checked) != sum(map(len, tensors)):
        raise ValueError('source occurrences remain outside the joint relation')
    return {'schema': 'whole-joint-edge-relation/v1',
        'representation': 'one addressed edge variable per residue, shared by every incident pair gate',
        'events': events, 'pair_ports': [p['residues'] for p in pairs],
        'source_occurrence_tensors': tensors,
        'sixteen3_gate_table': [sixteen3_contexts(s // 16, s % 16) for s in range(256)],
        'source_graphs': source['site_graphs'], 'occurrences_bound': len(checked),
        'assignment_family': 'implicit; no edge coupling or probability law inferred',
        'full_chain_lattice_fold_closed': False}


def evaluate_assignment(relation, event_index, edges):
    """Execute all pair gates using the same twenty source edge addresses."""
    event = relation['events'][event_index]
    if len(edges) != len(event['ports']):
        raise ValueError('incomplete joint edge assignment')
    states = []
    for port, edge in zip(event['ports'], edges):
        if type(edge) is not int or not 0 <= edge < len(port['edge_states']):
            raise ValueError('edge assignment exceeds its source graph')
        states.append(port['edge_states'][edge])
    return {'event_address': event['address'], 'edge_addresses': edges,
        'source_sha256': relation['source_sha256'],
        'states': states, 'pair_contexts': [relation['sixteen3_gate_table'][16*states[i]+states[j]]
            for i, j in relation['pair_ports']]}


def source_overlap_correlations(relation):
    """Expose the co-occurrence constraint shared by overlapping pair gates."""
    ports=relation['pair_ports'];n=len(relation['events'][0]['ports'])
    if ports!=[[i,j] for i in range(n) for j in range(i+1,n)]:
        raise ValueError('source pair gates do not cover the complete residue graph')
    rows=[];nontrivial=0
    for event in relation['events']:
        for residue,port in enumerate(event['ports']):
            if port['residue']!=residue:raise ValueError('source port identity changed')
            counts=Counter(port['edge_states']);mass=sum(counts.values())
            mismatch=mass*mass-sum(c*c for c in counts.values())
            nontrivial+=mismatch>0
            rows.append({'event_address':event['address'],'residue':residue,
                'incident_pair_gates':[i for i,pair in enumerate(ports) if residue in pair],
                'shared_port_disagreement_mass':0,
                'independent_pair_copy_disagreement_numerator':mismatch,
                'independent_pair_copy_disagreement_denominator':mass*mass})
    return {'source_sha256':relation['source_sha256'],'shared_source_port_bindings':rows,
        'nontrivial_overlap_correlations':nontrivial,
        'relation':'every incident pair gate reads the same addressed edge variable',
        'full_chain_lattice_fold_closed':False}


def mechanical_assignment_response(relation, event_index, edges, sites, orbital_source=None, bank_override=None):
    """Return energy and site forces for one retained joint assignment.

    Sites are the carried native FOUR frames. The resulting gradients must pass
    through their frame pullback before any backbone or bank transformation.
    """
    carried = evaluate_assignment(relation, event_index, edges)
    if len(sites) != len(edges) or any(len(row) != 4 for row in sites):
        raise ValueError('mechanical return lacks complete carried FOUR frames')
    gradient = [[[0.0] * 3 for _ in range(4)] for _ in sites]
    energy = 0.0
    channels = []
    for (i, j), contexts in zip(relation['pair_ports'], carried['pair_contexts']):
        for order in range(3):
            masks = contexts[2 * order:2 * order + 2]
            k = first = second = 0
            for mask in masks:
                positive = ((mask >> 1) & 1) + ((mask >> 3) & 1)
                negative = ((mask >> 2) & 1) + ((mask >> 3) & 1)
                span = positive + negative
                k += span
                first += negative * span
                second += negative * span * span
            if not k:
                continue
            supports = [[v for v in range(4) if mask & (1 << v)] or list(range(4))
                for mask in masks]
            centers = [[sum(sites[residue][v][axis] for v in support) / len(support)
                for axis in range(3)] for residue, support in zip((i, j), supports)]
            delta = [a - b for a, b in zip(*centers)]
            distance = math.sqrt(sum(x * x for x in delta))
            if distance == 0 and first:
                raise ValueError('joint mechanical return holds an unresolved zero-distance force')
            energy += (k * distance * distance - 2 * first * distance + second) / 2
            slope = k - first / distance if distance else k
            for residue, support, sign in ((i, supports[0], 1), (j, supports[1], -1)):
                for vertex in support:
                    for axis in range(3):
                        gradient[residue][vertex][axis] += sign * slope * delta[axis] / len(support)
            channels.append({'pair': [i, j], 'order': order, 'supports': masks,
                'coefficients': [k, first, second], 'distance': distance})
    orbital_channels = []
    if orbital_source is not None:
        table = orbital_source['orbital_transfer']['source_and_closed_families']
        preferences = {tuple(c['source']): c['live_preferred_distance']
            for c in (bank_override or orbital_source['glut_work'])['cells']}
        for i,j in relation['pair_ports']:
            family = table[16*carried['states'][i]+carried['states'][j]][1]
            for member in range(16):
                if not family & (1 << member):
                    continue
                a,b = divmod(member,4)
                positive = (a & 1)+(b & 1)
                negative = (a >> 1)+(b >> 1)
                stiffness = positive+negative
                if not stiffness:
                    continue
                preferred = preferences.get((i,j,a,b),float(negative))
                delta = [x-y for x,y in zip(sites[i][a],sites[j][b])]
                distance = math.sqrt(sum(x*x for x in delta))
                if distance == 0 and preferred:
                    raise ValueError('orbital mechanical return has unresolved zero-distance force')
                error = distance-preferred
                energy += stiffness*error*error/2
                slope = stiffness*error/distance if distance else stiffness
                for axis in range(3):
                    gradient[i][a][axis] += slope*delta[axis]
                    gradient[j][b][axis] -= slope*delta[axis]
                orbital_channels.append({'source':[i,j,a,b],'preferred':preferred,
                    'stiffness':stiffness,'distance':distance,'joint_branch_multiplicity':1})
    carried.update({'mechanical_energy': energy, 'site_energy_gradient': gradient,
        'joint_orbital_channels':orbital_channels,
        'mechanical_channels': channels, 'frame_pullback_pending': True,
        'full_chain_lattice_fold_closed': False})
    return carried


def compile_mechanical_family(relation, sites, orbital_source=None, bank_override=None):
    """Compile the energy as pair functions of shared edge variables.

    All events use these same carried frames. No probabilities or independent
    pair averages are introduced: an assignment selects one entry per pair.
    """
    centroids = []
    for row in sites:
        if len(row) != 4:
            raise ValueError('mechanical family lacks a complete FOUR frame')
        centers = []
        for mask in range(16):
            vertices = [v for v in range(4) if mask & (1 << v)] or list(range(4))
            centers.append([sum(row[v][axis] for v in vertices)/len(vertices) for axis in range(3)])
        centroids.append(centers)
    tables = []
    bank = (bank_override or orbital_source['glut_work']) if orbital_source is not None else None
    bank_cells = {tuple(c['source']):c for c in bank['cells']} if bank else {}
    for i, j in relation['pair_ports']:
        energies = []; singular = []
        for state, contexts in enumerate(relation['sixteen3_gate_table']):
            energy = 0.0
            for order in range(3):
                a, b = contexts[2*order:2*order+2]
                k = first = second = 0
                for mask in (a, b):
                    positive = ((mask >> 1) & 1) + ((mask >> 3) & 1)
                    negative = ((mask >> 2) & 1) + ((mask >> 3) & 1)
                    span = positive + negative
                    k += span; first += negative*span; second += negative*span*span
                distance = math.sqrt(sum((x-y)**2 for x,y in zip(centroids[i][a],centroids[j][b])))
                energy += (k*distance*distance-2*first*distance+second)/2
                if distance == 0 and first:
                    singular.append([state, order])
            if orbital_source is not None:
                family = orbital_source['orbital_transfer']['source_and_closed_families'][state][1]
                for member in range(16):
                    if not family & (1 << member):
                        continue
                    a,b = divmod(member,4)
                    positive,negative = (a&1)+(b&1),(a>>1)+(b>>1)
                    stiffness = positive+negative
                    cell = bank_cells.get((i,j,a,b))
                    preferred = cell['live_preferred_distance'] if cell else float(negative)
                    distance = math.sqrt(sum((x-y)**2 for x,y in zip(sites[i][a],sites[j][b])))
                    energy += stiffness*(distance-preferred)**2/2
                    if cell:
                        # Unit branch mass; source aggregate multiplicity remains
                        # in the retained bank and does not rescale this branch.
                        energy += positive*positive*negative/2+positive*cell['release_coordinate']**2/2
            energies.append(energy)
        tables.append({'residue_pair': [i,j], 'state_energy': energies,
            'unresolved_force_entries': singular})
    return {'schema': 'joint-source-mechanical-family/v1',
        'source_sha256': relation['source_sha256'],
        'operator': 'sum of pair gate energies evaluated on common addressed residue variables',
        'event_count': len(relation['events']), 'pair_tables': tables,
        'source_occurrences_bound': relation['occurrences_bound'],
        'joint_orbital_bank_included': orbital_source is not None,
        'carried_bank': bank,
        'four_sites': sites, 'assignment_family_retained': True,
        'arithmetic': 'binary64 mechanical readout; exact source edge and Mobius addresses retained',
        'full_chain_lattice_fold_closed': False}


def evaluate_mechanical_family(relation, family, event_index, edges):
    if family['source_sha256'] != relation['source_sha256']:
        raise ValueError('mechanical family differs from its source relation')
    state = evaluate_assignment(relation, event_index, edges)['states']
    return sum(table['state_energy'][16*state[i]+state[j]]
        for table in family['pair_tables'] for i,j in [table['residue_pair']])


def coupled_event_readout(relation, family, event_index, timeout_ms=20000):
    import z3
    if relation['source_sha256'] != family['source_sha256']:
        raise ValueError('coupled readout has inconsistent source binding')
    ports = relation['events'][event_index]['ports']
    domains = [sorted(set(port['edge_states'])) for port in ports]
    variables = [z3.Int(f'residue_{i}') for i in range(len(ports))]
    solver = z3.Optimize(); solver.set(timeout=timeout_ms)
    for variable, domain in zip(variables, domains):
        solver.add(z3.Or([variable == value for value in domain]))
    terms = []
    for table in family['pair_tables']:
        i,j = table['residue_pair']
        for left in domains[i]:
            for right in domains[j]:
                energy = table['state_energy'][16*left+right]
                numerator,denominator = energy.as_integer_ratio()
                terms.append(z3.If(z3.And(variables[i]==left,variables[j]==right),
                    z3.RealVal(f'{numerator}/{denominator}'),z3.RealVal(0)))
    objective = solver.minimize(z3.Sum(terms))
    status = solver.check()
    result = {'solver_status': str(status), 'event_index': event_index,
        'source_sha256': relation['source_sha256'], 'full_chain_lattice_fold_closed': False}
    if status == z3.sat:
        model = solver.model()
        states = [model.eval(v).as_long() for v in variables]
        edges = [port['edge_states'].index(state) for port,state in zip(ports,states)]
        result.update(evaluate_assignment(relation,event_index,edges))
        result.update({'objective_lower':str(objective.lower()),'objective_upper':str(objective.upper()),
            'mechanical_energy':evaluate_mechanical_family(relation,family,event_index,edges),
            'readout_scope':'selected assignment; original source family remains held'})
        event = relation['events'][event_index]
        mismatches = []
        for pair_index, (i,j) in enumerate(relation['pair_ports']):
            ordinal = event['pair_occurrence_ordinals'][pair_index]
            occurrence = relation['source_occurrence_tensors'][pair_index][ordinal]
            counts = dict(occurrence[4]); total = sum(counts.values())
            selected = 16*states[i]+states[j]
            if counts.get(selected,0) != total:
                mismatches.append({'residue_pair':[i,j], 'occurrence_ordinal':ordinal,
                    'selected_state':selected,'selected_source_mass':counts.get(selected,0),
                    'total_source_mass':total})
        result['source_occurrence_recovery_verified'] = not mismatches
        result['source_occurrence_mismatches'] = mismatches
        result['carrier_return'] = {'source_sha256':relation['source_sha256'],
            'event_index':event_index,'complete_edge_domains':ports,
            'pair_occurrence_ordinals':event['pair_occurrence_ordinals'],
            'status':'held' if mismatches else 'candidate requires mechanical and SIC return'}
    else:
        result['reason'] = solver.reason_unknown()
    return result


if __name__ == '__main__':
    path, output = map(Path, sys.argv[1:3])
    raw = path.read_bytes()
    result = compile_source(json.loads(raw))
    result['source_sha256'] = hashlib.sha256(raw).hexdigest()
    output.write_text(json.dumps(result, separators=(',', ':')) + '\n')
    print('events', len(result['events']), 'source occurrences', result['occurrences_bound'])
