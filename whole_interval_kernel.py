"""Compiled outward interval AD for the complete shared-port response fold."""
import ctypes
from pathlib import Path
import subprocess
import tempfile

import numpy as np

N=17
JET=1+N+N*N
WORDS=6
MASK=(1<<64)-1
SIGN=1<<191
MOD=1<<192

def _library():
    source=Path(__file__).with_name('whole_interval_radial.cpp')
    target=Path(tempfile.gettempdir())/'vox_whole_interval_radial.so'
    if not target.exists() or target.stat().st_mtime<source.stat().st_mtime:
        subprocess.run(['g++','-O2','-fno-inline-functions','-fno-inline-small-functions',
            '-std=c++17','-shared','-fPIC',str(source),'-o',str(target)],
            check=True,timeout=55)
    library=ctypes.CDLL(str(target))
    fn=library.whole_interval_energy
    ptr=ctypes.POINTER(ctypes.c_uint64)
    fn.argtypes=[ptr,ptr,ctypes.POINTER(ctypes.c_uint8),ctypes.POINTER(ctypes.c_uint32),
        ptr,ctypes.c_size_t,ptr]
    fn.restype=None
    return fn

def _store_endpoint(out,index,value):
    value=int(value)
    if not -(1<<191)<=value<(1<<191):raise OverflowError('interval endpoint exceeds 192-bit kernel lane')
    value &= MOD-1
    base=index*3
    out[base]=value&MASK;out[base+1]=(value>>64)&MASK;out[base+2]=(value>>128)&MASK

def _store_interval(out,index,value):
    _store_endpoint(out,index*2,value.lo)
    _store_endpoint(out,index*2+1,value.hi)

def _read_endpoint(words,index):
    base=index*3
    value=int(words[base])|(int(words[base+1])<<64)|(int(words[base+2])<<128)
    return value-MOD if value>=SIGN else value

def evaluate_whole(whole,box,center,interval_type,jet_type):
    if getattr(whole,'active_orbital_force_channels',[]):
        from fractions import Fraction
        from types import SimpleNamespace
        _,_,sites=whole.geometry(box,True,hessian=False)
        responses=list(whole.responses)
        for index,(i,a,j,b,count,p,n) in enumerate(whole.active_orbital_force_channels):
            I=interval_type;c=I.value(count);span=p+n
            if not p or not n:
                responses.append((i,1<<a,j,1<<b,c*span,c*span*n,c*span*n*n));continue
            delta=[sites[i][a][axis]-sites[j][b][axis] for axis in range(3)]
            distance=sum(x*x for x in delta).sqrt().v
            lower=I.value(Fraction(p*n*n,span)).sqrt()
            cap=I.value(Fraction(float(whole.channel_preference_caps[index])))
            if hasattr(whole,'channel_preference_cap_squares'):
                cap=I.value(whole.channel_preference_cap_squares[index]).sqrt()
            collapsed=(hasattr(whole,'channel_preference_cap_squares') and
                whole.channel_preference_cap_squares[index]==Fraction(p*n*n,span))
            if collapsed:q=lower
            elif distance.hi<=(span*lower/n).lo:q=lower
            elif distance.lo>=(span*cap/n).hi:q=cap
            elif distance.lo>=(span*lower/n).hi and distance.hi<=(span*cap/n).lo:
                responses.append((i,1<<a,j,1<<b,c*p,I.value(0),-c*p*p*n));continue
            else:
                # The potential is C1 at each bank boundary. Hull the
                # applicable branch Jacobians through their radial forms.
                options=[]
                candidates=[]
                if distance.lo<=(span*lower/n).hi:candidates.append(lower)
                if distance.hi>=(span*cap/n).lo:candidates.append(cap)
                for q0 in candidates:
                    options.append((c*span,c*span*q0,c*span*q0*q0+c*p*(span*q0*q0/n-p*n)))
                if distance.hi>=(span*lower/n).lo and distance.lo<=(span*cap/n).hi:
                    options.append((c*p,I.value(0),-c*p*p*n))
                for option_index,coefficients in enumerate(options):
                    residue=i|(len(options)<<16) if option_index==0 else i
                    responses.append((residue,1<<a,j,1<<b,*coefficients))
                continue
            responses.append((i,1<<a,j,1<<b,c*span,c*span*q,
                c*span*q*q+c*p*(span*q*q/n-p*n)))
        whole=SimpleNamespace(charts=whole.charts,responses=responses)
    box_words=np.empty(N*WORDS,dtype=np.uint64)
    center_words=np.empty(N*WORDS,dtype=np.uint64)
    for i in range(N):
        _store_interval(box_words,i,box[i])
        _store_interval(center_words,i,interval_type.value(center[i]))
    charts=np.asarray(whole.charts,dtype=np.uint8)
    count=len(whole.responses)
    keys=np.empty(count*4,dtype=np.uint32)
    coeff=np.empty(count*3*WORDS,dtype=np.uint64)
    for i,row in enumerate(whole.responses):
        keys[4*i:4*i+4]=row[:4]
        for j,value in enumerate(row[4:]):
            _store_interval(coeff,i*3+j,interval_type.value(value))
    output=np.zeros(JET*WORDS,dtype=np.uint64)
    _library()(box_words.ctypes.data_as(ctypes.POINTER(ctypes.c_uint64)),
        center_words.ctypes.data_as(ctypes.POINTER(ctypes.c_uint64)),
        charts.ctypes.data_as(ctypes.POINTER(ctypes.c_uint8)),
        keys.ctypes.data_as(ctypes.POINTER(ctypes.c_uint32)),
        coeff.ctypes.data_as(ctypes.POINTER(ctypes.c_uint64)),count,
        output.ctypes.data_as(ctypes.POINTER(ctypes.c_uint64)))
    values=[]
    for i in range(JET):values.append(interval_type(
        _read_endpoint(output,i*2),_read_endpoint(output,i*2+1)))
    gradient=np.asarray(values[1:1+N],dtype=object)
    hessian=np.asarray(values[1+N:],dtype=object).reshape((N,N))
    return jet_type(values[0],gradient,hessian)
