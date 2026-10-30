import re,sys
def enc(n):
    b=[];m=n
    while m: b.append(m&1); m>>=1
    if not b: b=[0]
    return '⊢'+''.join('≻⋈∈⊥∋' if x else '≻⋈∈⊤∋' for x in b)+'⊙⊡⊣'
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from numeral_codec import decode_word

def dec(w):
    return decode_word(w)
if __name__=='__main__':
    if sys.argv[1]=='enc': print(enc(int(sys.argv[2])))
    else: print(dec(sys.argv[2]))
