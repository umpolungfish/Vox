//! Square-congruence phase relations over resident IMASM numeral tapes.
//! Sparse DQI row elimination cancels odd prime columns. Shared columns fold
//! into the square-root accumulator, retaining an exact modular square witness.
use super::{mf, trim, ONE, ZERO};
use alloc::{collections::BTreeMap, vec, vec::Vec};
type Tape = Vec<char>;
fn address(mut value: usize) -> Tape {
    let mut tape=Vec::new();
    while value!=0 { tape.push(if value&1==1 {ONE} else {ZERO}); value>>=1; }
    if tape.is_empty() { tape.push(ZERO); }
    tape
}
fn offset(tape: &[char]) -> usize {
    let mut value=0;
    for &mark in tape.iter().rev() { value=value*2+usize::from(mark==ONE); }
    value
}
fn mm(a: &[char], b: &[char], n: &[char]) -> Tape { mf::modulo(&mf::mul(a,b),n) }
fn power(a: &[char], exponent: &[char], n: &[char]) -> Tape {
    let mut value=vec![ONE]; let mut square=mf::modulo(a,n);
    for &mark in exponent {
        if mark==ONE { value=mm(&value,&square,n); }
        square=mm(&square,&square,n);
    }
    value
}
fn root(n: &[char], p: &[char]) -> Option<Tape> {
    let one=vec![ONE]; let two=vec![ZERO,ONE];
    if p==two { return Some(mf::modulo(n,p)); }
    let less=mf::sub(p,&one);
    let half=trim(less[1..].to_vec());
    if power(n,&half,p)!=one { return None; }
    let count=less.iter().position(|&mark|mark==ONE).unwrap();
    let odd=trim(less[count..].to_vec());
    let mut nonresidue=two.clone();
    while power(&nonresidue,&half,p)==one { nonresidue=mf::add(&nonresidue,&one); }
    let mut c=power(&nonresidue,&odd,p);
    let mut t=power(n,&odd,p);
    let mut r=power(n,&trim(mf::add(&odd,&one)[1..].to_vec()),p);
    let mut extent=count;
    while t!=one {
        let mut at=0; let mut test=t.clone();
        while test!=one {
            test=mm(&test,&test,p); at+=1;
            if at==extent { return None; }
        }
        let mut b=c.clone();
        for _ in 0..extent-at-1 { b=mm(&b,&b,p); }
        c=mm(&b,&b,p); t=mm(&t,&c,p); r=mm(&r,&b,p); extent=at;
    }
    Some(r)
}
struct Column { prime: Tape, step: usize, roots: (Tape,Tape) }
struct Relation { cells: Vec<usize>, history: Vec<usize> }
struct Witness { cells: Vec<usize>, x: Tape, y: Tape, residual: Tape }
fn xor_addresses(left: &[usize], right: &[usize]) -> Vec<usize> {
    let mut out=Vec::new(); let (mut a,mut b)=(0,0);
    while a<left.len() || b<right.len() {
        match (left.get(a),right.get(b)) {
            (Some(&i),Some(&j)) if i==j => {a+=1;b+=1;}
            (Some(&i),Some(&j)) if i<j => {out.push(i);a+=1;}
            (Some(_),Some(&j)) => {out.push(j);b+=1;}
            (Some(&i),None) => {out.push(i);a+=1;}
            (None,Some(&j)) => {out.push(j);b+=1;}
            (None,None) => break,
        }
    }
    out
}
pub(super) struct SquarePhase {
    n: Tape,
    start: Tape,
    coefficient_seed: Tape,
    columns: Vec<Column>,
    basis: BTreeMap<usize,Relation>,
    witnesses: Vec<Witness>,
    partial: BTreeMap<Tape,Relation>,
    bound: usize,
    span: usize,
    examined: usize,
    ready: Option<Tape>,
}
impl SquarePhase {
    pub fn new(n: &[char], source_root: &[char]) -> Self {
        let start=if mf::mul(source_root,source_root)==n { source_root.to_vec() }
            else { mf::add(source_root,&[ONE]) };
        let span=n.len()*n.len();
        let target=mf::divmod(&mf::isqrt(&mf::add(n,n)),&address(span)).0;
        let mut coefficient_seed=mf::isqrt(&target);
        if mf::cmp(&coefficient_seed,&[ONE,ONE])==core::cmp::Ordering::Less {coefficient_seed=vec![ONE,ONE];}
        if coefficient_seed[0]==ZERO {coefficient_seed=mf::add(&coefficient_seed,&[ONE]);}
        Self {n:n.to_vec(),start,coefficient_seed,
            columns:vec![Column {prime:mf::sub(n,&[ONE]),step:0,roots:(vec![ZERO],vec![ZERO])}],basis:BTreeMap::new(),witnesses:Vec::new(),
            partial:BTreeMap::new(),bound:n.len()*n.len(),span,
            examined:1,ready:None}
    }
    pub fn source(&self) -> &[char] {&self.n}
    pub fn cells(&self) -> usize {
        self.n.len()+self.start.len()+self.columns.iter().map(|c|c.prime.len()+c.roots.0.len()+c.roots.1.len()).sum::<usize>()
            +self.basis.values().chain(self.partial.values()).map(|r|r.cells.len()+r.history.len()).sum::<usize>()
            +self.witnesses.iter().map(|r|r.cells.len()+r.x.len()+r.y.len()+r.residual.len()).sum::<usize>()
    }
    fn grow(&mut self, observe: &mut impl FnMut(char,usize,usize,usize)) {
        observe('∈',self.columns.len(),self.basis.len(),self.partial.len());
        let mut composite=vec![false;self.bound+1];
        for position in 2..=self.bound {
            if composite[position] {continue;}
            let mut multiple=position.checked_mul(position).unwrap_or(self.bound+1);
            while multiple<=self.bound {composite[multiple]=true;multiple+=position;}
            if position<=self.examined {continue;}
            if position.is_power_of_two() {observe('∈',self.columns.len(),self.basis.len(),self.partial.len());}
            let prime=address(position);
            let residue=mf::modulo(&self.n,&prime);
            if mf::zero(&residue) {self.ready=Some(prime);return;}
            if let Some(r)=root(&residue,&prime) {
                let other=if mf::zero(&r) {r.clone()} else {mf::sub(&prime,&r)};
                self.columns.push(Column {prime,step:position,roots:(r,other)});
            }
        }
        self.examined=self.bound;
    }
    fn join(&self, left: Relation, right: &Relation) -> Relation {
        Relation {cells:xor_addresses(&left.cells,&right.cells),
            history:xor_addresses(&left.history,&right.history)}
    }
    fn witness(&self, history: &[usize]) -> (Tape,Tape) {
        use alloc::collections::BTreeSet;
        let mut x=vec![ONE];let mut y=vec![ONE];
        let mut odd=BTreeSet::new();let mut residuals=BTreeSet::new();
        for &index in history {
            let leaf=&self.witnesses[index];
            x=mm(&x,&leaf.x,&self.n);y=mm(&y,&leaf.y,&self.n);
            for &column in &leaf.cells {
                if !odd.insert(column) {
                    odd.remove(&column); y=mm(&y,&self.columns[column].prime,&self.n);
                }
            }
            if leaf.residual!=vec![ONE] && !residuals.insert(leaf.residual.clone()) {
                residuals.remove(&leaf.residual);y=mm(&y,&leaf.residual,&self.n);
            }
        }
        assert!(odd.is_empty() && residuals.is_empty(),"phase ancestry must cancel every column");
        (x,y)
    }
    fn retain(&mut self, mut relation: Relation) -> Option<Tape> {
        while let Some(&pivot)=relation.cells.last() {
            if let Some(other)=self.basis.get(&pivot) {relation=self.join(relation,other);}
            else {self.basis.insert(pivot,relation);return None;}
        }
        // Materialize values only after their symbolic phase ancestry closes.
        let (x,y)=self.witness(&relation.history);
        assert_eq!(mm(&x,&x,&self.n),mm(&y,&y,&self.n));
        let difference=if mf::cmp(&x,&y)==core::cmp::Ordering::Less {
            mf::sub(&y,&x)
        } else {mf::sub(&x,&y)};
        for value in [difference,mf::add(&x,&y)] {
            let divisor=mf::gcd(value,self.n.clone());
            if mf::cmp(&divisor,&[ONE])==core::cmp::Ordering::Greater
                && mf::cmp(&divisor,&self.n)==core::cmp::Ordering::Less {return Some(divisor);}
        }
        None
    }
    fn polynomial(&mut self, observe: &mut impl FnMut(char,usize,usize,usize)) -> (Tape,Tape,Tape) {
        loop {
            let q=self.coefficient_seed.clone();
            self.coefficient_seed=mf::add(&self.coefficient_seed,&[ZERO,ONE]);
            observe('≻',self.columns.len(),self.basis.len(),self.partial.len());
            if !mf::miller_rabin(&q) {continue;}
            let residue=mf::modulo(&self.n,&q);
            observe('⊞',self.columns.len(),self.basis.len(),self.partial.len());
            let Some(r)=root(&residue,&q) else {continue;};
            observe('⋈',self.columns.len(),self.basis.len(),self.partial.len());
            let a=mf::mul(&q,&q);
            let (difference,remainder)=mf::divmod(&mf::sub(&self.n,&mf::mul(&r,&r)),&q);
            assert!(mf::zero(&remainder));
            let inverse=mf::mod_inv(&mf::add(&r,&r),&q).expect("lifted coefficient root is invertible");
            let b=mf::add(&r,&mf::mul(&q,&mm(&difference,&inverse,&q)));
            assert_eq!(mf::modulo(&mf::mul(&b,&b),&a),mf::modulo(&self.n,&a));
            return (a,b,q);
        }
    }
    pub fn advance(&mut self, mut observe: impl FnMut(char,usize,usize,usize)) -> Option<(Tape,Tape)> {
        if self.examined<self.bound {self.grow(&mut observe);}
        if let Some(p)=self.ready.take() {return Some((p.clone(),mf::divmod(&self.n,&p).0));}
        let (a,b,coefficient_root)=self.polynomial(&mut observe);
        let half=self.span/2;
        let half_tape=address(half);
        let mut hits=vec![Vec::new();self.span];
        let mut weights=vec![0;self.span];
        for (column,field) in self.columns.iter().enumerate() {
            if column.is_power_of_two() {observe('∋',self.columns.len(),self.basis.len(),self.partial.len());}
            if field.step==0 {continue;}
            let reduced_a=mf::modulo(&a,&field.prime);
            let reduced_b=mf::modulo(&b,&field.prime);
            let modulus=&field.prime;
            let displacement=|r: &[char]| {
                if mf::cmp(r,&reduced_b)==core::cmp::Ordering::Less {mf::sub(&mf::add(r,modulus),&reduced_b)}
                else {mf::sub(r,&reduced_b)}
            };
            let (first,second)=if mf::zero(&reduced_a) {
                let square=mf::mul(&b,&b);
                let c=if mf::cmp(&square,&self.n)==core::cmp::Ordering::Less {
                    mf::divmod(&mf::sub(&self.n,&square),&a).0
                } else {
                    let value=mf::modulo(&mf::divmod(&mf::sub(&square,&self.n),&a).0,modulus);
                    if mf::zero(&value) {value} else {mf::sub(modulus,&value)}
                };
                let inverse=mf::mod_inv(&mf::add(&reduced_b,&reduced_b),modulus).expect("linear field root is invertible");
                let position=offset(&mf::modulo(&mf::add(&mm(&c,&inverse,modulus),&half_tape),modulus));
                (position,position)
            } else {
                let inverse=if field.step==2 {vec![ONE]}
                    else {mf::mod_inv(&reduced_a,modulus).expect("quadratic field coefficient is invertible")};
                let position=|r: &[char]| offset(&mf::modulo(&mf::add(&mm(&displacement(r),&inverse,modulus),&half_tape),modulus));
                (position(&field.roots.0),position(&field.roots.1))
            };
            for begin in [Some(first),(second!=first).then_some(second)].into_iter().flatten() {
                let mut index=begin;
                while index<self.span {
                    hits[index].push(column);weights[index]+=field.prime.len()-1;index+=field.step;
                }
            }
        }
        let slack=2*address(self.bound).len()+4;
        let mut useful=0;
        let partial_before=self.partial.len();
        observe('⊥',self.columns.len(),self.basis.len(),self.partial.len());
        for index in 0..self.span {
            if index.is_power_of_two() {observe('⊥',self.columns.len(),self.basis.len(),self.partial.len());}
            let expected=a.len()+2*half_tape.len();
            if weights[index]+slack<expected {continue;}
            let term=mf::mul(&a,&address(index.abs_diff(half)));
            let x=if index>=half {mf::add(&term,&b)}
                else if mf::cmp(&term,&b)==core::cmp::Ordering::Less {mf::sub(&b,&term)}
                else {mf::sub(&term,&b)};
            let square=mf::mul(&x,&x);
            let negative=mf::cmp(&square,&self.n)==core::cmp::Ordering::Less;
            let difference=if negative {mf::sub(&self.n,&square)} else {mf::sub(&square,&self.n)};
            let (mut residue,remainder)=mf::divmod(&difference,&a);
            assert!(mf::zero(&remainder));
            if mf::zero(&residue) {return Some((x.clone(),x));}
            let mut cells=if negative {vec![0]} else {Vec::new()};
            let mut y=coefficient_root.clone();
            for &column in &hits[index] {
                let field=&self.columns[column]; let mut odd=false;
                loop {
                    let (quotient,remainder)=mf::divmod(&residue,&field.prime);
                    if !mf::zero(&remainder) {break;}
                    residue=quotient; odd=!odd;
                    if !odd {y=mm(&y,&field.prime,&self.n);}
                }
                if odd {cells.push(column);}
            }
            let leaf=self.witnesses.len();
            self.witnesses.push(Witness {cells:cells.clone(),x,y,residual:residue.clone()});
            let mut relation=Relation {cells,history:vec![leaf]};
            if residue!=vec![ONE] {
                if let Some(previous)=self.partial.remove(&residue) {
                    relation=self.join(relation,&previous);
                } else {self.partial.insert(residue,relation);continue;}
            }
            useful+=1;
            observe('⊡',self.columns.len(),self.basis.len(),self.partial.len());
            let found=self.retain(relation);
            observe('⊥',self.columns.len(),self.basis.len(),self.partial.len());
            if let Some(p)=found {return Some((p.clone(),mf::divmod(&self.n,&p).0));}
        }
        self.start=mf::add(&self.start,&address(self.span));
        // Relation support grows when a block contributes no reduced rows.
        // Existing columns, partial relations and ancestry remain resident.
        if useful==0 && self.partial.len()==partial_before {self.bound+=self.bound;}
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn modular_roots_match_the_resident_64_bit_source() {
        let n=mf::parse_numeral(include_str!("../membranes/glut_rsa_native_2026-09-30_transport_identities/rsa_64_0/input.imasm").trim()).unwrap();
        assert_eq!(n.len(),64);
        let mut checked=0;
        for position in [257,263,269,271,277,281,283,293] {
            let p=address(position); let residue=mf::modulo(&n,&p);
            if let Some(r)=root(&residue,&p) {
                assert_eq!(mm(&r,&r,&p),residue);checked+=1;
            }
        }
        assert!(checked>1);
    }
}
