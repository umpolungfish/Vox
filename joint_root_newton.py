"""Simultaneous source-bound force solve using differentiated Jacobians."""
import hashlib
import json
from pathlib import Path
import sys
from fractions import Fraction
import numpy as np
from scipy.optimize import root
from joint_geometry_return import numerical_operator
from whole_algebraic_readout import I,S


def refine_interval_center(equation_path,source_path,witness_path,output):
    equations=json.loads(equation_path.read_text());source=json.loads(source_path.read_text())
    if equations['current_carrier_sha256']!=hashlib.sha256(source_path.read_bytes()).hexdigest():
        raise ValueError('interval refinement carrier binding changed')
    record=json.loads(witness_path.read_text())
    if record['source_sha256']!=equations['source_sha256']:
        raise ValueError('interval refinement witness source differs')
    whole=numerical_operator(equations,source);whole.charts=record['charts']
    x=[Fraction(v) for v in record['extended_parameters']]
    for step in range(3):
        exact=whole.evaluate([I.value(v) for v in x],True,hessian=False)[0]
        numeric=whole.evaluate(list(map(float,x)),hessian=True)[0]
        C=np.linalg.inv(np.asarray(numeric.h,float))
        force=[Fraction(v.lo+v.hi,2*S) for v in exact.g]
        correction=[sum(Fraction(float(C[i,k]))*force[k] for k in range(whole.dimension)) for i in range(whole.dimension)]
        print('interval-center correction',step,float(max(map(abs,correction))),flush=True)
        def dyadic(value):
            scaled=value*(1<<100)
            return Fraction(scaled.numerator//scaled.denominator,1<<100)
        x=[dyadic(v-d) for v,d in zip(x,correction)]
        record.update(extended_parameters=[str(v) for v in x],parameters=list(map(float,x)),
            refinement='interval center force and exact carried bank cap squares',
            full_chain_lattice_fold_closed=False)
        output.write_text(json.dumps(record,indent=2)+'\n')


def solve(equation_path, source_path, witness_path, output):
    equations=json.loads(equation_path.read_text());source=json.loads(source_path.read_text())
    if equations['current_carrier_sha256']!=hashlib.sha256(source_path.read_bytes()).hexdigest():
        raise ValueError('joint root source carrier changed')
    witness=json.loads(witness_path.read_text())
    if witness['source_sha256']!=equations['source_sha256']:
        raise ValueError('joint root witness belongs to another source')
    whole=numerical_operator(equations,source);whole.charts=witness['charts']
    cached=None;cache_value=None;best=float('inf');calls=0
    def evaluate(x):
        nonlocal cached,cache_value,best,calls
        if cached is not None and np.array_equal(cached,x):return cache_value
        energy,_,_=whole.evaluate(x,hessian=True)
        cached=np.array(x,copy=True);cache_value=(np.asarray(energy.g,float),np.asarray(energy.h,float))
        residual=float(np.max(np.abs(energy.g)));calls+=1
        if residual<best:
            best=residual
            record={'parameters':list(map(float,x)),'charts':whole.charts,
                'energy':float(energy.v),'gradient_inf':residual,'jacobian_evaluations':calls,
                'source_sha256':equations['source_sha256'],
                'current_carrier_sha256':equations['current_carrier_sha256'],
                'status':'held simultaneous root witness','full_chain_lattice_fold_closed':False}
            output.write_text(json.dumps(record,indent=2)+'\n')
            print('joint Jacobian evaluation',calls,'best force residual',best,flush=True)
        return cache_value
    result=root(lambda x:evaluate(x)[0],witness['parameters'],jac=lambda x:evaluate(x)[1],
        options={'maxfev':25,'xtol':1e-10,'factor':0.1})
    record=json.loads(output.read_text());record['solver_message']=str(result.message)
    record['solver_success']=bool(result.success)
    output.write_text(json.dumps(record,indent=2)+'\n')


if __name__=='__main__':
    if sys.argv[1]=='--interval-center':refine_interval_center(*map(Path,sys.argv[2:6]))
    else:solve(*map(Path,sys.argv[1:5]))
