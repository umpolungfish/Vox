import sys
def enc(n):
    b=[];m=n
    while m: b.append(m&1); m>>=1
    if not b: b=[0]
    return '⊢'+''.join('≻⋈∈⊥∋' if x else '≻⋈∈⊤∋' for x in b)+'⊙⊡⊣'
def dec(w):
    from numeral_codec import decode_word
    return decode_word(w)
if __name__=='__main__':
    if sys.argv[1]=='enc': print(enc(int(sys.argv[2])))
    else: print(dec(sys.argv[2]))
