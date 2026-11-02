"""Whole source-bound algebraic fold construction and exact real closure.

No coordinate trajectory or reference structure enters this compiler. The
native prepared source supplies its complete joint distributions, FOUR frame
and shell channels. Closure retains the full implicit source equation family.
"""
import argparse
import base64
from collections import defaultdict
from fractions import Fraction
import hashlib
import json
from math import gcd
from pathlib import Path
import time
import subprocess
import zlib
import numpy as np
import z3 as z
from whole_shared_tensor import contract as contract_shared_source_events


def add(a, b): return tuple(x+y for x, y in zip(a, b))
def sub(a, b): return tuple(x-y for x, y in zip(a, b))
def scale(a, s): return tuple(x*s for x in a)
def cross(a, b): return tuple(a[(k+1)%3]*b[(k+2)%3]-a[(k+2)%3]*b[(k+1)%3] for k in range(3))
def dot(a, b): return sum(x*y for x, y in zip(a, b))
def rational(x):
    x=Fraction(x)
    return z.RealVal(f'{x.numerator}/{x.denominator}')


def decoded_occurrences(records):
    """Yield addresses and exact 256-state rows, batching legacy Möbius inverses."""
    if not records:return
    if isinstance(records[0],list) and len(records[0])==6:
        for left,right,component,tick,joint_rows,terms in records:
            row=[0]*256
            for state,count in joint_rows:row[state]=count
            yield {'left_form':left,'right_form':right,'component':component,'tick':tick},row,len(terms)
        return
    if isinstance(records[0],list) and len(records[0])==5:
        coefficients=np.zeros((len(records),256),dtype=object)
        for ordinal,(_,_,_,_,terms) in enumerate(records):
            for state,value in terms:coefficients[ordinal,state]=int(value)
        states=np.arange(256)
        for bit in range(8):
            selected=states[(states&(1<<bit))!=0]
            coefficients[:,selected]=coefficients[:,selected]+coefficients[:,selected^(1<<bit)]
        for ordinal,(left,right,component,tick,terms) in enumerate(records):
            yield {'left_form':left,'right_form':right,'component':component,'tick':tick},coefficients[ordinal],len(terms)
        return
    for occurrence in records:
        address=occurrence['address'];row=[0]*256
        if 'signed_terms' in occurrence:
            for term in occurrence['signed_terms']:
                row[16*term['left_support_subset']+term['right_support_subset']]=term['coefficient']
            for bit in range(8):
                for state in range(256):
                    if state&(1<<bit):row[state]+=row[state^(1<<bit)]
            count=len(occurrence['signed_terms'])
        else:
            for item in occurrence['joint_counts']:
                row[16*item['left_state']+item['right_state']]=item['count']
            count=0
        yield address,row,count


