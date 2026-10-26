//! Exact paired division and feedback. Every numerical value is a BelnapWord;
//! machine indices address cells and arms, and never replace their evidence.
use serde_json::{json, Value};
use crate::{belnap::B4, belnap_word::BelnapWord as Word, morphism_factor as mf,
    pwoat_torus::BitBraid};
use alloc::{vec,vec::Vec,string::{String,ToString}};
use core::cmp::Ordering;
#[path = "stark_rotation.rs"]
mod stark_rotation;

pub const OPERATIONS: [&str; 8] = ["inverse-rotation", "forward-rotation",
    "source-jump", "pinch-braid", "reverse-pinch-braid", "winding",
    "evidence-reentry", "source-center"];

fn small(n: usize) -> Word { Word::from_parity(&mf::tape_u64(n as u64)) }
fn normalize(x: Word) -> Word { Word::new(x.0) }
fn low(x: &Word, width: usize) -> Word {
    Word::new((0..width).map(|i| x.cell(i)).collect())
}
fn high(x: &Word, width: usize) -> Word {
    Word::new(x.0.iter().skip(width).copied().collect())
}
fn definite(x: &Word) -> Result<(), String> { x.to_parity().map(|_|()).map_err(str::to_string) }
fn subtract(a: &Word, b: &Word) -> Result<Word, String> {
    let (d, borrow) = a.subtract(b);
    if borrow != B4::F { return Err("subtraction retains unresolved borrow".into()); }
    Ok(normalize(d))
}
fn inverse(p: &Word, width: usize) -> Result<Word, String> {
    definite(p)?;
    if p.cell(0) != B4::T { return Err("inverse requires explicit odd definite restriction".into()); }
    let mut x = small(1);
    let mut precision = 1;
    while precision < width {
        precision = (precision*2).min(width);
        // 2-p*x in the fixed-width ring, with its wrap retained by the pair's height.
        let px = low(&p.multiply(&x),precision);
        let complement=Word::new((0..precision).map(|i|px.cell(i).bnot()).collect());
        let correction = low(&small(2).add(&complement).add(&small(1)),precision);
        x = low(&x.multiply(&correction),precision);
    }
    if low(&p.multiply(&x),width) != small(1) { return Err("inverse failed exact ring closure".into()); }
    Ok(x)
}
fn power(base: &Word, exponent: &Word, width: usize) -> Result<Word,String> {
    definite(exponent)?;
    let mut value=small(1); let mut base=base.clone();
    for cell in &exponent.0 {
        match cell { B4::T=>value=low(&value.multiply(&base),width), B4::F=>(),
            _=>return Err("power retains unresolved exponent evidence".into()) }
        base=low(&base.multiply(&base),width);
    }
    Ok(value)
}
fn dyadic(place:usize)->Word {
    let mut cells=vec![B4::F;place];cells.push(B4::T);Word::new(cells)
}
/// Lift a supplied relative residue in 1+8Z to the exact rotation index.
/// The correction powers and the exponent remain Belnap words throughout.
fn rotation_index(base:&Word,value:&Word,width:usize)->Result<Word,String>{
    definite(value)?;
    if low(value,3)!=small(1){return Err("rotation changes the retained odd coset".into());}
    let mut current=small(1);let mut exponent=small(0);let mut correction=base.clone();
    for precision in 3..width {
        if current.cell(precision)!=value.cell(precision) {
            current=low(&current.multiply(&correction),width);
            exponent=exponent.add(&dyadic(precision-3));
        }
        correction=low(&correction.multiply(&correction),width);
    }
    if current!=low(value,width){return Err("rotation index failed exact residue recovery".into());}
    Ok(exponent)
}
fn gcd(mut a:Word,mut b:Word)->Result<Word,String>{
    definite(&a)?;definite(&b)?;
    while b!=small(0){let r=a.divide(&b).map_err(str::to_string)?.1;a=b;b=normalize(r);}
    Ok(a)
}
pub fn word(x:&Word)->Value {
    json!({"cells":x.0.iter().map(|c|c.name()).collect::<Vec<_>>(),
        "numeral":x.to_parity().ok().map(|t|mf::dec_of(&t))})
}

