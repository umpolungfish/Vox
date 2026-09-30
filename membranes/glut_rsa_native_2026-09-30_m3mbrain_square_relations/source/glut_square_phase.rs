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
struct Relation { cells: Vec<usize>, x: Tape, y: Tape }
pub(super) struct SquarePhase {
    n: Tape,
    start: Tape,
    columns: Vec<Column>,
    basis: BTreeMap<usize,Relation>,
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
        Self {n:n.to_vec(),start,columns:Vec::new(),basis:BTreeMap::new(),
            partial:BTreeMap::new(),bound:n.len()*n.len(),span:n.len()*n.len(),
            examined:1,ready:None}
    }
    pub fn source(&self) -> &[char] {&self.n}
    pub fn cells(&self) -> usize {
        self.n.len()+self.start.len()+self.columns.iter().map(|c|c.prime.len()+c.roots.0.len()+c.roots.1.len()).sum::<usize>()
            +self.basis.values().chain(self.partial.values()).map(|r|r.cells.len()+r.x.len()+r.y.len()).sum::<usize>()
    }
    fn grow(&mut self) {
        let mut composite=vec![false;self.bound+1];
        for position in 2..=self.bound {
            if composite[position] {continue;}
            let mut multiple=position.checked_mul(position).unwrap_or(self.bound+1);
            while multiple<=self.bound {composite[multiple]=true;multiple+=position;}
            if position<=self.examined {continue;}
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
    fn join(&self, mut left: Relation, right: &Relation) -> Relation {
        left.x=mm(&left.x,&right.x,&self.n);
        left.y=mm(&left.y,&right.y,&self.n);
        let mut cells=Vec::new(); let (mut a,mut b)=(0,0);
        while a<left.cells.len() || b<right.cells.len() {
            match (left.cells.get(a),right.cells.get(b)) {
                (Some(&i),Some(&j)) if i==j => {
                    left.y=mm(&left.y,&self.columns[i].prime,&self.n); a+=1;b+=1;
                }
                (Some(&i),Some(&j)) if i<j => {cells.push(i);a+=1;}
                (Some(_),Some(&j)) => {cells.push(j);b+=1;}
                (Some(&i),None) => {cells.push(i);a+=1;}
                (None,Some(&j)) => {cells.push(j);b+=1;}
                (None,None) => break,
            }
        }
        left.cells=cells; left
    }
    fn retain(&mut self, mut relation: Relation) -> Option<Tape> {
        while let Some(&pivot)=relation.cells.last() {
            if let Some(other)=self.basis.get(&pivot) {relation=self.join(relation,other);}
            else {self.basis.insert(pivot,relation);return None;}
        }
        // A canceled phase row must carry its exact modular square witness.
        assert_eq!(mm(&relation.x,&relation.x,&self.n),mm(&relation.y,&relation.y,&self.n));
        let difference=if mf::cmp(&relation.x,&relation.y)==core::cmp::Ordering::Less {
            mf::sub(&relation.y,&relation.x)
        } else {mf::sub(&relation.x,&relation.y)};
        for value in [difference,mf::add(&relation.x,&relation.y)] {
            let divisor=mf::gcd(value,self.n.clone());
            if mf::cmp(&divisor,&[ONE])==core::cmp::Ordering::Greater
                && mf::cmp(&divisor,&self.n)==core::cmp::Ordering::Less {return Some(divisor);}
        }
        None
    }
    pub fn advance(&mut self) -> Option<(Tape,Tape)> {
        if self.columns.is_empty() || self.examined<self.bound {self.grow();}
        if let Some(p)=self.ready.take() {return Some((p.clone(),mf::divmod(&self.n,&p).0));}
        let mut hits=vec![Vec::new();self.span];
        let mut weights=vec![0;self.span];
        for (column,field) in self.columns.iter().enumerate() {
            let rem=mf::modulo(&self.start,&field.prime);
            let displacement=|r: &[char]| {
                if mf::cmp(r,&rem)==core::cmp::Ordering::Less {mf::sub(&mf::add(r,&field.prime),&rem)}
                else {mf::sub(r,&rem)}
            };
            let first=offset(&displacement(&field.roots.0));
            let second=offset(&displacement(&field.roots.1));
            for begin in [Some(first),(second!=first).then_some(second)].into_iter().flatten() {
                let mut index=begin;
                while index<self.span {
                    hits[index].push(column);weights[index]+=field.prime.len()-1;index+=field.step;
                }
            }
        }
        let slack=2*address(self.bound).len()+4;
        let mut useful=0;
        for index in 0..self.span {
            let expected=self.n.len().div_ceil(2)+address(index+1).len();
            if weights[index]+slack<expected {continue;}
            let x=mf::add(&self.start,&address(index));
            let mut residue=mf::sub(&mf::mul(&x,&x),&self.n);
            if mf::zero(&residue) {return Some((x.clone(),x));}
            let mut cells=Vec::new(); let mut y=vec![ONE];
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
            let mut relation=Relation {cells,x,y};
            if residue!=vec![ONE] {
                if let Some(previous)=self.partial.remove(&residue) {
                    relation=self.join(relation,&previous);
                    relation.y=mm(&relation.y,&residue,&self.n);
                } else {self.partial.insert(residue,relation);continue;}
            }
            useful+=1;
            if let Some(p)=self.retain(relation) {return Some((p.clone(),mf::divmod(&self.n,&p).0));}
        }
        self.start=mf::add(&self.start,&address(self.span));
        // Relation support grows when a block contributes no reduced rows.
        // Existing columns, partial relations and ancestry remain resident.
        if useful==0 {self.bound+=self.n.len()*self.n.len();self.span+=self.n.len()*self.n.len();}
        None
    }
}
