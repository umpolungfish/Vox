"""One interval inclusion for the complete joint mechanical return equations."""
import json
from fractions import Fraction
from pathlib import Path
import sys
import numpy as np
from joint_geometry_return import numerical_operator
from whole_algebraic_readout import I,Jet,S
from whole_interval_kernel import evaluate_whole


def positive_congruence(jacobian):
    n=len(jacobian)
    _,vectors=np.linalg.eigh(np.array([[x.midpoint() for x in row] for row in jacobian]))
    Q=[[I.value(float(vectors[i,j])) for j in range(n)] for i in range(n)]
    HQ=[[sum(jacobian[i][k]*Q[k][j] for k in range(n)) for j in range(n)] for i in range(n)]
    transformed=[[sum(Q[k][i]*HQ[k][j] for k in range(n)) for j in range(n)] for i in range(n)]
    gram=[[sum(Q[k][i]*Q[k][j] for k in range(n)) for j in range(n)] for i in range(n)]
    def margins(matrix):return [matrix[i][i].lo-sum(matrix[i][j].abs_upper() for j in range(n) if j!=i) for i in range(n)]
    root_margins=margins(transformed);basis_margins=margins(gram)
    return {'verified':min(root_margins)>0 and min(basis_margins)>0,
        'rational_basis':[[x.record() for x in row] for row in Q],
        'congruent_jacobian':[[x.record() for x in row] for row in transformed],
        'positive_margin_numerators':[str(x) for x in root_margins],
        'basis_rank_margin_numerators':[str(x) for x in basis_margins],
        'denominator_power_two':128}


def enclose(equation_path,source_path,witness_path,output):
    equations=json.loads(equation_path.read_text());source=json.loads(source_path.read_text())
    witness=json.loads(witness_path.read_text());whole=numerical_operator(equations,source)
    whole.charts=witness['charts'];n=whole.dimension
    center=[Fraction(x) for x in witness['extended_parameters']]
    midpoint=whole.evaluate([I.value(x) for x in center],True,hessian=False)[0]
    print('joint interval center complete',flush=True)
    numeric=whole.evaluate([float(x) for x in center])[0]
    C=[[I.value(float(x)) for x in row] for row in np.linalg.inv(np.asarray(numeric.h,float))]
    correction=[sum(C[i][k]*midpoint.g[k] for k in range(n)) for i in range(n)]
    needed=8*max(Fraction(x.abs_upper(),S) for x in correction)
    rational_radius=Fraction(1,1<<50)
    while rational_radius<needed:rational_radius*=2
    print('joint enclosure radius',float(rational_radius),flush=True)
    radius=I.value(rational_radius);box=[I.value(x)+radius*I(-S,S) for x in center]
    enclosure=evaluate_whole(whole,box,center,I,Jet)
    matrix=[[I.value(int(i==j))-sum(C[i][k]*enclosure.h[k][j] for k in range(n)) for j in range(n)] for i in range(n)]
    image=[I.value(center[i])-sum(C[i][k]*midpoint.g[k] for k in range(n))+
        sum(matrix[i][j]*(box[j]-I.value(center[j])) for j in range(n)) for i in range(n)]
    inside=all(b.lo<a.lo and a.hi<b.hi for a,b in zip(image,box))
    diagonal=[];L=[[I.value(0) for _ in range(n)] for _ in range(n)]
    for i in range(n):
        d=enclosure.h[i][i]-sum(L[i][k]*L[i][k]*diagonal[k] for k in range(i))
        if d.lo<=0:break
        diagonal.append(d);L[i][i]=I.value(1)
        for j in range(i+1,n):L[j][i]=(enclosure.h[j][i]-sum(L[j][k]*L[i][k]*diagonal[k] for k in range(i)))/d
    result={'source_sha256':equations['source_sha256'],'current_carrier_sha256':equations['current_carrier_sha256'],
        'strict_inclusion':inside,'positive_mechanical_pivots':len(diagonal)==n,
        'box':[x.record() for x in box],'image':[x.record() for x in image],
        'jacobian':[[x.record() for x in row] for row in enclosure.h],
        'positive_pivots':[x.record() for x in diagonal],
        'complete_mechanical_root_enclosed':inside and len(diagonal)==n,
        'full_chain_lattice_fold_closed':False}
    result['positive_mechanical_congruence']=positive_congruence(enclosure.h)
    result['complete_mechanical_root_enclosed']=inside and (len(diagonal)==n or result['positive_mechanical_congruence']['verified'])
    output.write_text(json.dumps(result,indent=2)+'\n');print('joint root inclusion',inside,'positive pivots',len(diagonal),flush=True)

if __name__=='__main__':enclose(*map(Path,sys.argv[1:5]))
