//! Dimension-indexed norm-one unit in the integral basis 1, epsilon.
//! epsilon^2 - (d-1)epsilon + 1 = 0. Coefficients stay in Belnap words;
//! reduction modulo the source retains its norm carry and inverse witness.
use super::{Word,small,normalize,subtract,gcd,word};
use crate::belnap::B4;
use alloc::{vec,vec::Vec,string::{String,ToString}};
use serde_json::{json,Value};

#[derive(Clone)]
struct Unit { a:Word,b:Word }
fn reduce(x:Word,n:&Word)->Result<Word,String>{
    Ok(normalize(x.divide(n).map_err(str::to_string)?.1))
}
fn minus(a:&Word,b:&Word,n:&Word)->Result<Word,String>{
    if a.compare(b).map_err(str::to_string)?==core::cmp::Ordering::Less {
        subtract(n,&subtract(b,a)?)
    }else{subtract(a,b)}
}
fn mul(x:&Unit,y:&Unit,t:&Word,n:&Word)->Result<Unit,String>{
    let bd=reduce(x.b.multiply(&y.b),n)?;
    Ok(Unit{a:minus(&reduce(x.a.multiply(&y.a),n)?,&bd,n)?,
        b:reduce(x.a.multiply(&y.b).add(&x.b.multiply(&y.a)).add(&t.multiply(&bd)),n)?})
}
fn conj(x:&Unit,t:&Word,n:&Word)->Result<Unit,String>{
    Ok(Unit{a:reduce(x.a.add(&t.multiply(&x.b)),n)?,
        b:if x.b==small(0){small(0)}else{subtract(n,&x.b)?}})
}
fn power(x:&Unit,k:&Word,t:&Word,n:&Word)->Result<Unit,String>{
    let mut out=Unit{a:small(1),b:small(0)};let mut base=x.clone();
    for c in &k.0 {
        match c {B4::T=>out=mul(&out,&base,t,n)?,B4::F=>(),
            _=>return Err("Stark exponent retains unresolved numerical evidence".into())}
        base=mul(&base,&base,t,n)?;
    }
    Ok(out)
}
fn pair(x:&Unit)->Value{json!({"a":word(&x.a),"b":word(&x.b)})}
pub(super) struct StarkRotation {
    dimension:usize,trace:Word,state:Unit,positive:Word,negative:Word,
    events:Vec<Value>,
}
impl StarkRotation {
    pub fn new(dimension:usize)->Self{
        Self{dimension,trace:small(dimension-1),state:Unit{a:small(0),b:small(1)},
            positive:small(1),negative:small(0),events:vec![]}
    }
    pub fn advance(&mut self,source:&Word,port:usize,clock:&Word)
        ->Result<Option<(Word,Word)>,String>{
        let before=self.state.clone();
        let before_positive=self.positive.clone();let before_negative=self.negative.clone();
        let reverse=port==0||port==4;
        let k=if port==2||port==5{clock.add(&small(2))}else{small(1)};
        let operator=if port==2||port==5 {
            // Repeated winding is a multiplication by the retained unit's
            // (k-1)st power, with its exact conjugate available for return.
            self.positive=self.positive.multiply(&k);self.negative=self.negative.multiply(&k);
            power(&before,&subtract(&k,&small(1))?,&self.trace,source)?
        }else{
            if reverse{self.negative=self.negative.add(&k);}else{self.positive=self.positive.add(&k);}
            let generator=Unit{a:small(0),b:small(1)};
            if reverse{conj(&generator,&self.trace,source)?}else{generator}
        };
        let inverse=conj(&operator,&self.trace,source)?;
        let after=mul(&before,&operator,&self.trace,source)?;
        let recovered=mul(&after,&inverse,&self.trace,source)?;
        if recovered.a!=before.a||recovered.b!=before.b{
            return Err("Stark conjugate failed carried-state inverse recovery".into());
        }
        let norm=after.a.multiply(&after.a).add(&self.trace.multiply(&after.a).multiply(&after.b))
            .add(&after.b.multiply(&after.b));
        let (carry,remainder)=norm.divide(source).map_err(str::to_string)?;
        if normalize(remainder)!=small(1){return Err("Stark rotation lost its norm-one residue".into());}
        let trace=reduce(after.a.multiply(&small(2)).add(&self.trace.multiply(&after.b)),source)?;
        let trace_minus_two=minus(&trace,&small(2),source)?;
        let mut result=None;let mut readouts=Vec::new();
        for (name,value) in [("epsilon_coefficient",&after.b),("trace_minus_two",&trace_minus_two)] {
            let divisor=gcd(source.clone(),value.clone())?;
            readouts.push(json!({"kind":name,"value":word(value),"gcd":word(&divisor)}));
            if divisor!=small(1)&&divisor!=*source {
                let (quotient,remainder)=source.divide(&divisor).map_err(str::to_string)?;
                if normalize(remainder)!=small(0)||divisor.multiply(&quotient)!=*source {
                    return Err("Stark readout failed exact source division".into());
                }
                result=Some((divisor,quotient));break;
            }
        }
        self.events.push(json!({"port":port,"clock":word(clock),"power":word(&k),"reverse":reverse,
            "before":pair(&before),"operator":pair(&operator),"inverse":pair(&inverse),
            "after":pair(&after),"recovered":pair(&recovered),"norm_carry":word(&carry),
            "before_positive":word(&before_positive),"before_negative":word(&before_negative),
            "positive":word(&self.positive),"negative":word(&self.negative),"readouts":readouts}));
        self.state=after;Ok(result)
    }
    pub fn multiplier(&self)->Word{
        // The source-ring coordinates act on the same 1+8Z paired orbit.
        self.state.a.add(&self.state.b).multiply(&small(8)).add(&small(1))
    }
    pub fn snapshot(&self)->Value{
        json!({"component":"dimension-indexed-quadratic-unit","dimension":self.dimension,
            "trace":word(&self.trace),"basis":"1,epsilon; epsilon^2-trace*epsilon+1=0",
            "state":pair(&self.state),"positive":word(&self.positive),"negative":word(&self.negative),
            "events":self.events})
    }
}
