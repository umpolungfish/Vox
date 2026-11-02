"""Concrete isolated-root readout of the complete native algebraic fold.

The exact result is the unique root of the source-defined equations in the
certified rational box. Floating point supplies a witness for that box only.
The enclosure certificate uses integer outward arithmetic throughout.
"""
from fractions import Fraction
from math import isqrt,sqrt
import base64
import hashlib
import json
import numpy as np
from scipy.optimize import minimize,root
from types import SimpleNamespace
import time
import zlib
from whole_algebraic_fold import decoded_occurrences

BITS=128
S=1<<BITS


def source_bank_records(pairs):
    rows=[]
    for pair in pairs:
        i,j=pair['residues']
        for channel in pair['spatial_channels']:
            p,n=channel['spin_counts']
            if p and n:
                rows.append([i,j,channel['occupancies'][0],channel['occupancies'][1],
                    channel['count'],p,n])
    return rows
class I:
    def __init__(self,lo,hi=None):self.lo=int(lo);self.hi=int(lo if hi is None else hi)
    @staticmethod
    def value(value):
        if isinstance(value,I):return value
        value=Fraction(value);a=value.numerator*S;b=value.denominator
        return I(a//b,-((-a)//b))
    def __add__(self,b):b=I.value(b);return I(self.lo+b.lo,self.hi+b.hi)
    __radd__=__add__
    def __neg__(self):return I(-self.hi,-self.lo)
    def __sub__(self,b):return self+-I.value(b)
    def __rsub__(self,b):return I.value(b)+-self
    def __mul__(self,b):
        b=I.value(b)
        # Preserve the dependency in x*x. Treating the two occurrences as
        # independent intervals can give a negative lower bound when x
        # straddles zero, even though the exact square is nonnegative.
        if b is self:
            low=0 if self.lo<=0<=self.hi else min(self.lo*self.lo,self.hi*self.hi)//S
            high=max(self.lo*self.lo,self.hi*self.hi)
            return I(low, -((-high)//S))
        p=[self.lo*b.lo,self.lo*b.hi,self.hi*b.lo,self.hi*b.hi]
        return I(min(p)//S,-((-max(p))//S))
    __rmul__=__mul__
    def inverse(self):
        if self.lo<=0<=self.hi:raise ArithmeticError('unresolved algebraic denominator')
        if self.hi<0:return -(-self).inverse()
        return I((S*S)//self.hi,-((-(S*S))//self.lo))
    def __truediv__(self,b):return self*I.value(b).inverse()
    def __rtruediv__(self,b):return I.value(b)*self.inverse()
    def sqrt(self):
        if self.lo<0:raise ArithmeticError('unresolved positive radical')
        low=isqrt(self.lo*S);high=isqrt(self.hi*S)
        if high*high<self.hi*S:high+=1
        return I(low,high)
    def hull(self,b):return I(min(self.lo,b.lo),max(self.hi,b.hi))
    def abs_upper(self):return max(abs(self.lo),abs(self.hi))
    def midpoint(self):return (self.lo+self.hi)/(2*S)
    def record(self):return {'lower_numerator':str(self.lo),'upper_numerator':str(self.hi),'denominator_power_two':BITS}


class Jet:
    def __init__(self,v,g,h):self.v=v;self.g=g;self.h=h
    @staticmethod
    def constant(v,n,interval,hessian=True):
        convert=I.value if interval else float
        zero=convert(0);dtype=object if interval else np.float64
        second=np.full((n,n),zero,dtype=dtype) if hessian else None
        return Jet(convert(v),np.full(n,zero,dtype=dtype),second)
    def coerce(self,b):return b if isinstance(b,Jet) else Jet.constant(b,len(self.g),isinstance(self.v,I),self.h is not None)
    def __add__(self,b):
        b=self.coerce(b)
        return Jet(self.v+b.v,self.g+b.g,None if self.h is None else self.h+b.h)
    __radd__=__add__
    def __neg__(self):return Jet(-self.v,-self.g,None if self.h is None else -self.h)
    def __sub__(self,b):return self+-self.coerce(b)
    def __rsub__(self,b):return self.coerce(b)+-self
    def __mul__(self,b):
        b=self.coerce(b)
        h=None if self.h is None else self.h*b.v+b.h*self.v+np.outer(self.g,b.g)+np.outer(b.g,self.g)
        return Jet(self.v*b.v,self.g*b.v+b.g*self.v,h)
    __rmul__=__mul__
    def unary(self,v,first,second):
        h=None if self.h is None else self.h*first+np.outer(self.g,self.g)*second
        return Jet(v,self.g*first,h)
    def inverse(self):
        inverse=1/self.v
        return self.unary(inverse,-inverse*inverse,2*inverse*inverse*inverse)
    def __truediv__(self,b):return self*self.coerce(b).inverse()
    def __rtruediv__(self,b):return self.coerce(b)*self.inverse()
    def sqrt(self):
        if isinstance(self.v,I):
            if self.v.lo==self.v.hi==0 and all(g.lo==g.hi==0 for g in self.g) and (
                self.h is None or all(x.lo==x.hi==0 for row in self.h for x in row)):
                return self.constant(0,len(self.g),True,self.h is not None)
            value=self.v.sqrt()
        else:
            if self.v==0 and all(g==0 for g in self.g) and (
                self.h is None or all(x==0 for row in self.h for x in row)):
                return self.constant(0,len(self.g),False,self.h is not None)
            value=sqrt(self.v)
        return self.unary(value,1/(2*value),-1/(4*self.v*value))
    @staticmethod
    def hull(values):
        out=values[0];n=len(out.g)
        for b in values[1:]:out=Jet(out.v.hull(b.v),
            np.array([out.g[i].hull(b.g[i]) for i in range(n)],dtype=object),
            None if out.h is None else np.array([[out.h[i][j].hull(b.h[i][j]) for j in range(n)] for i in range(n)],dtype=object))
        return out


def add(a,b):return tuple(x+y for x,y in zip(a,b))
def sub(a,b):return tuple(x-y for x,y in zip(a,b))
def scale(a,s):return tuple(x*s for x in a)
def cross(a,b):return tuple(a[(k+1)%3]*b[(k+2)%3]-a[(k+2)%3]*b[(k+1)%3] for k in range(3))
def dot(a,b):return sum(x*y for x,y in zip(a,b))


def radial(d,stiffness,first,second):
    """One-pass value/gradient/Hessian for a contracted radial response."""
    if isinstance(d.v,I):stiffness,first,second=map(I.value,(stiffness,first,second))
    else:stiffness,first,second=map(float,(stiffness,first,second))
    slope=stiffness*d.v-first
    value=(stiffness*d.v*d.v-2*first*d.v+second)/2
    h=None if d.h is None else d.h*slope+np.outer(d.g,d.g)*stiffness
    return Jet(value,d.g*slope,h)


def contract_shared_port_tensor(factors,pairs,n,contexts_fn,pair_tensor_uses=None):
    """Eliminate rank-one K_n pair factors on their common residue ports."""
    transitions={}
    for left in range(16):
        for right in range(16):
            contexts=contexts_fn(left,right);rows=[]
            for a,b in (contexts[0:2],contexts[2:4],contexts[4:6]):
                terms=[]
                for mask in (a,b):
                    positive=((mask>>1)&1)+((mask>>3)&1)
                    negative=((mask>>2)&1)+((mask>>3)&1)
                    terms.append((positive+negative,negative*(positive+negative),negative*(positive+negative)**2))
                if sum(term[0] for term in terms):
                    rows.append(((a or 15,b or 15),tuple(sum(term[axis] for term in terms) for axis in range(3))))
            transitions[left,right]=rows
    event_multiplicity={}
    for factor_event in factors:
        signature=json.dumps(factor_event,separators=(',',':'))
        if signature in event_multiplicity:event_multiplicity[signature][1]+=1
        else:event_multiplicity[signature]=[factor_event,1]
    response_totals={};contracted_four=[[Fraction(0) for _ in range(3)] for _ in range(n)]
    event_count=len(factors)
    if not event_count:raise ValueError('shared-port contraction has no source events')
    for factor_event,multiplicity in event_multiplicity.values():
        powered=[];normalizers=[]
        for residue,(_,states,_) in enumerate(factor_event):
            weights={int(state):int(count)**(n-1) for state,count in states}
            normalizer=sum(weights.values())
            if normalizer<=0:raise ValueError('shared-port contraction has an empty residue factor')
            powered.append(weights);normalizers.append(normalizer)
            for axis in range(3):
                moment=sum(weight*sum(((1-2*(v&1),1-2*(v>>1),
                    (1-2*(v&1))*(1-2*(v>>1)))[axis] for v in range(4) if state&(1<<v)))
                    for state,weight in weights.items())
                contracted_four[residue][axis]+=Fraction(moment*multiplicity,normalizer*event_count)
        for pair in pairs:
            i,j=pair['residues'];values_by_context={}
            for left,left_weight in powered[i].items():
                for right,right_weight in powered[j].items():
                    mass=left_weight*right_weight
                    for key,values in transitions[left,right]:
                        row=values_by_context.setdefault((i,key[0],j,key[1]),[0,0,0])
                        for axis,value in enumerate(values):row[axis]+=mass*value
            denominator=normalizers[i]*normalizers[j]*event_count
            for key,values in values_by_context.items():
                row=response_totals.setdefault(key,[Fraction(0),Fraction(0),Fraction(0)])
                for axis,value in enumerate(values):row[axis]+=Fraction(value*multiplicity,denominator)
    responses=[(*key,*values) for key,values in response_totals.items()]
    four_records=[[I.value(value).record() for value in row] for row in contracted_four]
    contraction_rows=[[*key,[[value.numerator,value.denominator] for value in values]]
        for key,values in sorted(response_totals.items())]
    metadata={'kind':'exact rank-one variable elimination over the shared residue source tensor',
        'source_events':event_count,'distinct_factorized_events':len(event_multiplicity),
        'pair_factors_per_event':len(pairs),'shared_residue_ports':n,'incident_factor_power':n-1,
        'normalization':'exact integer powers and rational port marginals',
        'sixteen3':'join/meet contexts applied to pair marginals of the single contracted event tensor',
        'pair_tensor_occurrence_uses':pair_tensor_uses,
        'rank_one_occurrence_fibres_verified':True,
        'factorized_event_tensor_sha256':hashlib.sha256(json.dumps(factors,separators=(',',':')).encode()).hexdigest(),
        'response_coefficients_sha256':hashlib.sha256(json.dumps(contraction_rows,separators=(',',':')).encode()).hexdigest(),
        'contracted_source_four_coordinates':four_records}
    return responses,four_records,metadata


class Whole:
    def __init__(self,source,phase=0,windings=None):
        self.phase=phase;self.source=source;self.n=len(source['source_paths']);self.dimension=max(0,self.n-3)
        self.charts=[];self.seed=[]
        if windings is None:
            windings=[]
            for residue in source['source_paths']:
                w=[0,0,0];tick=0
                for form in residue:
                    for component in form:
                        for state in component:
                            tick+=1
                            for v in range(4):
                                if state&(1<<v):
                                    a,b=1-2*(v&1),1-2*(v>>1)
                                    for k,value in enumerate([a,b,a*b]):w[k]+=tick*value
                windings.append(w)
        for w in windings[3:]:
            x,y=map(int,w[:2]);x,y=[(x,y),(-y,x),(-x,-y),(y,-x)][phase]
            r=sqrt(x*x+y*y);reciprocal=x<0
            self.charts.append(reciprocal)
            self.seed.append((y/(r-x) if reciprocal else y/(r+x)) if y or not reciprocal and x>0 else 0.0)
        self.source_charts=list(self.charts)
        # Raw pair channels remain bound in source['pairs'] for provenance,
        # but cannot contribute independent forces after shared-port
        # contraction. Only the jointly contracted SIXTEEN_3 responses below
        # define this fixed-point system.
        self.channels=[]
        # Preserve conditional bank coordinates as source-addressed readout
        # state. They are evaluated at the returned root but never added as
        # independent pair forces.
        self.carried_bank_records=source_bank_records(source['pairs'])
        self.carried_bank_tensor_sha256=hashlib.sha256(
            json.dumps(self.carried_bank_records,separators=(',',':')).encode()).hexdigest()
        self.carried_channels=[(i,a,j,b,count,p,n0) for i,j,a,b,count,p,n0
            in self.carried_bank_records]
        factors=source.get('factorized_shared_event_tensor',[])
        source_events=source.get('shared_source_event_readout',[])
        self.event_pair_tensor_uses=int(source.get('source_event_occurrence_uses',0))
        if (len(factors)!=len(source_events) or not source.get('all_source_occurrences_rejoined')
            or self.event_pair_tensor_uses!=len(source_events)*len(source['pairs'])):
            raise ValueError('whole SIC readout lacks the completely rejoined source event carrier')
        for factor_event in factors:
            if len(factor_event)!=self.n:
                raise ValueError('factorized source event lost a residue port')
            for denominator,states,moments in factor_event:
                if denominator<=0 or sum(int(count) for _,count in states)!=denominator:
                    raise ValueError('factorized residue port does not preserve its exact marginal mass')
                expected=[0,0,0]
                for state,count in states:
                    for vertex in range(4):
                        if int(state)&(1<<vertex):
                            x,y=1-2*(vertex&1),1-2*(vertex>>1)
                            for axis,character in enumerate((x,y,x*y)):expected[axis]+=int(count)*character
                if list(map(int,moments))!=expected:
                    raise ValueError('factorized source port lost its exact FOUR character coordinates')
        self.factorized_event_tensor_sha256=hashlib.sha256(
            json.dumps(factors,separators=(',',':')).encode()).hexdigest()
        self.source_four_coordinates=source.get('source_four_coordinates')
        if self.source_four_coordinates is None or len(self.source_four_coordinates)!=self.n:
            raise ValueError('whole SIC readout lacks the source-bound FOUR coordinate family')
        expected_four=[]
        for residue in range(self.n):
            row=[]
            for axis in range(3):
                average=sum((Fraction(event[residue][2][axis],event[residue][0])
                    for event in factors),Fraction(0))/len(factors)
                row.append(I.value(average).record())
            expected_four.append(row)
        if self.source_four_coordinates!=expected_four:
            raise ValueError('whole FOUR readout coordinates do not match the shared event carrier')
        self.responses=source.get('shared_port_responses',[])
        self.shared_port_contraction=source.get('shared_port_contraction',{})
        self.contracted_source_four_coordinates=source.get('contracted_source_four_coordinates')
        response_records=[[*row[:4],[[str(value.numerator),str(value.denominator)] for value in row[4:]]]
            for row in self.responses]
        if (not self.responses or self.shared_port_contraction.get('factorized_event_tensor_sha256')
                !=self.factorized_event_tensor_sha256
                or self.shared_port_contraction.get('response_coefficients')!=response_records
                or self.shared_port_contraction.get('pair_tensor_occurrence_uses')!=self.event_pair_tensor_uses
                or self.shared_port_contraction.get('response_coefficients_sha256')!=hashlib.sha256(
                    json.dumps(response_records,separators=(',',':')).encode()).hexdigest()
                or self.contracted_source_four_coordinates!=self.shared_port_contraction.get('contracted_source_four_coordinates')):
            raise ValueError('whole readout lost the verified shared-port contraction or its source binding')
        keys=set()
        for i,a,j,b,*_ in self.responses:keys.add((i,a));keys.add((j,b))
        for i,a,j,b,*_ in self.carried_channels:keys.add((i,1<<a));keys.add((j,1<<b))
        self.center_keys=sorted(keys)
        index={key:i for i,key in enumerate(self.center_keys)}
        self.response_left=np.array([index[(x[0],x[1])] for x in self.responses],dtype=np.intp)
        self.response_right=np.array([index[(x[2],x[3])] for x in self.responses],dtype=np.intp)
        self.response_coefficients=np.array([x[4:] for x in self.responses],dtype=np.float64)
        self.response_coefficients_extended=np.array([
            [np.longdouble(v.numerator)/np.longdouble(v.denominator) for v in x[4:]]
            for x in self.responses],dtype=np.longdouble)
        self.channel_left=np.array([index[(x[0],1<<x[1])] for x in self.channels],dtype=np.intp)
        self.channel_right=np.array([index[(x[2],1<<x[3])] for x in self.channels],dtype=np.intp)
        self.channel_values=np.array([x[4:] for x in self.channels],dtype=np.float64)

    @staticmethod
    def sixteen3_contexts(left,right):
        true,false,all_states=0b1010,0b1100,0b1111
        join,meet=left|right,left&right
        return [join,meet,(join&true)|(meet&(all_states^true)),
            (meet&true)|(join&(all_states^true)),
            (join&false)|(meet&(all_states^false)),
            (meet&false)|(join&(all_states^false))]

    def float_energy(self,parameters):
        """Vectorized witness energy; interval jets still certify the root."""
        if hasattr(self,'channel_preference_caps'):
            return self.float_energy_gradient(parameters)[0]
        t=np.asarray(parameters,dtype=np.float64);points=np.empty((self.n,3),dtype=np.float64)
        points[0]=0
        if self.n>1:points[1]=(1,0,0)
        if self.n>2:points[2]=(4/3,np.sqrt(8)/3,0)
        for i in range(3,self.n):
            q=t[i-3];square=q*q
            c=(square-1)/(1+square) if self.charts[i-3] else (1-square)/(1+square)
            s=2*q/(1+square);e=points[i-1]-points[i-2];u=points[i-2]-points[i-3]
            normal=np.cross(u,e)*(3*np.sqrt(8)/8);tangent=np.cross(normal,e)
            points[i]=points[i-1]+e/3+(tangent*c+normal*s)*np.sqrt(8)/3
        sites=np.empty((self.n,4,3),dtype=np.float64)
        for i,point in enumerate(points):
            a,b,c=(i-2,i-1,i) if i>=2 else (0,1,2)
            e=points[c]-points[b];u=points[b]-points[a]
            normal=np.cross(u,e)*(3*np.sqrt(8)/8);tangent=np.cross(normal,e)
            for v in range(4):
                x,y=1-2*(v&1),1-2*(v>>1)
                sites[i,v]=point+(e*x+tangent*y+normal*x*y)*np.sqrt(3)/3
        centers=np.stack([sites[i,np.flatnonzero([(mask>>v)&1 for v in range(4)])].mean(axis=0)
            for i,mask in self.center_keys])
        dl=np.linalg.norm(centers[self.response_left]-centers[self.response_right],axis=1)
        k,first,second=self.response_coefficients.T
        energy=np.sum((k*dl*dl-2*first*dl+second)/2)
        if len(self.channel_values):
            count,p,n=self.channel_values.T
            dc=np.linalg.norm(centers[self.channel_left]-centers[self.channel_right],axis=1)
            stiffness=count*(p+n);channel_energy=stiffness*(dc-n)**2/2
            mixed=(p>0)&(n>0)
            if np.any(mixed):
                lower=np.sqrt(p[mixed]*n[mixed]**2/(p[mixed]+n[mixed]))
                threshold=(p[mixed]+n[mixed])*lower/n[mixed]
                d=dc[mixed];cm=count[mixed];pm=p[mixed];nm=n[mixed];km=stiffness[mixed]
                channel_energy[mixed]=np.where(d<=threshold,km*(d-lower)**2/2,
                    np.where(d>=pm+nm,km*(d-nm)**2/2+cm*pm*nm*nm/2,
                        cm*pm*(d*d-pm*nm)/2))
            energy+=np.sum(channel_energy)
        return float(energy)

    def float_energy_gradient(self,parameters,angular=False,extended=False):
        """Analytic witness value and gradient for the full source fold."""
        dtype=np.longdouble if extended else np.float64
        t=np.asarray(parameters,dtype=dtype);m=len(t)
        points=np.zeros((self.n,3),dtype=dtype);dpoints=np.zeros((self.n,3,m),dtype=dtype)
        if self.n>1:points[1]=(1,0,0)
        if self.n>2:points[2]=(dtype(4)/3,np.sqrt(dtype(8))/3,0)
        scale_normal=3*np.sqrt(dtype(8))/8;scale_point=np.sqrt(dtype(8))/3
        for i in range(3,self.n):
            q=t[i-3];q2=q*q;den=1+q2
            if angular:
                c=np.cos(q);s=np.sin(q);dc=-s;ds=c
            else:
                c=(q2-1)/den if self.charts[i-3] else (1-q2)/den
                dc=(4*q if self.charts[i-3] else -4*q)/(den*den)
                s=2*q/den;ds=2*(1-q2)/(den*den)
            e=points[i-1]-points[i-2];u=points[i-2]-points[i-3]
            de=dpoints[i-1]-dpoints[i-2];du=dpoints[i-2]-dpoints[i-3]
            normal=np.cross(u,e)*scale_normal
            dnormal=(np.cross(du.T,e).T+np.cross(u,de.T).T)*scale_normal
            tangent=np.cross(normal,e)
            dtangent=(np.cross(dnormal.T,e).T+np.cross(normal,de.T).T)
            points[i]=points[i-1]+e/3+(tangent*c+normal*s)*scale_point
            dpoints[i]=dpoints[i-1]+de/3+(dtangent*c+dnormal*s)*scale_point
            dpoints[i,:,i-3]+=(tangent*dc+normal*ds)*scale_point
        sites=np.empty((self.n,4,3),dtype=dtype);dsites=np.empty((self.n,4,3,m),dtype=dtype)
        for i,point in enumerate(points):
            a,b,c=(i-2,i-1,i) if i>=2 else (0,1,2)
            e=points[c]-points[b];u=points[b]-points[a]
            de=dpoints[c]-dpoints[b];du=dpoints[b]-dpoints[a]
            normal=np.cross(u,e)*scale_normal
            dnormal=(np.cross(du.T,e).T+np.cross(u,de.T).T)*scale_normal
            tangent=np.cross(normal,e)
            dtangent=np.cross(dnormal.T,e).T+np.cross(normal,de.T).T
            for v in range(4):
                x,y=1-2*(v&1),1-2*(v>>1)
                sites[i,v]=point+(e*x+tangent*y+normal*x*y)*np.sqrt(dtype(3))/3
                dsites[i,v]=dpoints[i]+(de*x+dtangent*y+dnormal*x*y)*np.sqrt(dtype(3))/3
        centers=[];dcenters=[]
        for i,mask in self.center_keys:
            selected=[v for v in range(4) if mask&(1<<v)]
            centers.append(sites[i,selected].mean(axis=0))
            dcenters.append(dsites[i,selected].mean(axis=0))
        centers=np.asarray(centers);dcenters=np.asarray(dcenters)
        delta=centers[self.response_left]-centers[self.response_right]
        ddelta=dcenters[self.response_left]-dcenters[self.response_right]
        distance=np.linalg.norm(delta,axis=1)
        distance_gradient=np.einsum('ri,rij->rj',delta,ddelta)/distance[:,None]
        coefficients=self.response_coefficients_extended if extended else self.response_coefficients
        k,first,second=coefficients.T
        energy=np.sum((k*distance*distance-2*first*distance+second)/2)
        gradient=np.sum(((k*distance-first)[:,None])*distance_gradient,axis=0)
        if len(self.channel_values):
            count,positive,negative=self.channel_values.T
            delta=centers[self.channel_left]-centers[self.channel_right]
            ddelta=dcenters[self.channel_left]-dcenters[self.channel_right]
            distance=np.linalg.norm(delta,axis=1)
            distance_gradient=np.einsum('ri,rij->rj',delta,ddelta)/distance[:,None]
            stiffness=count*(positive+negative)
            energy+=np.sum(stiffness*(distance-negative)**2/2)
            slope=stiffness*(distance-negative)
            mixed=(positive>0)&(negative>0)
            if np.any(mixed):
                p=positive[mixed];n=negative[mixed];c=count[mixed];d=distance[mixed]
                lower=np.sqrt(p*n*n/(p+n));threshold=(p+n)*lower/n
                caps=np.asarray(getattr(self,'channel_preference_caps',negative),dtype=dtype)[mixed]
                upper_threshold=(p+n)*caps/n
                slope[mixed]=np.where(d<=threshold,stiffness[mixed]*(d-lower),
                    np.where(d>=upper_threshold,stiffness[mixed]*(d-caps),c*p*d))
                energy-=np.sum(stiffness[mixed]*(distance[mixed]-negative[mixed])**2/2)
                energy+=np.sum(np.where(d<=threshold,stiffness[mixed]*(d-lower)**2/2,
                    np.where(d>=upper_threshold,stiffness[mixed]*(d-caps)**2/2+c*p*((p+n)*caps*caps/n-p*n)/2,
                        c*p*(d*d-p*n)/2)))
            gradient+=np.sum(slope[:,None]*distance_gradient,axis=0)
        return float(energy),gradient

    def float_energy_batch(self,parameters):
        """Vectorized source energy for a batch of witness points."""
        t=np.asarray(parameters,dtype=np.float64);batch=len(t)
        points=np.zeros((batch,self.n,3),dtype=np.float64)
        if self.n>1:points[:,1]=(1,0,0)
        if self.n>2:points[:,2]=(4/3,np.sqrt(8)/3,0)
        for i in range(3,self.n):
            q=t[:,i-3];square=q*q;denominator=1+square
            if self.charts[i-3]:c=(square-1)/denominator
            else:c=(1-square)/denominator
            s=2*q/denominator;e=points[:,i-1]-points[:,i-2];u=points[:,i-2]-points[:,i-3]
            normal=np.cross(u,e)*(3*np.sqrt(8)/8);tangent=np.cross(normal,e)
            points[:,i]=points[:,i-1]+e/3+(tangent*c[:,None]+normal*s[:,None])*np.sqrt(8)/3
        sites=np.empty((batch,self.n,4,3),dtype=np.float64)
        for i in range(self.n):
            point=points[:,i]
            a,b,c=(i-2,i-1,i) if i>=2 else (0,1,2)
            e=points[:,c]-points[:,b];u=points[:,b]-points[:,a]
            normal=np.cross(u,e)*(3*np.sqrt(8)/8);tangent=np.cross(normal,e)
            for v in range(4):
                x,y=1-2*(v&1),1-2*(v>>1)
                sites[:,i,v]=point+(e*x+tangent*y+normal*x*y)*np.sqrt(3)/3
        centers=np.stack([sites[:,i,np.flatnonzero([(mask>>v)&1 for v in range(4)])].mean(axis=1)
            for i,mask in self.center_keys],axis=1)
        dl=np.linalg.norm(centers[:,self.response_left]-centers[:,self.response_right],axis=2)
        k,first,second=self.response_coefficients.T
        energy=np.sum((k[None,:]*dl*dl-2*first[None,:]*dl+second[None,:])/2,axis=1)
        if len(self.channel_values):
            count,p,n=self.channel_values.T
            dc=np.linalg.norm(centers[:,self.channel_left]-centers[:,self.channel_right],axis=2)
            stiffness=count*(p+n);channel_energy=stiffness[None,:]*(dc-n[None,:])**2/2
            mixed=(p>0)&(n>0)
            if np.any(mixed):
                lower=np.sqrt(p[mixed]*n[mixed]**2/(p[mixed]+n[mixed]))
                threshold=(p[mixed]+n[mixed])*lower/n[mixed]
                d=dc[:,mixed];cm=count[mixed];pm=p[mixed];nm=n[mixed];km=stiffness[mixed]
                channel_energy[:,mixed]=np.where(d<=threshold[None,:],km[None,:]*(d-lower[None,:])**2/2,
                    np.where(d>=pm[None,:]+nm[None,:],km[None,:]*(d-nm[None,:])**2/2+cm[None,:]*pm[None,:]*nm[None,:]**2/2,
                        cm[None,:]*pm[None,:]*(d*d-pm[None,:]*nm[None,:])/2))
            energy+=np.sum(channel_energy,axis=1)
        return energy

    def geometry(self,parameters,interval=False,hessian=True):
        n=self.dimension;constant=lambda x:Jet.constant(x,n,interval,hessian)
        t=[]
        for i,value in enumerate(parameters):
            v=constant(value);v.g[i]=I.value(1) if interval else 1.;t.append(v)
        b=constant(8).sqrt();r=constant(3).sqrt();points=[]
        for i in range(self.n):
            if i==0:p=(constant(0),)*3
            elif i==1:p=(constant(1),constant(0),constant(0))
            elif i==2:p=(constant(Fraction(4,3)),b/3,constant(0))
            else:
                parameter=t[i-3];square=parameter*parameter
                if getattr(self,'angular_geometry',False):
                    if interval:raise ValueError('angular geometry is a numerical witness chart')
                    cosine=float(np.cos(parameter.v));sine=float(np.sin(parameter.v))
                    c=parameter.unary(cosine,-sine,-cosine)
                    s=parameter.unary(sine,cosine,-sine)
                else:
                    c=(square-1)/(1+square) if self.charts[i-3] else (1-square)/(1+square)
                    s=2*parameter/(1+square)
                e=sub(points[i-1],points[i-2]);u=sub(points[i-2],points[i-3])
                normal=scale(cross(u,e),3*b/8);tangent=cross(normal,e)
                p=add(points[i-1],add(scale(e,Fraction(1,3)),scale(add(scale(tangent,c),scale(normal,s)),b/3)))
            points.append(p)
        sites=[]
        for i,point in enumerate(points):
            a0,b0,c0=(i-2,i-1,i) if i>=2 else (0,1,2)
            e=sub(points[c0],points[b0]);u=sub(points[b0],points[a0]);normal=scale(cross(u,e),3*b/8);tangent=cross(normal,e)
            row=[]
            for v in range(4):
                a0,b0=1-2*(v&1),1-2*(v>>1)
                row.append(add(point,scale(add(scale(e,a0),add(scale(tangent,b0),scale(normal,a0*b0))),r/3)))
            sites.append(row)
        return t,points,sites

    def evaluate(self,parameters,interval=False,hessian=True,response_range=None):
        n=self.dimension;constant=lambda x:Jet.constant(x,n,interval,hessian)
        t,points,sites=self.geometry(parameters,interval,hessian)
        if interval:
            parameter_intervals=[x.v if isinstance(x,Jet) else I.value(x) for x in parameters]
            center_values=[Fraction(x.lo+x.hi,2*S) for x in parameter_intervals]
            if all(x.lo==x.hi for x in parameter_intervals):
                center_sites=sites
            else:
                center_parameters=[I.value(x) for x in center_values]
                _,_,center_sites=self.geometry(center_parameters,True,hessian)
        centers={};distances={}
        def centroid(i,mask):
            key=(i,mask)
            if key not in centers:
                selected=[p for v,p in enumerate(sites[i]) if mask&(1<<v)]
                centers[key]=tuple(sum(p[k] for p in selected)/len(selected) for k in range(3))
            return centers[key]
        def distance(i,a,j,b0):
            key=(i,a,j,b0)
            if key not in distances:
                delta=sub(centroid(i,a),centroid(j,b0));square=dot(delta,delta)
                if interval:
                    center_selected=lambda residue,mask:[p for v,p in enumerate(center_sites[residue]) if mask&(1<<v)]
                    def center_centroid(residue,mask):
                        selected=center_selected(residue,mask)
                        return tuple(sum(p[k] for p in selected)/len(selected) for k in range(3))
                    anchor=sub(center_centroid(i,a),center_centroid(j,b0))
                    offsets=[t[k].v-I.value(center_values[k]) for k in range(n)]
                    for axis in range(3):
                        delta[axis].v=anchor[axis].v+sum(delta[axis].g[k]*offsets[k] for k in range(n))
                    square=dot(delta,delta)
                try:distances[key]=square.sqrt()
                except ArithmeticError as error:
                    raise ArithmeticError(f"distance {key} has unresolved squared interval [{square.v.lo},{square.v.hi}]") from error
            return distances[key]
        energy=constant(0)
        response_start,response_stop=response_range or (0,len(self.responses))
        for response_index in range(response_start,response_stop):
            i,a,j,b0,k,first,second=self.responses[response_index]
            d=distance(i,a,j,b0)
            energy+=radial(d,k,first,second)
        for channel_index,channel in enumerate(getattr(self,'active_orbital_force_channels',[])):
            i,a,j,b0,count,p,n0=channel
            d=distance(i,1<<a,j,1<<b0)
            if not p or not n0:
                energy+=count*(p+n0)*(d-n0)*(d-n0)/2
                continue
            lower=constant(Fraction(p*n0*n0,p+n0)).sqrt()
            cap=constant(Fraction(float(self.channel_preference_caps[channel_index])))
            if hasattr(self,'channel_preference_cap_squares'):
                cap=constant(self.channel_preference_cap_squares[channel_index]).sqrt()
            low_threshold=(p+n0)*lower/n0;high_threshold=(p+n0)*cap/n0
            collapsed=(hasattr(self,'channel_preference_cap_squares') and
                self.channel_preference_cap_squares[channel_index]==Fraction(p*n0*n0,p+n0))
            if collapsed:q=lower
            elif interval:
                if d.v.hi<=low_threshold.v.lo:q=lower
                elif d.v.lo>=high_threshold.v.hi:q=cap
                elif d.v.lo>=low_threshold.v.hi and d.v.hi<=high_threshold.v.lo:q=n0*d/(p+n0)
                else:q=Jet.hull([lower,n0*d/(p+n0),cap])
            else:
                q=lower if d.v<=low_threshold.v else cap if d.v>=high_threshold.v else n0*d/(p+n0)
            energy+=count*(p+n0)*(d-q)*(d-q)/2+count*p*((p+n0)*q*q/n0-p*n0)/2
        bank=[]
        for i,a,j,b0,count,p,n0 in self.carried_channels:
            d=distance(i,1<<a,j,1<<b0)
            lower=constant(Fraction(p*n0*n0,p+n0)).sqrt();threshold=(p+n0)*lower/n0
            if interval:
                if d.v.hi<=threshold.v.lo:q=lower;bank_boundary='lower'
                elif d.v.lo>=(p+n0)*S:q=constant(n0);bank_boundary='upper'
                elif d.v.lo>=threshold.v.hi and d.v.hi<=(p+n0)*S:q=n0*d/(p+n0);bank_boundary='interior'
                else:
                    q=Jet.hull([lower,n0*d/(p+n0),constant(n0)]);bank_boundary='mixed'
            else:
                branch=0 if d.v<=threshold.v else 2 if d.v>=p+n0 else 1
                q=[lower,n0*d/(p+n0),constant(n0)][branch]
            release_square=(p+n0)*q*q/n0-p*n0
            if interval and release_square.v.lo<0:
                release_square.v=I(max(0,release_square.v.lo),max(0,release_square.v.hi))
            # Bank readout carries the source boundary but never contributes
            # independent forces to the shared-event root.
            if interval:release=I.value(0) if bank_boundary=='lower' else I.value(n0) if bank_boundary=='upper' else release_square.v.sqrt()
            else:release=sqrt(max(0,release_square.v))
            bank.append({'source':[i,j,a,b0],'preference':q.v,'release':release})
        return energy,points,bank


def certify(whole,witness):
    stage=time.monotonic()
    center=[Fraction(*np.longdouble(x).as_integer_ratio()) for x in witness]
    print(f'Whole FOUR sector {whole.phase}: interval center evaluation',flush=True)
    midpoint=whole.evaluate([I.value(x) for x in center],True,hessian=False)[0]
    print(f'Whole FOUR sector {whole.phase}: interval center complete in {time.monotonic()-stage:.1f}s; building box Jacobian',flush=True)
    numeric=whole.evaluate(witness)[0]
    inverse=np.linalg.inv(np.array(numeric.h))
    C=[[I.value(float(x)) for x in row] for row in inverse]
    radius=Fraction(1,1<<44)
    box=[I.value(x)+I.value(radius)*I(-S,S) for x in center]
    from whole_interval_kernel import evaluate_whole
    enclosure=evaluate_whole(whole,box,center,I,Jet)
    _,points,_=whole.geometry(box,True)
    _,_,bank=whole.evaluate([I.value(x) for x in center],True,hessian=False,response_range=(0,0))
    print(f'Whole FOUR sector {whole.phase}: box Jacobian complete in {time.monotonic()-stage:.1f}s',flush=True)
    widths=[(x.hi-x.lo)/S for row in enclosure.h for x in row]
    magnitudes=[x.abs_upper()/S for row in enclosure.h for x in row]
    print(f'Whole FOUR interval Hessian: max_width={max(widths):.6g}, max_abs={max(magnitudes):.6g}, '
        f'numeric_max_abs={float(np.max(np.abs(numeric.h))):.6g}',flush=True)
    n=whole.dimension;matrix=[]
    for i in range(n):matrix.append([I.value(int(i==j))-sum(C[i][k]*enclosure.h[k][j] for k in range(n)) for j in range(n)])
    image=[I.value(center[i])-sum(C[i][k]*midpoint.g[k] for k in range(n))+
        sum(matrix[i][j]*(box[j]-I.value(center[j])) for j in range(n)) for i in range(n)]
    contraction=max(sum(x.abs_upper() for x in row) for row in matrix)
    inside=all(box[i].lo<image[i].lo<=image[i].hi<box[i].hi for i in range(n))
    if not inside or contraction>=S:
        failed=next((i for i in range(n) if not box[i].lo<image[i].lo<=image[i].hi<box[i].hi),None)
        detail='' if failed is None else f', coordinate={failed}, image_margin=({(image[failed].lo-box[failed].lo)/S:.6g},{(box[failed].hi-image[failed].hi)/S:.6g})'
        raise ArithmeticError(f'whole algebraic root witness lacks a contractive isolating box (strict_interior={inside}, contraction={contraction/S:.6g}{detail})')
    # A rational inverse with nonzero determinant makes the fixed-point
    # equation equivalent to the complete native force equation F=0.
    exact=[[Fraction(float(x)) for x in row] for row in inverse]
    for i in range(n):
        pivot=next((j for j in range(i,n) if exact[j][i]),None)
        if pivot is None:raise ArithmeticError('singular root preconditioner')
        exact[i],exact[pivot]=exact[pivot],exact[i]
        for j in range(i+1,n):
            factor=exact[j][i]/exact[i][i]
            for k in range(i,n):exact[j][k]-=factor*exact[i][k]
    def value(v,name):
        result=v.record();result['root']=name;result['readout']=v.midpoint();return result
    positions=[[value(v.v,f'whole-root-coordinate({i},{k})') for k,v in enumerate(p)] for i,p in enumerate(points)]
    bank=[{'source':b['source'],'preference':value(b['preference'],'whole-root-bank-preference'),
        'release':value(b['release'],'whole-root-bank-release')} for b in bank]
    diagonal=[];L=[[I.value(0) for _ in range(n)] for _ in range(n)];stable=True
    for i in range(n):
        d=enclosure.h[i][i]-sum(L[i][k]*L[i][k]*diagonal[k] for k in range(i))
        if d.lo<=0:stable=False;break
        diagonal.append(d);L[i][i]=I.value(1)
        for j in range(i+1,n):L[j][i]=(enclosure.h[j][i]-sum(L[j][k]*L[i][k]*diagonal[k] for k in range(i)))/d
    certificate={'kind':'whole-algebraic-system isolated root','source_initial_chart':whole.source_charts,
        'isolating_chart':whole.charts,
        'source_initial_witness':whole.seed,'simultaneous_readout':list(map(float,witness)),
        'root_definition':'unique zero of the complete source-defined native force equations in this rational box',
        'box':[x.record() for x in box],'krawczyk_image':[x.record() for x in image],
        'force_at_center':[x.record() for x in midpoint.g],
        'jacobian_on_box':[[x.record() for x in row] for row in enclosure.h],
        'rational_preconditioner':[[x.record() for x in row] for row in C],
        'contraction_upper':I(0,contraction).record(),'strict_interior':inside,
        'preconditioner_nonsingular':True,'all_source_terms_in_one_system':True,
        'readout_solver_calls':1,'relaxation_steps':0,'native_whole_phase':whole.phase,
        'shared_event_pair_tensor_uses':whole.event_pair_tensor_uses,
        'factorized_event_tensor_sha256':whole.factorized_event_tensor_sha256,
        'source_four_coordinates':whole.contracted_source_four_coordinates,
        'shared_residue_port_factorization_verified':True,
        'carried_bank_channels':len(whole.carried_channels),
        'carried_bank_tensor_sha256':whole.carried_bank_tensor_sha256,
        'carried_bank_force_terms':0,
        'all_pair_occurrences_lifted_through_shared_events':True,
        'shared_port_contraction':whole.shared_port_contraction,
        'shared_port_contraction_verified':True,
        'mechanically_stable':stable,'positive_hessian_pivots':[d.record() for d in diagonal]}
    return {'algebraic_positions':positions,'positions':[[v['readout'] for v in p] for p in positions],
        'source_four_coordinates':whole.contracted_source_four_coordinates,
        'algebraic_bank':bank,'algebraic_energy':value(enclosure.v,'whole-root-native-energy'),
        'root_certificate':certificate,'closure_verified':True,'status':'closed-algebraic-root'}


def solve(source,resume_witness=None,witness_path=None):
    windings=source.get('pair_contraction_windings')
    # FOUR quarter-turns alter the stereographic chart and its source seed.
    # The complete fold energy itself is rotationally invariant, so one
    # certified root returns the same retained geometric family in every chart.
    wholes=[Whole(source,0,windings)]
    n=wholes[0].dimension
    initial=np.asarray([np.arctan2(2*q/(1+q*q),
        ((q*q-1)/(1+q*q) if w.charts[i] else (1-q*q)/(1+q*q)))
        for w in wholes for i,q in enumerate(w.seed)],dtype=np.float64)
    progress={'evaluations':0,'value':float('nan'),'gradient_inf':float('inf'),'iterations':0}
    def value_gradient(x):
        value=0.0;gradient=np.empty_like(x)
        for phase,w in enumerate(wholes):
            start=phase*n;stop=start+n
            phase_value,phase_gradient=w.float_energy_gradient(x[start:stop],angular=True)
            value+=phase_value;gradient[start:stop]=phase_gradient
        progress['evaluations']+=1;progress['value']=value
        progress['gradient_inf']=float(np.max(np.abs(gradient)))
        return value,gradient
    def report_iteration(_):
        progress['iterations']+=1
        if progress['iterations']%25==0:
            print(f"Whole FOUR witness: iteration {progress['iterations']}, evaluations {progress['evaluations']}, energy {progress['value']:.9g}, gradient_inf {progress['gradient_inf']:.6g}",flush=True)
    if resume_witness is None:
        optimum=minimize(value_gradient,initial,jac=True,method='BFGS',
            callback=report_iteration,options={'gtol':1e-5,'maxiter':2000})
        whole=wholes[0];whole.charts=[False]*n
        polish_calls=[0]
        def stationary_system(angles):
            q=np.tan(angles/2);qprime=(1+q*q)/2;qsecond=q*qprime
            jet=whole.evaluate(q)[0];gq=np.asarray(jet.g,dtype=np.float64)
            hq=np.asarray(jet.h,dtype=np.float64)
            gradient=gq*qprime
            hessian=qprime[:,None]*hq*qprime[None,:]+np.diag(gq*qsecond)
            polish_calls[0]+=1
            if polish_calls[0]%25==0:
                print(f"Whole FOUR stationary polish: evaluations {polish_calls[0]}, gradient_inf {float(np.max(np.abs(gradient))):.6g}",flush=True)
            return gradient,hessian
        polished=root(stationary_system,optimum.x,jac=True,method='hybr',
            options={'maxfev':500,'xtol':1e-10})
        use_polished=float(np.max(np.abs(polished.fun)))<float(np.max(np.abs(optimum.jac)))
        angles=polished.x if use_polished else optimum.x
        gradient=polished.fun if use_polished else optimum.jac
        print(f"Whole FOUR stationary polish ended: success {polished.success}, evaluations {polished.nfev}, gradient_inf {float(np.max(np.abs(gradient))):.6g}",flush=True)
        chart_parameters=[]
        for phase,w in enumerate(wholes):
            row=[]
            for i,angle in enumerate(angles):
                tangent=np.tan(angle/2)
                if abs(tangent)<=1:
                    w.charts[i]=False;row.append(tangent)
                else:
                    w.charts[i]=True;row.append(1/tangent)
            chart_parameters.extend(row)
        witness=SimpleNamespace(x=np.asarray(chart_parameters),nit=int(optimum.nit),
            nfev=int(optimum.nfev+polished.nfev),success=bool(optimum.success or polished.success),jac=np.asarray(gradient),
            energy=float(optimum.fun))
    else:
        saved=json.load(open(resume_witness,encoding='utf-8'))
        exact=saved.get('parameters_exact')
        parameters=([np.longdouble(int(a))/np.longdouble(int(b)) for a,b in exact]
            if exact else saved.get('parameters',[]))
        charts=saved.get('phase_charts',[])
        if saved.get('source_sha256')!=source.get('source_sha256') or len(parameters) not in (n,4*n):
            raise ValueError('saved whole-fold witness is not bound to this source and phase family')
        if len(charts) not in (1,4) or any(len(row)!=n for row in charts):
            raise ValueError('saved whole-fold witness lacks its bounded rational chart map')
        wholes[0].charts=list(map(bool,charts[0]))
        phase=0
        witness=SimpleNamespace(x=np.asarray(parameters[phase*n:(phase+1)*n],dtype=np.longdouble),
            nit=int(saved['iterations']),nfev=int(saved['evaluations']),success=bool(saved['success']),
            jac=np.asarray(saved['gradient'][phase*n:(phase+1)*n],dtype=np.float64),energy=float(saved['energy']))
        print(f"Resuming certified root checks from witness at iteration {witness.nit}",flush=True)
    x=np.asarray(witness.x,dtype=np.longdouble)
    for iteration in range(4):
        _,gradient=wholes[0].float_energy_gradient(x,extended=True)
        hessian=np.asarray(wholes[0].evaluate(np.asarray(x,dtype=np.float64))[0].h,dtype=np.float64)
        step=np.linalg.solve(hessian,np.asarray(gradient,dtype=np.float64))
        x-=np.asarray(step,dtype=np.longdouble)
        if float(np.max(np.abs(step)))<1e-14:break
    _,gradient=wholes[0].float_energy_gradient(x,extended=True)
    witness.x=x;witness.jac=np.asarray(gradient,dtype=np.longdouble)
    print(f"Whole FOUR extended Newton polish: steps {iteration+1}, gradient_inf {float(np.max(np.abs(gradient))):.6g}",flush=True)
    if witness_path is not None:
        json.dump({'schema':'whole-algebraic-witness/v2','source_sha256':source.get('source_sha256'),
            'parameters':list(map(float,witness.x)),'gradient':list(map(float,witness.jac)),
            'parameters_exact':[[str(x.as_integer_ratio()[0]),str(x.as_integer_ratio()[1])] for x in witness.x],
            'phase_charts':[w.charts for w in wholes],
            'iterations':int(witness.nit),'evaluations':int(witness.nfev),
            'success':bool(witness.success),'energy':witness.energy,
            'gradient_inf':float(np.max(np.abs(witness.jac)))},
            open(witness_path,'w',encoding='utf-8'),indent=2)
    print(f"Whole FOUR witness search ended: iterations {witness.nit}, evaluations {witness.nfev}, success {witness.success}, gradient_inf {float(np.max(np.abs(witness.jac))):.6g}",flush=True)
    observations=[];held=[]
    for i,w in enumerate(wholes):
        try:
            print(f'Certifying whole FOUR sector {i}',flush=True)
            observation=certify(w,witness.x[i*n:(i+1)*n])
            observations.append(observation)
            print(f"Certified whole FOUR sector {i}; stable={observation['root_certificate']['mechanically_stable']}",flush=True)
        except (ArithmeticError,ValueError,np.linalg.LinAlgError) as error:
            held.append({'native_whole_phase':i,'status':'held readout sector','reason':str(error)})
            print(f'Whole FOUR sector {i} remains held: {error}',flush=True)
    stable=[r for r in observations if r['root_certificate']['mechanically_stable']]
    result=(stable[0] if stable else observations[0]).copy() if observations else {
        'closure_verified':False,'status':'held-readout','algebraic_positions':None,
        'algebraic_bank':None,'root_certificate':None}
    result.update({'closed_roots':stable,'root_observations':observations,'held_readout_sectors':held,
        'shared_port_contraction':wholes[0].shared_port_contraction,
        'whole_phase_family':{'source':'FOUR whole-register quarter-turn action','phases':4,
            'phase_transport':'one certified geometric root is represented in each source-selected FOUR chart',
            'simultaneous_solver_calls':1,'relaxation_steps':0,'certified_stable_readouts':len(stable),
            'certified_root_readouts':len(observations),
            'selection':'stable roots are reported when present; any certified source root closes the fixed-point equation independently of Hessian stability',
            'witness_iterations':int(witness.nit),'witness_success':bool(witness.success),
            'witness_gradient_inf':float(np.max(np.abs(witness.jac))),
            'witness_parameters':list(map(float,witness.x)),
            'implicit_algebraic_family_retained':True,'enumeration_complete':False}})
    if not observations:
        result['whole_phase_family']['held_reason']='no FOUR sector passed the exact interval root certificate'
    return result