class Fold:
    def __init__(self, source, compile_geometry=True):
        self.source=source
        self.compile_geometry=compile_geometry
        self.n=len(source['source_paths'])
        self.variables=[]
        self.constants=[]
        self.domain=[]
        self.terms=[]
        self.bank=[]
        self.distances={}
        self.root_constants={}
        self.sqrt8=self.constant_root(Fraction(8)) if compile_geometry else None
        self.sqrt3=self.constant_root(Fraction(3)) if compile_geometry else None
        self.points=[]
        self.point_variables=[]
        self.geometry_equations=[]
        self.distance_equations=[]
        self.radial_derivatives=defaultdict(list)
        self.source_occurrence_count=0
        self.source_occurrence_joint_states=0
        self.source_occurrence_signed_terms=0
        self.source_events=source.get('shared_source_event_readout',[])
        self.source_event_pair_uses=int(source.get('shared_source_event_pair_tensor_uses',0))
        self.source_event_rejoin_closed=source.get('all_source_occurrences_rejoined') is True
        if not self.source_events or not self.source_event_rejoin_closed:
            raise ValueError('whole-chain fold requires the complete rejoined source-event carrier')
        expected_uses=len(self.source_events)*self.n*(self.n-1)//2
        if self.source_event_pair_uses!=expected_uses:
            raise ValueError('shared source-event readout did not consume every pair tensor at every event')
        event_sum=[[0,0,0] for _ in range(self.n)]
        for event in self.source_events:
            if len(event)!=5 or len(event[0])!=self.n or len(event[3])!=self.n:
                raise ValueError('shared source event has an incomplete residue address or return')
            if len(event[4])!=len(source['pairs']) or any(
                not isinstance(ordinal,int) or ordinal<0
                for ordinal in event[4]):
                raise ValueError('shared source event lacks one occurrence-tensor address per residue pair')
            for residue,winding in enumerate(event[3]):
                if len(winding)!=3:raise ValueError('shared source event has an invalid FOUR winding')
                for axis,value in enumerate(winding):event_sum[residue][axis]+=int(value)
        initial=source.get('source_initial_windings')
        if initial is None or any(len(row)!=3 for row in initial):
            raise ValueError('shared source-event carrier lacks its source winding boundary')
        event_sum=[[value+int(initial[i][axis]) for axis,value in enumerate(row)]
            for i,row in enumerate(event_sum)]
        if event_sum!=source.get('pair_contraction_windings'):
            raise ValueError('shared event returns do not sum to the source-bound residue windings')
        self.sites=[]
        if compile_geometry:
            for i in range(self.n):
                if i==0:p=(rational(0),)*3
                elif i==1:p=(rational(1),rational(0),rational(0))
                elif i==2:p=(rational(Fraction(4,3)),self.sqrt8/3,rational(0))
                else:
                    p=tuple(self.real(f'p_{i}_{k}') for k in range(3));self.point_variables.extend(p)
                self.points.append(p)
                if i>=3:
                    e=sub(p,self.points[i-1]);u=sub(self.points[i-1],self.points[i-2])
                    equations=[dot(e,e)-1,3*dot(u,e)-1]
                    self.geometry_equations.extend(equations);self.domain.extend(h==0 for h in equations)
                    self.domain.extend(z.And(x>=-self.n,x<=self.n) for x in p)
            for i,p in enumerate(self.points):
                if self.n>=3:
                    a,b,c=(i-2,i-1,i) if i>=2 else (0,1,2)
                    e=sub(self.points[c],self.points[b]);u=sub(self.points[b],self.points[a])
                    normal=scale(cross(u,e),3*self.sqrt8/8)
                    tangent=cross(normal,e)
                else:e,tangent,normal=(1,0,0),(0,1,0),(0,0,1)
                sites=[]
                for v in range(4):
                    a,b=1-2*(v&1),1-2*(v>>1)
                    sites.append(add(p,scale(add(scale(e,a),add(scale(tangent,b),scale(normal,a*b))),self.sqrt3/3)))
                self.sites.append(sites)
        self.compile_source()

    def real(self,name):
        value=z.Real(name);self.variables.append(value);return value

    def constant_root(self,value):
        value=Fraction(value)
        if value not in self.root_constants:
            root=z.Real(f'root_{value.numerator}_{value.denominator}')
            self.constants.append(root);self.domain.extend([root>0,root*root==rational(value)])
            self.root_constants[value]=root
        return self.root_constants[value]

    def centroid(self,i,mask):
        selected=[p for v,p in enumerate(self.sites[i]) if mask&(1<<v)] or self.sites[i]
        return tuple(sum(p[k] for p in selected)/len(selected) for k in range(3))

    def distance(self,i,a,j,b):
        # Exact centroid equivalence: empty and complete supports share the
        # frame origin; their source ranks/coefficients remain in the carrier.
        a=15 if a==0 else a;b=15 if b==0 else b
        key=(i,a,j,b)
        if key not in self.distances:
            delta=sub(self.centroid(i,a),self.centroid(j,b))
            square=dot(delta,delta)
            name='d_'+'_'.join(map(str,key))
            if j<3:
                d=z.Real(name);self.constants.append(d)
            else:
                d=self.real(name);self.distance_equations.append((d,square))
            self.domain.extend([d>=0,d<=2*self.n+2,d*d==square])
            self.distances[key]=d
        return self.distances[key]

    def compile_source(self):
        decoded_pairs=[]
        for pair in self.source['pairs']:
            occurrences=pair.get('source_occurrence_mobius_interactions')
            if isinstance(occurrences,str):
                payload=occurrences[2:] if occurrences.startswith('z:') else occurrences
                occurrences=json.loads(zlib.decompress(base64.b64decode(payload)))
            decoded_pairs.append(list(decoded_occurrences(occurrences or [])))
        pair_marginals=[]
        def normalized(hist):
            divisor=0
            for value in hist:divisor=gcd(divisor,int(value))
            if not divisor:raise ValueError('shared event pair occurrence has empty residue support')
            vector=tuple(int(value)//divisor for value in hist)
            return vector,sum(vector)
        for pair_index,rows in enumerate(decoded_pairs):
            marginals=[]
            for _,distribution,_ in rows:
                total=sum(int(value) for value in distribution)
                if total<=0:raise ValueError('shared event pair occurrence has zero mass')
                left,left_total=normalized([sum(int(distribution[16*x+y]) for y in range(16)) for x in range(16)])
                right,right_total=normalized([sum(int(distribution[16*x+y]) for x in range(16)) for y in range(16)])
                for x in range(16):
                    for y in range(16):
                        actual=int(distribution[16*x+y])*left_total*right_total
                        expected=left[x]*right[y]*total
                        if actual!=expected:
                            raise ValueError(f'pair occurrence {pair_index} is not a projection of one shared residue event')
                marginals.append(((left,left_total),(right,right_total)))
            pair_marginals.append(marginals)
        self.shared_event_factorizations=[]
        for event in self.source_events:
            residues=[None]*self.n
            for pair_index,ordinal in enumerate(event[4]):
                i,j=self.source['pairs'][pair_index]['residues']
                left,right=pair_marginals[pair_index][ordinal]
                for residue,signature in ((i,left),(j,right)):
                    if residues[residue] is None:residues[residue]=signature
                    elif residues[residue]!=signature:
                        raise ValueError(f'event residue {residue} has incompatible marginals across pair tensors')
            if any(signature is None for signature in residues):
                raise ValueError('shared source event does not cover every residue port')
            ports=[]
            for vector,total in residues:
                moments=[0,0,0]
                for state,value in enumerate(vector):
                    for vertex in range(4):
                        if state&(1<<vertex):
                            x,y=1-2*(vertex&1),1-2*(vertex>>1)
                            for axis,character in enumerate((x,y,x*y)):moments[axis]+=value*character
                ports.append([total,[[state,value] for state,value in enumerate(vector) if value],moments])
            self.shared_event_factorizations.append(ports)
        scale=1<<128
        self.source_four_coordinates=[]
        for residue in range(self.n):
            row=[]
            for axis in range(3):
                value=sum((Fraction(event[residue][2][axis],event[residue][0])
                    for event in self.shared_event_factorizations),Fraction(0))/len(self.shared_event_factorizations)
                numerator=value.numerator*scale;denominator=value.denominator
                row.append({'lower_numerator':str(numerator//denominator),
                    'upper_numerator':str(-((-numerator)//denominator)),
                    'denominator_power_two':128})
            self.source_four_coordinates.append(row)
        event_uses=[[0]*len(rows) for rows in decoded_pairs]
        for event in self.source_events:
            for pair_index,ordinal in enumerate(event[4]):
                if ordinal>=len(event_uses[pair_index]):
                    raise ValueError('shared event occurrence ordinal exceeds its pair tensor')
                event_uses[pair_index][ordinal]+=1
        self.source_event_occurrence_uses=0
        for pair_index,pair in enumerate(self.source['pairs']):
            i,j=pair['residues']
            # Reconstruct the full distribution from its signed interaction
            # terms before applying its geometry. No marginal replacement.
            coefficients=[0]*256
            for t in pair['mobius_joint_interactions']:
                coefficients[16*t['left_support_subset']+t['right_support_subset']]=t['coefficient']
            recovered=coefficients[:]
            for bit in range(8):
                for state in range(256):
                    if state&(1<<bit):recovered[state]+=recovered[state^(1<<bit)]
            assert recovered==pair['source_register_pair_counts']
            occurrence_total=[0]*256
            occurrence_count=0;joint_state_count=0;signed_term_count=0
            for ordinal,(occurrence_address,stated,term_count) in enumerate(decoded_pairs[pair_index]):
                nonzero=np.flatnonzero(stated)
                for state in nonzero:
                    state=int(state);mass=int(stated[state])
                    occurrence_total[state]+=mass
                occurrence_count+=1;joint_state_count+=len(nonzero);signed_term_count+=term_count
                uses=event_uses[pair_index][ordinal]
                if uses==0:raise ValueError('source occurrence tensor did not enter a shared chain event')
                self.source_event_occurrence_uses+=uses
            assert occurrence_total==recovered, 'source occurrence fibres do not join to the pair tensor'
            self.source_occurrence_count+=occurrence_count
            self.source_occurrence_joint_states+=joint_state_count
            self.source_occurrence_signed_terms+=signed_term_count
            # Keep the occurrence-addressed channel tensor in the retained
            # source, but do not feed its pairwise marginal into the fold.
            # The force system below is driven by the single shared-event
            # contraction, after SIXTEEN_3 has been applied across its ports.
        if self.source_event_occurrence_uses!=self.source_event_pair_uses:
            raise ValueError('whole fold did not consume every pair tensor at every shared event')
        self.shared_port_responses,self.contracted_source_four_coordinates,self.shared_port_contraction=(
            contract_shared_source_events(self.shared_event_factorizations,self.source['pairs'],
                self.n,self.source_event_occurrence_uses,self.source_events,decoded_pairs))
        if self.compile_geometry:
            for i,a,j,b,k,first,second in self.shared_port_responses:
                distance=self.distance(i,a,j,b)
                self.terms.append((k*distance*distance-2*first*distance+second)/2)
                self.radial_derivatives[distance.get_id()].append(k*distance-first)
        self.energy=z.Sum(self.terms or [z.RealVal(0)])
        self.domain_formula=z.And(self.domain)

    @staticmethod
    def sixteen3_contexts(left,right):
        true=0b1010
        false=0b1100
        all_states=0b1111
        join=left|right
        meet=left&right
        return [join,meet,(join&true)|(meet&(all_states^true)),
            (meet&true)|(join&(all_states^true)),
            (join&false)|(meet&(all_states^false)),
            (meet&false)|(join&(all_states^false))]

    def compile_pair_occurrence(self,i,j,distribution):
        grouped=defaultdict(lambda:[0,0,0])
        for state in np.flatnonzero(distribution):
            state=int(state);count=int(distribution[state])
            contexts=self.sixteen3_contexts(state//16,state%16)
            for a,b in (contexts[0:2],contexts[2:4],contexts[4:6]):
                k=first=second=0
                for mask in [a,b]:
                    p=((mask>>1)&1)+((mask>>3)&1);n=((mask>>2)&1)+((mask>>3)&1)
                    k+=p+n;first+=n*(p+n);second+=n*(p+n)**2
                if not k:continue
                key=(15 if a==0 else a,15 if b==0 else b)
                g=grouped[key]
                for index,value in enumerate([k,first,second]):g[index]+=count*value
        for (a,b),(k,first,second) in grouped.items():
            d=self.distance(i,a,j,b)
            self.terms.append((k*d*d-2*first*d+second)/2)
            self.radial_derivatives[d.get_id()].append(k*d-first)

    def compile_pair_channels(self,i,j,channels):
        for channel in channels:
            a,b=channel['occupancies'];count=channel['count']
            p,n=channel['spin_counts'];k=count*(p+n)
            if not k:continue
            d=self.distance(i,1<<a,j,1<<b)
            if p and n:
                lower=self.constant_root(Fraction(p*n*n,p+n))
                q=z.If(n*d<=(p+n)*lower,lower,z.If(d>=p+n,rational(n),n*d/(p+n)))
                self.terms.append(z.If(n*d<=(p+n)*lower,k*(d-lower)*(d-lower)/2,
                    z.If(d>=p+n,k*(d-n)*(d-n)/2+rational(Fraction(count*p*n*n,2)),
                        rational(Fraction(count*p,2))*(d*d-p*n))))
                self.radial_derivatives[d.get_id()].append(z.If(n*d<=(p+n)*lower,k*(d-lower),
                    z.If(d>=p+n,k*(d-n),count*p*d)))
                self.bank.append({'source':[i,j,a,b],'preference':q,'release_square':(p+n)*q*q/n-p*n})
            else:
                self.terms.append(k*(d-n)*(d-n)/2)
                self.radial_derivatives[d.get_id()].append(k*(d-n))

    def minimum_family(self):
        other=[z.Real('other_'+v.decl().name()) for v in self.variables]
        mapping=list(zip(self.variables,other))
        other_domain=z.substitute(self.domain_formula,*mapping)
        other_energy=z.substitute(self.energy,*mapping)
        return z.And(self.domain_formula,z.ForAll(other,z.Implies(other_domain,self.energy<=other_energy)))

    def fixed_family(self):
        """Complete simultaneous native force closure on the regular fibre.

        All source terms act together. Singular radical fibres remain an
        explicit held continuation of this same source relation.
        """
        memo={}
        def derivative(expression,variable):
            if z.eq(expression,variable):return rational(1)
            if z.is_const(expression):return rational(0)
            key=(expression.get_id(),variable.get_id())
            if key in memo:return memo[key]
            children=expression.children();kind=expression.decl().kind()
            if kind==z.Z3_OP_ADD:answer=z.Sum([derivative(c,variable) for c in children])
            elif kind==z.Z3_OP_SUB:
                values=[derivative(c,variable) for c in children];answer=values[0]-sum(values[1:])
            elif kind==z.Z3_OP_UMINUS:answer=-derivative(children[0],variable)
            elif kind==z.Z3_OP_MUL:
                terms=[]
                for index,c in enumerate(children):
                    d=derivative(c,variable)
                    if z.is_rational_value(d) and d.as_fraction()==0:continue
                    term=d
                    for j,other in enumerate(children):
                        if j!=index:term=term*other
                    terms.append(term)
                answer=z.Sum(terms or [rational(0)])
            elif kind==z.Z3_OP_DIV:
                assert z.is_true(z.simplify(derivative(children[1],variable)==0))
                answer=derivative(children[0],variable)/children[1]
            elif kind==z.Z3_OP_POWER:
                exponent=children[1].as_long();answer=exponent*children[0]**(exponent-1)*derivative(children[0],variable)
            else:raise ValueError('nonpolynomial frame operation '+expression.decl().name())
            answer=z.simplify(answer);memo[key]=answer;return answer
        multipliers=[self.real(f'geometry_multiplier_{i}') for i in range(len(self.geometry_equations))]
        forces=[]
        for index,(distance,square) in enumerate(self.distance_equations):
            multiplier=self.real(f'distance_multiplier_{index}')
            radial=z.Sum(self.radial_derivatives[distance.get_id()] or [rational(0)])
            self.domain.extend([distance>0,radial+2*distance*multiplier==0])
            forces.append((multiplier,square))
        for variable in self.point_variables:
            equation=z.Sum([-multiplier*derivative(square,variable) for multiplier,square in forces]
                +[multiplier*derivative(h,variable) for multiplier,h in zip(multipliers,self.geometry_equations)])
            self.domain.append(equation==0)
        self.singular_continuation={'condition':'some moving support distance is zero',
            'retained_domain':z.And(self.domain_formula,z.Or([d==0 for d,_ in self.distance_equations])).sexpr(),
            'boundary':'open radical fibre; no differentiable identity singleton is substituted'}
        return z.And(self.domain)


def algebraic_value(model,expression):
    value=z.simplify(model.eval(expression,model_completion=True))
    if z.is_rational_value(value):
        exact=Fraction(value.numerator_as_long(),value.denominator_as_long());lo=hi=exact
    elif z.is_algebraic_value(value):
        hi_value=value.approx(160);lo_value=(-value).approx(160)
        hi=Fraction(hi_value.numerator_as_long(),hi_value.denominator_as_long())
        lo=-Fraction(lo_value.numerator_as_long(),lo_value.denominator_as_long())
    else:raise ValueError('backend returned a non-algebraic coordinate: '+value.sexpr())
    denominator=1<<128
    lower=(lo.numerator*denominator)//lo.denominator
    upper=-((-hi.numerator*denominator)//hi.denominator)
    return {'root':value.sexpr(),'lower_numerator':str(lower),'upper_numerator':str(upper),
            'denominator_power_two':128,'readout':float((lo+hi)/2)}


def main():
    parser=argparse.ArgumentParser();parser.add_argument('prepared');parser.add_argument('output')
    parser.add_argument('--compile-only',action='store_true');parser.add_argument('--contraction-only',action='store_true')
    parser.add_argument('--budget-ms',type=int,default=180000)
    parser.add_argument('--resume-witness',type=Path)
    parser.add_argument('--closure',choices=['fixed-point','global'],default='fixed-point')
    parser.add_argument('--engine',choices=['isolated-root','real-closed-field'],default='isolated-root')
    args=parser.parse_args();start=time.monotonic()
    path=Path(args.prepared)
    if not path.is_file():
        sequence=args.prepared
        path=Path(args.output).with_suffix('.prepared.json')
        native=Path(__file__).resolve().parent/'target/x86_64-unknown-linux-musl/release/vox'
        subprocess.run([str(native),'compile',sequence,'--input','protein','--prepare-only','--carrier',str(path)],check=True)
    with path.open(encoding='utf-8') as prepared_file:
        prepared=json.load(prepared_file)
    def source_digest():
        digest=hashlib.sha256()
        with path.open('rb') as source_file:
            for block in iter(lambda:source_file.read(8*1024*1024),b''):
                digest.update(block)
        return digest.hexdigest()
    # Explicit production input boundary. Existing positions, torques,
    # trajectories, fitted scores and NMR records are never read by Fold.
    source={key:prepared[key] for key in ['source_paths','source','pair_interaction_operator','shell_composition']}
    contraction=prepared.get('one_shot_lattice_contraction') or {}
    if not contraction.get('occurrence_reconstruction_closed') or not contraction.get('occurrence_pair_fibres_closed'):
        raise ValueError('prepared source lacks exact occurrence Möbius closure')
    if any(not isinstance(pair.get('source_occurrence_mobius_interactions'),str)
            for pair in prepared['pairs']):
        raise ValueError('prepared source lacks compact source-addressed occurrence tensors')
    source['pair_contraction_windings']=contraction['residue_character_windings']
    source['source_initial_windings']=contraction.get('source_initial_windings')
    source['shared_source_event_readout']=contraction.get('shared_source_event_readout',[])
    source['shared_source_event_pair_tensor_uses']=contraction.get('global_event_pair_tensor_uses',0)
    source['all_source_occurrences_rejoined']=contraction.get('all_source_occurrences_rejoined')
    source['pairs']=[{key:pair[key] for key in ['residues','source_register_pair_counts',
        'mobius_joint_interactions','source_occurrence_mobius_interactions']}
        |{'spatial_channels':[{key:c[key] for key in ['occupancies','count','signed_joint_terms','spin_counts']} for c in pair['spatial_channels']]}
        for pair in prepared['pairs']]
    fold=Fold(source,compile_geometry=not args.contraction_only)
    source['factorized_shared_event_tensor']=fold.shared_event_factorizations
    source['source_four_coordinates']=fold.source_four_coordinates
    source['shared_port_responses']=fold.shared_port_responses
    source['shared_port_contraction']=fold.shared_port_contraction
    source['contracted_source_four_coordinates']=fold.contracted_source_four_coordinates
    source['source_event_occurrence_uses']=fold.source_event_occurrence_uses
    source['source_sha256']=source_digest()
    factorized_event_sha256=hashlib.sha256(json.dumps(
        fold.shared_event_factorizations,separators=(',',':')).encode()).hexdigest()
    print('Whole source algebra assembled; compiling simultaneous closure',flush=True)
    output=Path(args.output)
    if args.engine=='isolated-root':
        source_binding=source_digest()
        formula_binding=f'whole-algebraic-source-relation/v1:{source_binding}'
        family_sha256=hashlib.sha256(formula_binding.encode()).hexdigest()
    else:
        family=fold.fixed_family() if args.closure=='fixed-point' else fold.minimum_family()
        formula=output.with_suffix('.smt2')
        solver=z.Solver();solver.set(timeout=args.budget_ms);solver.add(family)
        formula.write_text(solver.to_smt2())
        formula_binding=str(formula)
        family_sha256=hashlib.sha256(family.sexpr().encode()).hexdigest()
    carried_bank_rows=[]
    for pair in source['pairs']:
        i,j=pair['residues']
        for channel in pair['spatial_channels']:
            positive,negative=channel['spin_counts']
            if positive and negative:
                carried_bank_rows.append([i,j,channel['occupancies'][0],channel['occupancies'][1],
                    channel['count'],positive,negative])
    carried_bank_binding=hashlib.sha256(
        json.dumps(carried_bank_rows,separators=(',',':')).encode()).hexdigest()
    result={'schema':'whole-native-algebraic-fold/v1',
        'source':{'residues':[{'aa':r['aa']} for r in source['source']['residues']]},
        'source_sha256':source_digest(),
        'construction':{'domain':'complete native joint distribution and open-chain FOUR geometry',
            'unit_bond':1,'adjacent_bond_pairing':'3 dot(u,e)=1','terminal_cut':[fold.n-1,0],
            'residue_modulus':fold.n,'full_pair_count':len(source['pairs']),
            'mobius_inverse_exact':True,'frame':'FOUR tetrahedral characters',
            'source_occurrence_tensor':{'address_fields':['residue_pair','left_form','right_form','component','tick'],
                'record_encoding':'zlib-base64-json-records/v1, losslessly decoded per pair during contraction',
                'occurrence_count':fold.source_occurrence_count,
                'joint_states':fold.source_occurrence_joint_states,
                'signed_terms':fold.source_occurrence_signed_terms,
                'shared_ports':'each occurrence tensor contracts on the common [residue,support] distance variables',
                'pairwise_fibre_closure':'the occurrence Möbius inverses sum to each exact pair register before the whole relation closes',
                'address_binding':'all occurrence addresses and coefficients are bound by source_sha256'},
            'shared_source_event_tensor':{'event_address_fields':['residue_forms','component','global_tick'],
                'event_count':len(fold.source_events),'pair_tensor_uses':fold.source_event_pair_uses,
                'occurrence_tensor_references_consumed':fold.source_event_occurrence_uses,
                'factorization':'exact product of 20 normalized residue-port marginals; every addressed pair joint tensor is checked against its two common marginals',
                'factorized_event_tensor_sha256':factorized_event_sha256,
                'factorized_event_tensors':fold.shared_event_factorizations,
                'source_four_coordinates':fold.source_four_coordinates,
                'contracted_source_four_coordinates':fold.contracted_source_four_coordinates,
                'all_occurrences_rejoined':fold.source_event_rejoin_closed,
                'windings_sum_to_residue_return':True,
                'initial_windings':source['source_initial_windings'],
                'residue_return_windings':source['pair_contraction_windings'],
                'events':fold.source_events,
                'operation':'the single native contraction rejoins every source occurrence across all 20 residue ports; its event readouts select the shared FOUR chart family used by this whole-chain closure'},
            'shared_residue_port_contraction':fold.shared_port_contraction,
            'carried_bank_tensor':{'channels':len(carried_bank_rows),
                'records_sha256':carried_bank_binding,
                'role':'source-addressed preference/release readout carried through SIC; excluded from independent pair force terms'},
            'shell_fusion':source['shell_composition'],'bank':'all native release/preference coordinates close jointly',
            'closure':args.closure,
            'reentry':'the same source-bound closed family is returned under repeated closure',
            'singular_continuation':getattr(fold,'singular_continuation',None),
            'formula':formula_binding,'variables':len(fold.variables),
            'prepared_state':{key:prepared['self_description'] if key=='self_description' else prepared[key] for key in ['source_continuations','site_graphs','orbital_transfer','orbital_filling','ensemble_reentry','self_description','sic_carrier','glut_work','algebraic_preparation']},
            'distance_relations':len(fold.distances),'bank_cells':len(fold.bank),
            'bank_contraction':'simultaneous exact completed-square projection with both retained boundaries',
            'fixed_algebraic_constants':len(fold.constants)},
        'positions':None,'status':'compiled','compilation_seconds':time.monotonic()-start,
    'reentry':{'operator':'source-bound whole algebraic closure on the certified returned roots',
            'family_sha256':family_sha256,
        'law':'the prepared parent executes closure twice and compares the returned source-bound SIC observations'},
        'reference_structures_used':[]}
    output.write_text(json.dumps(result,indent=2)+'\n')
    print('Whole algebraic construction:',len(fold.variables),'variables;',len(fold.distances),'distance relations;',len(fold.bank),'banks',flush=True)
    if args.contraction_only:
        from whole_algebraic_readout import Whole
        readout=Whole(source,0,source.get('pair_contraction_windings'))
        if readout.shared_port_contraction!=fold.shared_port_contraction:
            raise ValueError('whole readout changed the source shared-port contraction')
        result['shared_port_contraction']=fold.shared_port_contraction
        result['construction']['shared_residue_port_contraction']=fold.shared_port_contraction
        result['construction']['shared_source_event_tensor']['contracted_source_four_coordinates']=fold.contracted_source_four_coordinates
        result['shared_port_readout_binding']={'verified':True,
            'response_coefficients_sha256':readout.shared_port_contraction['response_coefficients_sha256'],
            'SIXTEEN3_response_channels':len(readout.responses),
            'contracted_FOUR_ports':len(readout.contracted_source_four_coordinates)}
        output.write_text(json.dumps(result,indent=2)+'\n')
        print('Shared-port tensor contraction:',fold.shared_port_contraction['distinct_factorized_events'],
            'distinct event factors;',len(fold.shared_port_responses),'SIXTEEN_3 response channels',flush=True)
        return
    if args.compile_only:return
    if args.engine=='isolated-root':
        if args.closure!='fixed-point':raise ValueError('isolated root readout applies to the native fixed-point construction')
        from whole_algebraic_readout import solve
        began=time.monotonic()
        result.update(solve(source,args.resume_witness,output.with_suffix('.witness.json')))
        result['construction']['shared_residue_port_contraction']=result['shared_port_contraction']
        result['construction']['shared_source_event_tensor']['contracted_source_four_coordinates']=result['shared_port_contraction']['contracted_source_four_coordinates']
        result['closure_seconds']=time.monotonic()-began
        output.write_text(json.dumps(result,indent=2)+'\n')
        if result.get('closure_verified') is not True:
            print('Whole algebraic family remains held; source-bound solver diagnostics written',flush=True)
            return
        native=Path(__file__).resolve().parent/'target/x86_64-unknown-linux-musl/release/vox'
        subprocess.run([str(native),'compile','--whole-algebraic-return',str(output)],check=True)
        print('Whole algebraic family closed and re-entered through native SIC',flush=True)
        return
    began=time.monotonic();answer=solver.check();result['closure_seconds']=time.monotonic()-began
    result['status']=str(answer)
    if answer==z.sat:
        model=solver.model();points=[[algebraic_value(model,x) for x in p] for p in fold.points]
        result['exact_joint_assignment']={v.decl().name():algebraic_value(model,v) for v in fold.variables+fold.constants}
        result['algebraic_positions']=points;result['positions']=[[x['readout'] for x in p] for p in points]
        result['algebraic_bank']=[{'source':b['source'],'preference':algebraic_value(model,b['preference']),
            'release':algebraic_value(model,z.Sqrt(b['release_square']))} for b in fold.bank]
        result['algebraic_energy']=algebraic_value(model,fold.energy)
        values=[model.eval(v,model_completion=True) for v in fold.variables]
        if args.closure=='global':
            counter=z.Solver();counter.set(timeout=args.budget_ms)
            selected=z.substitute(fold.energy,*list(zip(fold.variables,values)))
            counter.add(fold.domain_formula,fold.energy<selected)
            answer_counter=counter.check()
            result['global_minimum_check']=str(answer_counter)
            result['closure_verified']=answer_counter==z.unsat
        else:
            closed=z.simplify(z.substitute(family,*list(zip(fold.variables+fold.constants,
                [model.eval(v,model_completion=True) for v in fold.variables+fold.constants]))))
            result['all_joint_equations_check']=closed.sexpr()
            result['closure_verified']=z.is_true(closed)
    elif answer==z.unknown:result['held_reason']=solver.reason_unknown();result['closure_verified']=False
    else:result['closure_verified']=False
    output.write_text(json.dumps(result,indent=2)+'\n');print('Whole algebraic return:',result['status'],flush=True)
    if result.get('closure_verified'):
        native=Path(__file__).resolve().parent/'target/x86_64-unknown-linux-musl/debug/vox'
        subprocess.run([str(native),'compile','--whole-algebraic-return',str(output)],check=True)


if __name__=='__main__':main()
