//! MoDoT's actual cycle drives residual-frame arithmetic inside the resident.
use alloc::{string::{String,ToString},vec::Vec};
use super::mf;
#[path="../../MoDoT/ask_native/src/catalytic_cycle.rs"]
mod native;

#[derive(Clone,Debug)]
pub struct ResidualObservation{
    pub left:Vec<char>,pub right:Vec<char>,pub product:Vec<char>,
    pub residual:Vec<char>,pub common:Vec<char>,
}
#[derive(Clone,Debug)]
pub struct CatalystObservation{
    pub source:Vec<char>,pub shift:usize,
    pub resting:native::Tuple,pub spent:native::Tuple,pub returned:native::Tuple,
    pub substrate:native::Tuple,pub product:native::Tuple,
    pub direction:&'static str,pub imprint:&'static str,
    pub residuals:Vec<ResidualObservation>,
}
pub struct CatalystCycle{
    resting:native::Tuple,input:native::Tuple,maxima:[u8;12],
    axis_order:[usize;12],
    cursor:usize,
    pub observations:Vec<CatalystObservation>,
}
impl CatalystCycle{
    pub fn canonical()->Result<Self,String>{Self::from_record(include_str!("../data/modot_factor_cycle.json"))}
    pub fn from_record(text:&str)->Result<Self,String>{
        let record:serde_json::Value=serde_json::from_str(text).map_err(|e|e.to_string())?;
        if record["format"]!="modot-executed-cycle-v1"{return Err("unknown native catalyst program".into());}
        let tuple=|key:&str|serde_json::from_value::<native::Tuple>(record[key].clone()).map_err(|e|e.to_string());
        let resting=tuple("resting")?;let input=tuple("input")?;
        let maxima:[u8;12]=serde_json::from_value(record["ordinal_maxima"].clone()).map_err(|e|e.to_string())?;
        for t in [&resting,&input]{for (i,v) in t.iter().enumerate(){
            if v.is_none_or(|v|v>maxima[i]){return Err("incomplete or out-of-scale native cycle".into());}
        }}
        let (spent,product,direction,imprint)=native::working(&resting,&input,&maxima).map_err(String::from)?;
        let returned=native::restore(&spent,direction,&maxima).map_err(String::from)?;
        if spent!=tuple("spent")?||product!=tuple("product")?||returned!=resting
            ||returned!=tuple("returned")?||record["direction"]!=direction||record["imprint"].as_str()!=imprint{
            return Err("compiled catalyst differs from its executed MoDoT cycle".into());
        }
        // The executed catalyst product orders the twelve resident axes. This
        // permutation is the morphism that will be lowered onto factor-support
        // decision coordinates, rather than used only to rotate a probe cursor.
        let mut axis_order=core::array::from_fn(|axis|axis);
        axis_order.sort_by_key(|&axis|(product[axis].unwrap(),axis));
        Ok(Self{resting,input,maxima,axis_order,cursor:0,observations:Vec::new()})
    }
    pub fn axis_order(&self)->[usize;12]{self.axis_order}
    fn residual(n:&[char],left:Vec<char>,right:Vec<char>)->ResidualObservation{
        let product=mf::mul(&left,&right);
        let residual=if mf::cmp(n,&product).is_ge(){mf::sub(n,&product)}else{mf::sub(&product,n)};
        let common=mf::gcd(residual.clone(),n.to_vec());
        ResidualObservation{left,right,product,residual,common}
    }
    pub(super) fn advance(&mut self,n:&[char],support:Option<&crate::dialectic_reentry::FactorSupport>)
        ->Result<Option<(Vec<char>,Vec<char>)>,String>{
        if n.is_empty(){return Err("catalytic frame lost its source".into());}
        let (spent,product,direction,imprint)=native::working(&self.resting,&self.input,&self.maxima).map_err(String::from)?;
        let returned=native::restore(&spent,direction,&self.maxima).map_err(String::from)?;
        if returned!=self.resting{return Err("native catalyst failed regeneration".into());}
        let imprint=imprint.ok_or("numerical catalyst requires a structural imprint")?;
        let axis=match imprint{"R↔S"=>2,"T↔H"=>1,_=>return Err("unknown native conjugate imprint".into())};
        let polarity=if product[axis]==Some((self.input[axis].unwrap()+1)%(self.maxima[axis]+1)){1isize}else{-1};
        let winding=product[11].unwrap() as isize-self.input[11].unwrap() as isize;
        let shift=self.cursor;let mut left=Vec::new();let mut right=Vec::new();
        for at in 0..n.len(){let mark=n[(at+shift)%n.len()];if at%2==0{left.push(mark);}else{right.push(mark);}}
        let mut residuals=alloc::vec![Self::residual(n,mf::trim(left),mf::trim(right))];
        if let Some(support)=support{
            if support.source!=n{return Err("catalyst crossed its shared source boundary".into());}
            let lower=|bits:&[Option<char>]|mf::trim(bits.iter().map(|v|v.unwrap_or('⊤')).collect());
            residuals.push(Self::residual(n,lower(&support.p),lower(&support.q)));
        }
        self.cursor=(self.cursor as isize+winding*polarity).rem_euclid(n.len() as isize) as usize;
        let pair=residuals.iter().find_map(|r|{
            if mf::cmp(&r.common,&mf::one()).is_gt()&&mf::cmp(&r.common,n).is_lt(){
                let (q,remainder)=mf::divmod(n,&r.common);
                if mf::zero(&remainder){return Some((r.common.clone(),q));}
            }None
        });
        self.observations.push(CatalystObservation{source:n.to_vec(),shift,resting:self.resting,spent,returned,
            substrate:self.input,product,direction,imprint,residuals});
        Ok(pair)
    }
}
