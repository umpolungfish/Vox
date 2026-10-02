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
struct Column { prime: Tape, step: usize, roots: (Tape,Tape), root_words: (u64,u64) }
struct PolynomialFamily {
    a: Tape, b: Tape, root: Tape, terms: Vec<Tape>, signs: Vec<bool>,
    inverses: Vec<Option<u64>>,
}
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
    composite: Vec<bool>,
    ready: Option<Tape>,
    hit_heads: Vec<usize>,
    hit_links: Vec<(usize,usize)>,
    weights: Vec<usize>,
    retained_cells: usize,
    family: Option<PolynomialFamily>,
    family_pool: Vec<usize>,
    family_choice: Vec<usize>,
    family_columns: usize,
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
        let retained_cells=mf::sub(n,&[ONE]).len()+2;
        Self {n:n.to_vec(),start,coefficient_seed,
            columns:vec![Column {prime:mf::sub(n,&[ONE]),step:0,roots:(vec![ZERO],vec![ZERO]),root_words:(0,0)}],basis:BTreeMap::new(),witnesses:Vec::new(),
            partial:BTreeMap::new(),bound:n.len()*n.len(),span,
            examined:1,composite:Vec::new(),ready:None,
            hit_heads:Vec::new(),hit_links:Vec::new(),weights:Vec::new(),retained_cells,
            family:None,family_pool:Vec::new(),family_choice:Vec::new(),family_columns:0}
    }
    pub fn source(&self) -> &[char] {&self.n}
    pub fn cells(&self) -> usize {
        // Account for the field and its retained ancestry without rescanning
        // every witness after each activation. Scheduling uses work_extent.
        self.bound+1+self.n.len()+self.start.len()+self.retained_cells
            +self.hit_heads.len()+2*self.hit_links.len()+self.weights.len()
            +self.family_pool.len()+self.family_choice.len()
            +self.family.as_ref().map_or(0,|f|f.a.len()+f.b.len()+f.root.len()
                +f.terms.iter().map(Vec::len).sum::<usize>()+f.signs.len()+f.inverses.len())
    }
    /// Active work follows the source-width preparation frame or one sieve
    /// window. Retained witnesses are not traversed by every activation.
    pub fn work_extent(&self)->usize {
        if self.examined<self.bound {
            self.n.len()*address(self.bound).len()
        } else {
            self.span+self.columns.len()+self.n.len()
        }
    }
    fn expand_field(&mut self) {
        self.bound+=self.bound;
        self.span=self.bound;
        let target=mf::divmod(&mf::isqrt(&mf::add(&self.n,&self.n)),&address(self.span)).0;
        self.coefficient_seed=mf::isqrt(&target);
        if mf::cmp(&self.coefficient_seed,&[ONE,ONE])==core::cmp::Ordering::Less {
            self.coefficient_seed=vec![ONE,ONE];
        }
        self.family=None;
        self.family_columns=0;
    }
    fn grow(&mut self, observe: &mut impl FnMut(char,usize,usize,usize)) {
        observe('∈',self.columns.len(),self.basis.len(),self.partial.len());
        if self.composite.len()!=self.bound+1 {
            self.composite=vec![false;self.bound+1];
            for position in 2..=self.bound {
                if self.composite[position] {continue;}
                let Some(mut multiple)=position.checked_mul(position) else {break;};
                if multiple>self.bound {break;}
                while multiple<=self.bound {self.composite[multiple]=true;multiple+=position;}
            }
        }
        // A source-width frame is one scheduling turn, not a terminal field
        // limit. Resume from the retained address on the next call so the
        // other resident lanes advance during wide field preparation.
        let end=self.bound.min(self.examined.saturating_add(self.n.len()));
        for position in self.examined+1..=end {
            if self.composite[position] {continue;}
            if position.is_power_of_two() {observe('∈',self.columns.len(),self.basis.len(),self.partial.len());}
            let prime=address(position);
            let residue=mf::modulo(&self.n,&prime);
            if mf::zero(&residue) {self.ready=Some(prime);return;}
            if let Some(r)=root(&residue,&prime) {
                let other=if mf::zero(&r) {r.clone()} else {mf::sub(&prime,&r)};
                self.retained_cells+=prime.len()+r.len()+other.len();
                let root_words=(offset(&r) as u64,offset(&other) as u64);
                self.columns.push(Column {prime,step:position,roots:(r,other),root_words});
            }
        }
        self.examined=end;
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
            else {
                self.retained_cells+=relation.cells.len()+relation.history.len();
                self.basis.insert(pivot,relation);return None;
            }
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
    fn polynomial(&mut self, observe: &mut impl FnMut(char,usize,usize,usize)) -> Option<(Tape,Tape,Tape)> {
        observe('≻',self.columns.len(),self.basis.len(),self.partial.len());
        // Each enclosing coefficient owns a product of distinct prime squares.
        // Its sign siblings share that coefficient, its roots and its inverses.
        // The sign register grows with the chosen product; no machine shift
        // bounds the number of siblings.
        if self.family_columns==self.columns.len() {
            if let Some(family)=&mut self.family {
                let mut position=1;
                while position<family.signs.len() && family.signs[position] {
                    family.signs[position]=false;position+=1;
                }
                if position<family.signs.len() {
                    family.signs[position]=true;
                    let mut b=vec![ZERO];
                    for (term,&negative) in family.terms.iter().zip(&family.signs) {
                        let term=if negative {mf::sub(&family.a,term)} else {term.clone()};
                        b=mf::modulo(&mf::add(&b,&term),&family.a);
                    }
                    family.b=b;
                    return Some((family.a.clone(),family.b.clone(),family.root.clone()));
                }
                // Advance the prime-subset register after all its phase
                // siblings return. Exhaustion expands the enclosing field.
                let width=self.family_choice.len();
                let mut position=width;
                while position>0 && self.family_choice[position-1]==self.family_pool.len()-width+position-1 {
                    position-=1;
                }
                if position==0 {self.family=None;return None;}
                self.family_choice[position-1]+=1;
                for i in position..width {self.family_choice[i]=self.family_choice[i-1]+1;}
            }
        } else {
            self.family=None;
            self.family_columns=self.columns.len();
            let largest=self.columns.last()?.step;
            let mut width=1;
            let mut extent=address(largest);
            while mf::cmp(&extent,&self.coefficient_seed)==core::cmp::Ordering::Less {
                extent=mf::mul(&extent,&address(largest));width+=1;
            }
            // Find the per-prime coordinate of the source-derived coefficient.
            // Comparisons use complete tapes, including above a machine word.
            let (mut low,mut high)=(2,largest);
            while low<high {
                let middle=low+(high-low).div_ceil(2);
                let mut product=vec![ONE];
                for _ in 0..width {product=mf::mul(&product,&address(middle));}
                if mf::cmp(&product,&self.coefficient_seed)==core::cmp::Ordering::Greater {high=middle-1;}
                else {low=middle;}
            }
            let center=low;
            self.family_pool=(1..self.columns.len()).filter(|&i|self.columns[i].step>2).collect();
            self.family_pool.sort_unstable_by_key(|&i|self.columns[i].step.abs_diff(center));
            if self.family_pool.len()<width {return None;}
            self.family_choice=(0..width).collect();
        }
        let selected:Vec<usize>=self.family_choice.iter().map(|&i|self.family_pool[i]).collect();
        let mut coefficient_root=vec![ONE];
        for &column in &selected {coefficient_root=mf::mul(&coefficient_root,&self.columns[column].prime);}
        let a=mf::mul(&coefficient_root,&coefficient_root);
        let mut terms=Vec::new();
        let mut b=vec![ZERO];
        observe('⊞',self.columns.len(),self.basis.len(),self.partial.len());
        for &column in &selected {
            let field=&self.columns[column];
            let q=&field.prime;
            let r=&field.roots.0;
            let square=mf::mul(q,q);
            let (difference,remainder)=mf::divmod(&mf::sub(&self.n,&mf::mul(r,r)),q);
            assert!(mf::zero(&remainder));
            let inverse=mf::mod_inv(&mf::add(r,r),q).expect("lifted coefficient root is invertible");
            let lifted=mf::add(r,&mf::mul(q,&mm(&difference,&inverse,q)));
            let (other,remainder)=mf::divmod(&a,&square);
            assert!(mf::zero(&remainder));
            let inverse=mf::mod_inv(&other,&square).expect("distinct coefficient fields are coprime");
            let term=mf::mul(&other,&mm(&lifted,&inverse,&square));
            b=mf::modulo(&mf::add(&b,&term),&a);
            terms.push(term);
        }
        observe('⋈',self.columns.len(),self.basis.len(),self.partial.len());
        let folded_a=mf::FoldedTape::new(&a);
        let inverses=self.columns.iter().map(|field| {
            if field.step==0 {None} else {
                mf::inverse_residue_word(folded_a.remainder_word(field.step as u64),field.step as u64)
            }
        }).collect();
        assert_eq!(mf::modulo(&mf::mul(&b,&b),&a),mf::modulo(&self.n,&a));
        self.family=Some(PolynomialFamily {a:a.clone(),b:b.clone(),root:coefficient_root.clone(),
            signs:vec![false;terms.len()],terms,inverses});
        Some((a,b,coefficient_root))
    }
    pub fn advance(&mut self, mut observe: impl FnMut(char,usize,usize,usize)) -> Option<(Tape,Tape)> {
        if self.examined<self.bound {self.grow(&mut observe);}
        if let Some(p)=self.ready.take() {return Some((p.clone(),mf::divmod(&self.n,&p).0));}
        if self.examined<self.bound {return None;}
        let Some((a,b,coefficient_root))=self.polynomial(&mut observe) else {
            self.expand_field();return None;
        };
        let half=self.span/2;
        let half_tape=address(half);
        self.hit_heads.resize(self.span,0);self.hit_heads.fill(0);
        self.weights.resize(self.span,0);self.weights.fill(0);
        self.hit_links.clear();
        let folded_b=mf::FoldedTape::new(&b);
        for (column,field) in self.columns.iter().enumerate() {
            if column.is_power_of_two() {observe('∋',self.columns.len(),self.basis.len(),self.partial.len());}
            if field.step==0 {continue;}
            let inverse=self.family.as_ref().unwrap().inverses[column];
            let reduced_b_word=folded_b.remainder_word(field.step as u64);
            let (first,second)=if inverse.is_none() {
                let reduced_b=address(reduced_b_word as usize);
                let modulus=&field.prime;
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
                let inverse=inverse.unwrap();
                let position=|root:u64| {
                    let modulus=field.step as u64;
                    if u32::try_from(modulus).is_ok() {
                        let displacement=(root+modulus-reduced_b_word)%modulus;
                        ((displacement*inverse+(half as u64)%modulus)%modulus) as usize
                    } else {
                        let modulus=u128::from(modulus);
                        let displacement=(u128::from(root)+modulus-u128::from(reduced_b_word))%modulus;
                        ((displacement*u128::from(inverse)+half as u128)%modulus) as usize
                    }
                };
                (position(field.root_words.0),position(field.root_words.1))
            };
            for begin in [Some(first),(second!=first).then_some(second)].into_iter().flatten() {
                let mut index=begin;
                while index<self.span {
                    self.hit_links.push((column,self.hit_heads[index]));
                    self.hit_heads[index]=self.hit_links.len();
                    self.weights[index]+=field.prime.len()-1;index+=field.step;
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
            if self.weights[index]+slack<expected {continue;}
            let term=mf::mul(&a,&address(index.abs_diff(half)));
            let x=if index>=half {mf::add(&term,&b)}
                else if mf::cmp(&term,&b)==core::cmp::Ordering::Less {mf::sub(&b,&term)}
                else {mf::sub(&term,&b)};
            let square=mf::mul(&x,&x);
            let negative=mf::cmp(&square,&self.n)==core::cmp::Ordering::Less;
            let difference=if negative {mf::sub(&self.n,&square)} else {mf::sub(&square,&self.n)};
            let (residue,remainder)=mf::divmod(&difference,&a);
            assert!(mf::zero(&remainder));
            if mf::zero(&residue) {return Some((x.clone(),x));}
            let mut folded_residue=mf::FoldedTape::new(&residue);
            let mut cells=if negative {vec![0]} else {Vec::new()};
            let mut y=coefficient_root.clone();
            let mut link=self.hit_heads[index];
            while link!=0 {
                let (column,next)=self.hit_links[link-1];link=next;
                let field=&self.columns[column]; let mut odd=false;
                while folded_residue.divide_word_exact(field.step as u64) {
                    odd=!odd;
                    if !odd {y=mm(&y,&field.prime,&self.n);}
                }
                if odd {cells.push(column);}
            }
            let residue=folded_residue.into_tape();
            cells.sort_unstable();
            let leaf=self.witnesses.len();
            self.retained_cells+=cells.len()+x.len()+y.len()+residue.len();
            self.witnesses.push(Witness {cells:cells.clone(),x,y,residual:residue.clone()});
            let mut relation=Relation {cells,history:vec![leaf]};
            if residue!=vec![ONE] {
                if let Some(previous)=self.partial.remove(&residue) {
                    self.retained_cells-=previous.cells.len()+previous.history.len();
                    relation=self.join(relation,&previous);
                } else {
                    self.retained_cells+=relation.cells.len()+relation.history.len();
                    self.partial.insert(residue,relation);continue;
                }
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
        if useful==0 && self.partial.len()==partial_before {self.expand_field();}
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sibling_fields_retain_the_random_192_source_congruence() {
        let n=mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_192.imasm").trim()).unwrap();
        let mut phase=SquarePhase::new(&n,&mf::isqrt(&n));
        while phase.examined<phase.bound {phase.grow(&mut |_,_,_,_|{});}
        let mut siblings=0;
        let mut previous=None;
        let mut distinct=alloc::collections::BTreeSet::new();
        for _ in 0..n.len() {
            let (a,b,q)=phase.polynomial(&mut |_,_,_,_|{}).unwrap();
            assert_eq!(mf::mul(&q,&q),a);
            assert_eq!(mm(&b,&b,&a),mf::modulo(&n,&a));
            if previous.as_ref()==Some(&a) {siblings+=1;}
            previous=Some(a.clone());
            assert!(distinct.insert((a.clone(),b)));
            let folded=mf::FoldedTape::new(&a);
            for (column,inverse) in phase.columns.iter().zip(&phase.family.as_ref().unwrap().inverses).skip(1) {
                let residue=folded.remainder_word(column.step as u64);
                if let Some(inverse)=inverse {
                    assert_eq!((u128::from(residue)*u128::from(*inverse))%(column.step as u128),1);
                } else {assert_eq!(residue,0);}
            }
        }
        assert!(siblings>0);
        let retained=phase.columns.iter().map(|c|c.prime.len()+c.roots.0.len()+c.roots.1.len()).sum::<usize>();
        assert_eq!(phase.retained_cells,retained);
    }
    #[test]
    fn modular_roots_match_the_resident_2048_bit_source() {
        let n=mf::parse_numeral(include_str!("../tests/fixtures/random_rsa_2048.imasm").trim()).unwrap();
        assert_eq!(n.len(),2048);
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
