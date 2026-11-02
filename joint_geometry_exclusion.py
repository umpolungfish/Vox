"""Constrained joint geometry witness using native molecular exclusions."""
import json
from pathlib import Path
import sys
import numpy as np
from scipy.optimize import minimize
from joint_geometry_return import numerical_operator


def solve(equation_path,source_path,witness_path,geometry_path,output):
    load=lambda p:json.loads(p.read_text())
    equations,source,witness,geometry=map(load,(equation_path,source_path,witness_path,geometry_path))
    whole=numerical_operator(equations,source);whole.charts=witness['charts']
    whole.angular_geometry=True
    scale=geometry['coordinate_readout_A_per_native_unit']
    exclusions=geometry['source_ca_exclusion_relations_A']
    cached=None;value=None
    def constraints(x):
        nonlocal cached,value
        if cached is not None and np.array_equal(x,cached):return value
        _,points,_=whole.geometry(x,hessian=False)
        residuals=[];derivatives=[]
        for i,j,minimum in exclusions:
            delta=[a-b for a,b in zip(points[i],points[j])]
            square=sum(d*d for d in delta)
            residuals.append(square.v-(minimum/scale)**2)
            derivatives.append(square.g)
        cached=np.array(x,copy=True);value=np.array(residuals),np.asarray(derivatives)
        return value
    q=np.asarray(witness['parameters']);c=np.where(witness['charts'],(q*q-1)/(1+q*q),(1-q*q)/(1+q*q))
    initial=np.asarray(witness.get('angles',np.arctan2(2*q/(1+q*q),c)))
    initial_g=constraints(initial)[0]
    objective_scale=max(1.0,float(np.max(np.abs(whole.float_energy_gradient(initial,angular=True)[1]))))
    margin=64*np.sqrt(np.finfo(float).eps)
    print('source exclusion violations before',int(np.count_nonzero(initial_g<0)),flush=True)
    evaluations=0;productive=None;off_pathway=[]
    def save(x):
        nonlocal evaluations,productive
        evaluations+=1;energy,force=whole.float_energy_gradient(x,angular=True)
        g=constraints(x)[0]
        result={'parameters':(np.sin(x)/(1+np.abs(np.cos(x)))).tolist(),
            'charts':(np.cos(x)<0).tolist(),'angles':list(map(float,x)),'energy':energy,
            'geometry_constraints':exclusions,'calibration_A_per_native_unit':scale,
            'minimum_constraint_residual':float(g.min()),'violations':int(np.count_nonzero(g < -1e-8)),
            'source_sha256':equations['source_sha256'],'numerical_iterations':evaluations,
            'numerical_objective_scale':objective_scale,
            'numerical_interior_margin':margin,
            'force_inf':float(np.max(np.abs(force))),
            'status':'held constrained joint geometry witness','full_chain_lattice_fold_closed':False}
        if g.min()>=0:
            if productive is None or energy<productive['energy']:productive=dict(result)
        else:off_pathway.append({'angles':list(map(float,x)),'energy':energy,
            'conflicts':[[int(i),float(g[i])] for i in np.flatnonzero(g<0)]})
        result['productive_arm']=productive
        result['off_pathway_arm']=off_pathway
        result['source_operator_binding']='peptide_folding_operator_source_binding.json'
        result['reconstitution_status']='held joint ensemble; solvent boundary and full operator execution pending'
        output.write_text(json.dumps(result,indent=2)+'\n')
        if evaluations%10==0:print('coupled geometry witness',evaluations,'violations',result['violations'],flush=True)
    def objective(x):
        energy,gradient=whole.float_energy_gradient(x,angular=True)
        return energy/objective_scale,gradient/objective_scale
    save(initial)
    result=minimize(objective,initial,jac=True,method='SLSQP',
        constraints={'type':'ineq','fun':lambda x:constraints(x)[0]-margin,'jac':lambda x:constraints(x)[1]},
        callback=save,options={'maxiter':200,'ftol':1e-10})
    save(result.x)
    record=load(output);record['solver_message']=str(result.message)
    record['solver_success']=bool(result.success);output.write_text(json.dumps(record,indent=2)+'\n')
    print('source exclusion violations after',record['violations'],result.message,flush=True)


if __name__=='__main__':solve(*map(Path,sys.argv[1:6]))