struct Arm {
    p:Word,q:Word,height:Word,negative:B4,
    origin_p:Word,origin_q:Word,
    poloidal_half_turns:Word,pinch_crossings:Word,
    positive_turns:Word,negative_turns:Word,
    evidence:B4, ancestry:Vec<Value>,
    stark_frame:Word,
}
pub struct Torus {
    source:Word,width:usize,step:Word,back:Word,center:Word,
    arms:Vec<Arm>,grid:Vec<B4>,rule:B4,clock:Word,
    braid:BitBraid,observations:Vec<Value>,
    native_views:Vec<(Word,Word,B4)>,
    reentry_state:Word,
    next_arm:usize,
    stark:stark_rotation::StarkRotation,
}
impl Torus {
    pub fn new(source:Word)->Result<Self,String>{
        definite(&source)?;
        if source.0.len()<128{return Err("toroidal implementation requires at least128 source cells".into());}
        if source.cell(0)!=B4::T {return Err("odd toroidal source restriction".into());}
        let width=source.0.len();
        let tape=source.to_parity().map_err(str::to_string)?;
        let center=Word::from_parity(&mf::isqrt(&tape));
        let mut shifted=vec![B4::F;4];
        shifted.extend((0..4).map(|i|source.cell(i)));
        let step=small(9).add(&Word::new(shifted));
        let back=inverse(&step,width)?;
        let period=dyadic(width-3);
        if power(&step,&period,width)?!=small(1)
            || power(&step,&dyadic(width-4),width)?==small(1) {
            return Err("source rotation failed exact full-period closure".into());
        }
        let mut body=Self{source,width,step,back,center,arms:Vec::new(),
            grid:vec![B4::B;48],rule:B4::B,clock:small(0),braid:BitBraid::torus_twist(width),
            observations:Vec::new(),native_views:Vec::new(),reentry_state:small(0),next_arm:0,
            stark:stark_rotation::StarkRotation::new(width)};
        for coset in [1,3,5,7] {
            let mut p=body.center.clone();p.0.resize(width,B4::F);
            for i in 0..3 {p.0[i]=small(coset).cell(i);}
            let p=normalize(p);
            let q=low(&body.source.multiply(&inverse(&p,width)?),width);
            let (height,negative)=body.height(&p,&q)?;
            body.arms.push(Arm{origin_p:p.clone(),origin_q:q.clone(),p,q,height,negative,
                positive_turns:small(0),negative_turns:small(0),
                poloidal_half_turns:small(0),pinch_crossings:small(0),
                evidence:B4::N,ancestry:Vec::new(),stark_frame:small(9)});
        }
        Ok(body)
    }
    /// Transport the complete N,T,F,B membership mask into the same paired
    /// orbit. Differences of mask coordinates compose by telescoping; inverse
    /// logical actions therefore recover the exact arithmetic pair.
    pub fn reenter(&mut self,before:u8,after:u8,kind:u8)->Result<Value,String>{
        if before>15 || after>15 || self.reentry_state!=small(before as usize){
            return Err("re-entry lost its retained four-atom state".into());
        }
        let expected=match kind {
            0=>((before&5)<<1)|((before&10)>>1),
            1=>{let mut value=before;for bit in [1u8,2,4,8]{if value&bit==0{value|=bit;break;}}value},
            2=>15^(((before&1)<<3)|((before&2)<<1)|((before&4)>>1)|((before&8)>>3)),
            _=>return Err("unknown re-entry operation".into()),
        };
        if after!=expected{return Err("re-entry operation disagrees with native membership transport".into());}
        let reverse=after<before;
        let lift=small(before.abs_diff(after) as usize);
        let (step,back)=if reverse{(&self.back,&self.step)}else{(&self.step,&self.back)};
        let forward=power(step,&lift,self.width)?;let backward=power(back,&lift,self.width)?;
        let operation=match kind{0=>"permutation-reentry",1=>"inc-ascent",_=>"conflation-return"};
        for arm in 0..self.arms.len(){
            let p=self.arms[arm].p.clone();let q=self.arms[arm].q.clone();
            let next=low(&p.multiply(&forward),self.width);
            let paired=low(&q.multiply(&backward),self.width);
            if low(&next.multiply(&backward),self.width)!=p || low(&paired.multiply(&forward),self.width)!=q{
                return Err("logical re-entry lost paired inverse recovery".into());
            }
            self.install(arm,next,paired,operation,lift.clone(),reverse)?;
        }
        self.reentry_state=small(after as usize);
        let shadow=B4::from_u8((after>>1)&3);
        Ok(json!({"operation":operation,"before":word(&small(before as usize)),
            "after":word(&self.reentry_state),"atom_order":["N","T","F","B"],
            "TF_shadow":shadow.name(),"designated":after&2!=0,"inverse_verified":true}))
    }
    fn height(&self,p:&Word,q:&Word)->Result<(Word,B4),String>{
        let product=p.multiply(q);
        let negative=match product.compare(&self.source).map_err(str::to_string)?{
            Ordering::Less=>B4::T,_=>B4::F};
        let delta=if negative==B4::T{subtract(&self.source,&product)?}else{subtract(&product,&self.source)?};
        if low(&delta,self.width)!=small(0){return Err("paired rotation lost source residue".into());}
        Ok((high(&delta,self.width),negative))
    }
    fn install(&mut self,arm:usize,p:Word,q:Word,operation:&str,lift:Word,reverse:bool)->Result<(),String>{
        let (height,negative)=self.height(&p,&q)?;
        let a=&mut self.arms[arm];
        let before=json!({"p":word(&a.p),"q":word(&a.q),"height":word(&a.height),"negative":a.negative.name()});
        if reverse {a.negative_turns=a.negative_turns.add(&lift);}else{a.positive_turns=a.positive_turns.add(&lift);}
        a.ancestry.push(json!({"operation":operation,"before":before,"lift":word(&lift),
            "reverse":reverse,"after_p":word(&p),"after_q":word(&q),
            "after_height":word(&height),"negative":negative.name()}));
        a.p=p;a.q=q;a.height=height;a.negative=negative;
        Ok(())
    }
    fn feedback(&mut self,arm:usize,prior_height:&Word)->B4{
        let a=&self.arms[arm];
        // Distinct observations occupy their own rails. Accumulation can retain B.
        let support=match a.height.compare(prior_height) {
            Ok(Ordering::Less)=>B4::T,Ok(Ordering::Greater)=>B4::F,
            Ok(Ordering::Equal)=>if a.height==small(0){B4::T}else{B4::N},Err(_)=>B4::B};
        self.arms[arm].evidence=self.arms[arm].evidence.join(support);
        let mut next=self.grid.clone();
        for r in 0..3 {for c in 0..16 {
            let left=self.grid[r*16+(c+15)%16];
            let right=self.grid[((r+1)%3)*16+c];
            let fused=if left==B4::B && right==B4::B {self.rule}else{left.join(right)};
            // Both coordinates are periodically glued, and source evidence is injected by join.
            next[r*16+c]=fused.join(self.source.cell((r*16+c)%self.width));
        }}
        next[arm%3*16+arm%16]=next[arm%3*16+arm%16].join(support);
        self.grid=next;
        let prior_rule=self.rule;
        self.rule=support.bnot();
        self.clock=self.clock.add(&small(1));
        self.observations.push(json!({"arm":arm,"height_progress":support.name(),
            "height_zero":self.arms[arm].height==small(0),"prior_rule":prior_rule.name(),
            "retained_evidence":self.arms[arm].evidence.name(),"rule":self.rule.name(),"clock":word(&self.clock)}));
        support
    }
    pub fn act(&mut self,port:usize)->Result<(Option<(Word,Word)>,Value,B4),String>{
        let arm=self.next_arm;
        self.next_arm=(arm+1)%self.arms.len();
        self.act_on_arm(port,arm)
    }
    /// Continue a compound operation on the pair just transformed. Recording
    /// its observation advances the clock, but must not change its operand.
    pub fn continue_action(&mut self,port:usize)->Result<(Option<(Word,Word)>,Value,B4),String>{
        if self.observations.is_empty(){return Err("continued torus action has no carried arm".into());}
        let arm=self.observations.last().unwrap()["arm"].as_u64()
            .ok_or("continued torus action lost its arm")? as usize;
        self.act_on_arm(port,arm)
    }
    fn act_on_arm(&mut self,port:usize,arm:usize)->Result<(Option<(Word,Word)>,Value,B4),String>{
        let port=port%OPERATIONS.len();
        // The source and carried clock select addresses; values stay in FOUR.
        let p=self.arms[arm].p.clone();let q=self.arms[arm].q.clone();
        let prior_height=self.arms[arm].height.clone();
        let exponent=match port {
            2=>low(&self.source,12).add(&self.clock),
            5=>self.clock.add(&small(1)),_=>small(1)};
        match port {
            3|4=>{
                let entry=self.arms[arm].poloidal_half_turns.clone();
                let at_pinch=if entry.cell(0)==B4::F{entry.add(&small(1))}else{entry.clone()};
                // Odd half-turns are exactly theta=pi modulo2pi. The carried
                // poloidal coordinate reaches the pinch before the crossing.
                self.arms[arm].poloidal_half_turns=at_pinch.clone();
                let mut marks=p.0.clone();marks.resize(self.width,B4::F);
                if port==3 {for &i in &self.braid.generators {marks.swap(i,i+1);}}
                else {for &i in self.braid.generators.iter().rev(){marks.swap(i,i+1);}}
                let next=Word::new(marks);
                let rebound=low(&self.source.multiply(&inverse(&next,self.width)?),self.width);
                let relative=low(&next.multiply(&inverse(&p,self.width)?),self.width);
                let base=if port==4{&self.back}else{&self.step};
                let lift=rotation_index(base,&relative,self.width)?;
                self.install(arm,next,rebound,OPERATIONS[port],lift,port==4)?;
                let returned=at_pinch.add(&small(1));
                let a=&mut self.arms[arm];
                a.poloidal_half_turns=returned.clone();a.pinch_crossings=a.pinch_crossings.add(&small(1));
                let transition=a.ancestry.last_mut().unwrap();
                transition["poloidal_entry_half_turns"]=word(&entry);
                transition["poloidal_pinch_half_turns"]=word(&at_pinch);
                transition["poloidal_return_half_turns"]=word(&returned);
            },
            7=>{
                let offset=self.clock.multiply(&small(8)).add(&small(2*arm+1));
                let mut next=self.center.add(&offset);
                for i in 0..3 {next.0[i]=p.cell(i);}
                let next=low(&next,self.width);
                let rebound=low(&self.source.multiply(&inverse(&next,self.width)?),self.width);
                let relative=low(&next.multiply(&inverse(&p,self.width)?),self.width);
                let lift=rotation_index(&self.step,&relative,self.width)?;
                self.install(arm,next,rebound,OPERATIONS[port],lift,false)?;
            },
            6 if !self.native_views.is_empty()=>{
                let (left,right,conjugate)=self.native_views.last().unwrap();
                let (mut left_value,mut right_value,orientation)=Self::coordinates(&p,&q,*conjugate)?;
                // Known cells act on the carried hypothesis. N and B remain
                // in the native view and evidence; this numerical branch
                // retains its prior choice at those unresolved positions.
                for (value,view) in [(&mut left_value,left),(&mut right_value,right)] {
                    // A finite numerical word has definite F above its
                    // addressed extent. Those high zeros constrain this same
                    // coordinate; they are not unobserved native cells.
                    for i in 0..value.0.len().max(view.0.len()) {
                        let cell=view.cell(i);
                        if matches!(cell,B4::T|B4::F) {
                            value.0.resize(value.0.len().max(i+1),B4::F);value.0[i]=cell;
                        }
                    }
                }
                let mut next=if *conjugate==B4::T {
                    if orientation==B4::T {
                        match left_value.compare(&right_value).map_err(str::to_string)? {
                            Ordering::Less=>subtract(&right_value,&left_value)?,_=>subtract(&left_value,&right_value)?}
                    }else{left_value.add(&right_value)}
                }else{left_value};
                next.0.resize(self.width,B4::F);
                for i in 0..3 {next.0[i]=p.cell(i);}
                let next=low(&next,self.width);
                let rebound=low(&self.source.multiply(&inverse(&next,self.width)?),self.width);
                let relative=low(&next.multiply(&inverse(&p,self.width)?),self.width);
                let lift=rotation_index(&self.step,&relative,self.width)?;
                self.install(arm,next,rebound,"native-view-reentry",lift,false)?;
                self.arms[arm].ancestry.last_mut().unwrap()["native_view_index"]=json!(self.native_views.len()-1);
                self.arms[arm].ancestry.last_mut().unwrap()["zero_extended_view"]=json!(true);
            },
            _=>{
                let reverse=port==0 || (port==6 && self.grid[arm]==B4::F);
                let (forward,backward)=if reverse{(&self.back,&self.step)}else{(&self.step,&self.back)};
                let f=power(forward,&exponent,self.width)?;let b=power(backward,&exponent,self.width)?;
                let next=low(&p.multiply(&f),self.width);let paired=low(&q.multiply(&b),self.width);
                // Inverse is checked on the actual carried values before commit.
                if low(&next.multiply(&b),self.width)!=p || low(&paired.multiply(&f),self.width)!=q {
                    return Err("opposite rotations failed inverse recovery".into());
                }
                self.install(arm,next,paired,OPERATIONS[port],exponent,reverse)?;
            }
        }
        let selected_transition=self.arms[arm].ancestry.last().cloned();
        let mut result=None;let mut stark_transition=Value::Null;
        if port!=6 {
            result=self.stark.advance(&self.source,port,&self.clock)?;
            let before_frame=self.arms[arm].stark_frame.clone();
            let after_frame=low(&self.stark.multiplier(),self.width);
            let multiplier=low(&after_frame.multiply(&inverse(&before_frame,self.width)?),self.width);
            let backward=inverse(&multiplier,self.width)?;
            let p=self.arms[arm].p.clone();let q=self.arms[arm].q.clone();
            let next=low(&p.multiply(&multiplier),self.width);
            let paired=low(&q.multiply(&backward),self.width);
            if low(&next.multiply(&backward),self.width)!=p
                ||low(&paired.multiply(&multiplier),self.width)!=q {
                return Err("Stark-driven paired rotation failed inverse recovery".into());
            }
            let lift=rotation_index(&self.step,&multiplier,self.width)?;
            self.install(arm,next,paired,"stark-unit-paired-rotation",lift,false)?;
            stark_transition=self.arms[arm].ancestry.last().cloned().unwrap();
            stark_transition["multiplier"]=word(&multiplier);
            stark_transition["inverse_multiplier"]=word(&backward);
            stark_transition["before_unit_frame"]=word(&before_frame);
            stark_transition["after_unit_frame"]=word(&after_frame);
            self.arms[arm].stark_frame=after_frame;
        }
        let readout=if result.is_some(){"stark-unit"}else{"paired-orbit"};
        let evidence=self.feedback(arm,&prior_height);
        for candidate in [&self.arms[arm].p,&self.arms[arm].q] {
            if result.is_some(){break;}
            let divisor=gcd(self.source.clone(),candidate.clone())?;
            if divisor.compare(&small(1)).map_err(str::to_string)?==Ordering::Greater
                && divisor.compare(&self.source).map_err(str::to_string)?==Ordering::Less {
                let (quotient,remainder)=self.source.divide(&divisor).map_err(str::to_string)?;
                if normalize(remainder)!=small(0) || divisor.multiply(&quotient)!=self.source {
                    return Err("toroidal divisor failed exact source closure".into());
                }
                result=Some((divisor,quotient));break;
            }
        }
        Ok((result,json!({"operation":OPERATIONS[port],"arm":arm,"evidence":evidence.name(),
            "transition":selected_transition,"stark_paired_transition":stark_transition,
            "readout_component":readout,"clock":word(&self.clock),"axes":self.axes()}),evidence))
    }
    pub fn axes(&self)->[usize;12]{
        let mut axes=self.braid.axis_permutation();
        for i in 0..12 {
            // Execute each rail's action separately. B executes both actions;
            // it does not erase the current cell or its exact arithmetic state.
            for evidence in [self.grid[i],self.arms[i%4].evidence] {
                let (support,refutation)=evidence.to_wh2();
                if support {axes.swap(i,(i+1)%12);}
                if refutation {axes.swap(i,(i+5)%12);}
            }
            let (support,refutation)=self.arms[i%4].p.cell(i).to_wh2();
            if support {axes.swap(i,(i+3)%12);}
            if refutation {axes.swap(i,(i+7)%12);}
        }
        axes
    }
    pub fn support(&self)->(Word,Word,B4){
        let selected=self.observations.last().and_then(|o|o["arm"].as_u64()).unwrap_or(0) as usize;
        let a=&self.arms[selected];
        (a.p.clone(),a.q.clone(),a.evidence)
    }
    pub fn clock_word(&self)->Word{self.clock.clone()}
    /// A full-complex SIC readout supplies a lifted rotation of the SAME
    /// observed pair. The caller retains its frame, masses and coherences.
    pub fn rotate_from_sic(&mut self,lift:Word,reverse:bool)
        ->Result<(Option<(Word,Word)>,Value,B4),String>{
        definite(&lift)?;
        let arm=self.observations.last().and_then(|o|o["arm"].as_u64())
            .ok_or("SIC rotation has no observed paired arm")? as usize;
        let p=self.arms[arm].p.clone();let q=self.arms[arm].q.clone();
        let height=self.arms[arm].height.clone();
        let (step,back)=if reverse{(&self.back,&self.step)}else{(&self.step,&self.back)};
        let f=power(step,&lift,self.width)?;let b=power(back,&lift,self.width)?;
        let next=low(&p.multiply(&f),self.width);let paired=low(&q.multiply(&b),self.width);
        if low(&next.multiply(&b),self.width)!=p || low(&paired.multiply(&f),self.width)!=q {
            return Err("SIC lifted rotation lost its paired inverse".into());
        }
        self.install(arm,next,paired,"sic-coherence-rotation",lift,reverse)?;
        let evidence=self.feedback(arm,&height);
        let mut result=None;
        for value in [&self.arms[arm].p,&self.arms[arm].q] {
            let d=gcd(self.source.clone(),value.clone())?;
            if d!=small(1)&&d!=self.source {
                let (q,r)=self.source.divide(&d).map_err(str::to_string)?;
                if normalize(r)!=small(0)||d.multiply(&q)!=self.source{return Err("SIC rotation failed source closure".into());}
                result=Some((d,q));break;
            }
        }
        Ok((result,json!({"arm":arm,"operation":"sic-coherence-rotation",
            "transition":self.arms[arm].ancestry.last(),"clock":word(&self.clock),
            "evidence":evidence.name(),"inverse_verified":true}),evidence))
    }
    fn coordinates(p:&Word,q:&Word,conjugate:B4)->Result<(Word,Word,B4),String>{
        if conjugate==B4::F{return Ok((p.clone(),q.clone(),B4::T));}
        if conjugate!=B4::T{return Err("coordinate transport retains an unresolved frame".into());}
        let orientation=if p.compare(q).map_err(str::to_string)?==Ordering::Greater{B4::F}else{B4::T};
        let gap=if orientation==B4::T{subtract(q,p)?}else{subtract(p,q)?};
        let (midpoint,midpoint_remainder)=p.add(q).divide(&small(2)).map_err(str::to_string)?;
        let (gap,gap_remainder)=gap.divide(&small(2)).map_err(str::to_string)?;
        if midpoint_remainder!=small(0)||gap_remainder!=small(0){
            return Err("odd-pair coordinate transport lost a half unit".into());
        }
        Ok((midpoint,gap,orientation))
    }
    /// Observe another coordinate view of this same resident, retaining FOUR
    /// cell values and their compatibility with each paired arm.
    pub fn observe_native_view(&mut self,left:Word,right:Word,conjugate:B4)->Result<(),String>{
        for (index,a) in self.arms.iter_mut().enumerate(){
            let (p,q,_)=Self::coordinates(&a.p,&a.q,conjugate)?;
            let mut evidence=B4::N;
            for (actual,view) in [(&p,&left),(&q,&right)]{
                for i in 0..actual.0.len().max(view.0.len()){
                    let value=view.cell(i);
                    evidence=evidence.join(match value {
                        B4::N=>B4::N,B4::B=>B4::B,
                        _=>if actual.cell(i)==value{B4::T}else{B4::F},
                    });
                }
            }
            a.evidence=a.evidence.join(evidence);
            self.grid[index*4]=self.grid[index*4].join(evidence);
        }
        self.native_views.push((left,right,conjugate));
        Ok(())
    }
    /// Consume an internally produced divisor through exact Belnap division.
    /// This is return integration; the caller preserves the producing port.
    pub fn retain_closed_pair(&mut self,p:Word,q:Word)->Result<(),String>{
        let (derived,remainder)=self.source.divide(&p).map_err(str::to_string)?;
        if normalize(remainder)!=small(0) || derived!=q || p.multiply(&q)!=self.source {
            return Err("resident pair failed joint toroidal division return".into());
        }
        let arm=self.arms.iter().position(|a|low(&a.p,3)==low(&p,3))
            .ok_or("closed divisor escaped all retained odd cosets")?;
        let previous=self.arms[arm].height.clone();
        let relative=low(&p.multiply(&inverse(&self.arms[arm].p,self.width)?),self.width);
        let lift=rotation_index(&self.step,&relative,self.width)?;
        self.install(arm,p,q,"resident-divisor-to-joint-division",lift,false)?;
        self.feedback(arm,&previous);
        Ok(())
    }
    /// The selected paired state enters the learner without projecting away
    /// either evidence rail. Historical ancestry stays in this same torus.
    pub fn learning_context(&self)->Value{
        let selected=self.observations.last().and_then(|o|o["arm"].as_u64()).unwrap_or(0) as usize;
        let arm=&self.arms[selected];
        let fields=[("source",self.source.clone()),("p",arm.p.clone()),("q",arm.q.clone()),
            ("height",arm.height.clone()),("negative",Word::new(vec![arm.negative])),
            ("positive_turns",arm.positive_turns.clone()),("negative_turns",arm.negative_turns.clone()),
            ("clock",self.clock.clone()),("membership",self.reentry_state.clone()),
            ("evidence",Word::new(vec![arm.evidence])),("grid",Word::new(self.grid.clone()))];
        json!({"schema":"toroidal-learning-context-v1","arm":selected,
            "fields":fields.iter().map(|(name,value)|json!({"name":name,"word":word(value)})).collect::<Vec<_>>()})
    }
    pub fn snapshot(&self)->Value{
        let mut period=vec![B4::F;self.width-3];period.push(B4::T);
        let period=Word::new(period);
        json!({"carrier":"BelnapWord","source":word(&self.source),"modulus_exponent":self.width,
            "stark_frame_transport":"per-arm frame ratio",
            "stark_rotation":self.stark.snapshot(),
            "native_view_action_same_arm":true,"next_arm":self.next_arm,
            "reentry_state":word(&self.reentry_state),"reentry_atom_order":["N","T","F","B"],
            "orbit_period":word(&period),"full_period_verified":true,
            "numerical_restriction":"both rails checked definite; operands retained",
            "step":word(&self.step),"inverse_step":word(&self.back),"clock":word(&self.clock),
            "grid":self.grid.iter().map(|c|c.name()).collect::<Vec<_>>(),"rule":self.rule.name(),
            "pinch":{"poloidal_numerator":1,"poloidal_denominator":2,
                "crossings":self.braid.generators,"burau_word":self.braid.burau_word},
            "observations":self.observations,"axes":self.axes(),
            "native_views":self.native_views.iter().map(|(p,q,c)|json!({"left":word(p),"right":word(q),
                "conjugate":c.name()})).collect::<Vec<_>>(),
            "arms":self.arms.iter().map(|a|{
                let (forward_turns,forward_phase)=a.positive_turns.divide(&period)
                    .expect("retained definite winding counter");
                let (reverse_turns,reverse_phase)=a.negative_turns.divide(&period)
                    .expect("retained definite winding counter");
                let (midpoint,gap,orientation)=Self::coordinates(&a.p,&a.q,B4::T)
                    .expect("retained definite paired coordinate frame");
                json!({"p":word(&a.p),"q":word(&a.q),"origin_p":word(&a.origin_p),"origin_q":word(&a.origin_q),
                "stark_frame":word(&a.stark_frame),
                "square_view":{"midpoint":word(&midpoint),"gap":word(&gap),"p_is_lower":orientation.name()},
                "signed_product_height":{"magnitude":word(&a.height),"negative":a.negative.name()},
                "positive_turns":word(&a.positive_turns),"negative_turns":word(&a.negative_turns),
                "rotation_lift":{"forward_full_turns":word(&forward_turns),"forward_phase":word(&forward_phase),
                    "reverse_full_turns":word(&reverse_turns),"reverse_phase":word(&reverse_phase)},
                "poloidal_half_turns":word(&a.poloidal_half_turns),"pinch_crossings":word(&a.pinch_crossings),
                "evidence":a.evidence.name(),"ancestry":a.ancestry})}).collect::<Vec<_>>()})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn logical_reentry_recovers_the_exact_200_bit_paired_state(){
        let source=Word::from_parity(&mf::decimal_to_tape("1306593903193228327879866132486971792765599839798626833574803").unwrap());
        let mut torus=Torus::new(source.clone()).unwrap();
        let before:Vec<_>=torus.arms.iter().map(|a|(a.p.clone(),a.q.clone())).collect();
        for (old,new) in [(0u8,1u8),(1,3),(3,7),(7,15)]{
            let permutation=((old&5)<<1)|((old&10)>>1);
            torus.reenter(old,permutation,0).unwrap();
            torus.reenter(permutation,old,0).unwrap();
            let event=torus.reenter(old,new,1).unwrap();
            assert_eq!(event["designated"],new&2!=0);
            assert_ne!(torus.arms[0].p,before[0].0);
        }
        torus.reenter(15,0,2).unwrap();
        for (arm,(p,q)) in torus.arms.iter().zip(before){
            assert_eq!((&arm.p,&arm.q),(&p,&q));
            assert_eq!(torus.height(&arm.p,&arm.q).unwrap().0,arm.height);
        }
        assert_eq!(torus.reentry_state,small(0));
        assert_eq!(torus.source,source);
        assert!(torus.reenter(0,3,1).is_err());
    }
    #[test]
    fn square_and_division_views_return_the_same_128_bit_carrier(){
        let source=Word::from_parity(&mf::decimal_to_tape("211060082272597355141159798675733025589").unwrap());
        let mut t=Torus::new(source.clone()).unwrap();
        for port in 0..8 {
            t.act(port).unwrap();
            for a in &t.arms {
                let (midpoint,gap,orientation)=Torus::coordinates(&a.p,&a.q,B4::T).unwrap();
                let lower=subtract(&midpoint,&gap).unwrap();let upper=midpoint.add(&gap);
                let (p,q)=if orientation==B4::T{(lower,upper)}else{(upper,lower)};
                assert_eq!((&p,&q),(&a.p,&a.q));
                assert_eq!(subtract(&midpoint.multiply(&midpoint),&gap.multiply(&gap)).unwrap(),p.multiply(&q));
            }
        }
        let mut left=Word(vec![B4::N;128]);left.0[3]=B4::B;left.0[5]=B4::T;
        let right=Word(vec![B4::N;128]);
        t.observe_native_view(left.clone(),right.clone(),B4::T).unwrap();
        t.act(6).unwrap();
        assert_eq!(t.native_views.last().unwrap(),&(left,right,B4::T));
        assert_eq!(t.source,source);
        assert!(t.arms.iter().any(|a|a.evidence==B4::B));
    }
    #[test]
    fn integrated_belnap_torus_keeps_exact_pair_and_feedback_on_128_bit_sources(){
        for raw in ["211060082272597355141159798675733025589",
            "228894146075466224257261280392675893141"] {
            let source=Word::from_parity(&mf::decimal_to_tape(raw).unwrap());
            let mut t=Torus::new(source.clone()).unwrap();
            let initial=t.snapshot();let mut axes=std::collections::BTreeSet::new();
            axes.insert(t.axes());
            for round in 0..3 {for port in 0..8 {
                let (_,event,_)=t.act(port).unwrap();
                assert_eq!(event["operation"],OPERATIONS[port]);
                for a in &t.arms {assert_eq!(low(&a.p.multiply(&a.q),t.width),source);
                    assert_eq!(t.height(&a.p,&a.q).unwrap(),(a.height.clone(),a.negative));
                    let forward=power(&t.step,&a.positive_turns,t.width).unwrap();
                    let backward=power(&t.back,&a.negative_turns,t.width).unwrap();
                    assert_eq!(low(&a.origin_p.multiply(&forward).multiply(&backward),t.width),a.p);
                    let forward=power(&t.back,&a.positive_turns,t.width).unwrap();
                    let backward=power(&t.step,&a.negative_turns,t.width).unwrap();
                    assert_eq!(low(&a.origin_q.multiply(&forward).multiply(&backward),t.width),a.q);
                }
                assert_eq!(t.observations.len(),round*8+port+1);
                axes.insert(t.axes());
            }}
            assert_ne!(t.snapshot(),initial);
            assert!(axes.len()>1);
            assert!(t.arms.iter().any(|a|a.evidence==B4::B));
            assert!(t.arms.iter().all(|a|!a.ancestry.is_empty()));
        }
    }
    #[test]
    fn unresolved_belnap_source_is_retained_at_explicit_numeric_boundary(){
        let source=Word::from_parity(&mf::decimal_to_tape("211060082272597355141159798675733025589").unwrap());
        for value in [B4::N,B4::B] {
            let mut carried=source.clone();carried.0[70]=value;
            let retained=carried.clone();
            assert!(Torus::new(carried.clone()).is_err());
            assert_eq!(carried,retained);
            assert!(inverse(&carried,128).is_err());
        }
    }
}
