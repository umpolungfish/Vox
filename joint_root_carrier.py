"""Prepare the enclosed joint mechanical root and its retained source bank."""
from fractions import Fraction
import hashlib
import json
from pathlib import Path
import sys
from joint_geometry_return import numerical_operator
from whole_algebraic_readout import I


def prepare(equation_path,source_path,relation_path,enclosure_path,witness_path,output):
    load=lambda p:json.loads(p.read_text())
    equations,source,relation,certificate,witness=map(load,(equation_path,source_path,relation_path,enclosure_path,witness_path))
    sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
    if certificate['current_carrier_sha256']!=sha(source_path):raise ValueError('root carrier source changed')
    if not certificate['strict_inclusion'] or not certificate['positive_mechanical_congruence']['verified']:
        raise ValueError('complete mechanical root is not enclosed')
    for box,image in zip(certificate['box'],certificate['image']):
        if not (int(box['lower_numerator'])<int(image['lower_numerator']) and
                int(image['upper_numerator'])<int(box['upper_numerator'])):
            raise ValueError('root image is not strictly inside the carried box')
    if relation['source_sha256']!=equations['source_sha256']:raise ValueError('root source relation changed')
    whole=numerical_operator(equations,source);whole.charts=witness['charts']
    box=[I(int(x['lower_numerator']),int(x['upper_numerator'])) for x in certificate['box']]
    _,points,sites=whole.geometry(box,True,hessian=False)
    bank=[]
    for cell in source['glut_work']['cells']:
        i,j,a,b=cell['source'];p,n=cell['positive'],cell['negative'];span=p+n
        delta=[sites[i][a][axis]-sites[j][b][axis] for axis in range(3)]
        distance=sum(x*x for x in delta).sqrt().v
        z=Fraction(cell['release_coordinate'])
        cap_square=Fraction(n,span)*(p*n+z*z)
        lower=I.value(Fraction(p*n*n,span)).sqrt();cap=I.value(cap_square).sqrt()
        # Match the native exact integer quotient, avoiding a widened
        # reciprocal multiplication in the prepared conditional expression.
        target=I(distance.lo*n//span,-((-distance.hi*n)//span))
        q=I(min(cap.lo,max(lower.lo,target.lo)),min(cap.hi,max(lower.hi,target.hi)))
        square=span*q*q/n-p*n
        release=I(max(0,square.lo),max(0,square.hi)).sqrt()
        bank.append({'source':cell['source'],'multiplicity':cell['multiplicity'],
            'positive':p,'negative':n,'source_release_coordinate':str(z),
            'preference':q.record(),'release':release.record(),'distance':distance.record(),
            'expression':'q=min(cap,max(sqrt(p*n*n/(p+n)),n*distance/(p+n)))',
            'cap_square':str(cap_square),
            'repeat_equation':'min(q,max(lower,target))=q'})
    character_images=[]
    for mask in range(16):
        row=[mask.bit_count(),0,0,0]
        for vertex in range(4):
            if mask&(1<<vertex):
                x,y=1-2*(vertex&1),1-2*(vertex>>1)
                for axis,value in enumerate((x,y,x*y)):row[axis+1]+=value
        character_images.append(row)
    result={'schema':'joint-source-mechanical-root-carrier/v1',
        'source_sha256':relation['source_sha256'],'source_relation':str(relation_path),
        'source_relation_sha256':sha(relation_path),'equations_sha256':sha(equation_path),
        'source_event_count':len(relation['events']),'source_occurrences':relation['occurrences_bound'],
        'joint_relation':relation,'root_certificate':certificate,
        'algebraic_positions':[[v.v.record() for v in point] for point in points],
        'algebraic_bank':bank,'four_state_character_images':character_images,
        'mechanical_root_prepared':True,'native_return_pending':True,
        'full_chain_lattice_fold_closed':False}
    output.write_text(json.dumps(result,separators=(',',':'))+'\n')
    print('root carrier events',len(relation['events']),'bank cells',len(bank),flush=True)


if __name__=='__main__':prepare(*map(Path,sys.argv[1:7]))
