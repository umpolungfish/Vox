import re,sys
def enc(n):
    b=[];m=n
    while m: b.append(m&1); m>>=1
    if not b: b=[0]
    return '⊢'+''.join('≻⋈∈⊥∋' if x else '≻⋈∈⊤∋' for x in b)+'⊙⊡⊣'
def dec(w):
    cs=list(w); i=0; bits=[]
    while i+4<len(cs):
        if cs[i]=='≻' and cs[i+1]=='⋈' and cs[i+2]=='∈' and cs[i+3] in '⊥⊤' and cs[i+4]=='∋':
            bits.append(1 if cs[i+3]=='⊥' else 0); i+=5
        else: i+=1
    return sum(b<<k for k,b in enumerate(bits))
if __name__=='__main__':
    if sys.argv[1]=='enc': print(enc(int(sys.argv[2])))
    else: print(dec(sys.argv[2]))
