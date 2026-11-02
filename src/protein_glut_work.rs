//! Source-bound Glut release coordinates and a conservative work ledger.
//! A cell starts at z=n. Releasing its second preference leaves p*z²/2;
//! measured decreases fund coordinate work without discarding source evidence.
use alloc::vec::Vec;

#[derive(Clone)]
pub struct Cell {
    pub source:[usize;4],pub multiplicity:u64,pub positive:usize,pub negative:usize,
    pub stored:f64,pub capacity:f64,pub spent:f64,
    pub returned_coordinate:Option<f64>,
}
impl Cell {
    pub fn new(source:[usize;4],multiplicity:u64,positive:usize,negative:usize)->Self{
        let (c,p,n)=(multiplicity as f64,positive as f64,negative as f64);
        Self{source,multiplicity,positive,negative,stored:c*p*n*(p+n)/2.0,
            capacity:c*p*n*n/2.0,spent:0.0,returned_coordinate:None}
    }
    pub fn coordinate(&self)->f64{
        if let Some(value)=self.returned_coordinate{return value;}
        if self.capacity==0.0{return self.negative as f64;}
        libm::sqrt(2.0*(self.capacity-self.spent)/(self.multiplicity as f64*self.positive as f64))
    }
    /// Complete the same two opposed quadratic preferences at the retained
    /// strain. The banked preference stays present as accessible work leaves.
    /// k*qStar² = c*n*(p*n+z²), where z is this cell's live release coordinate.
    pub fn preference(&self)->f64{
        let (p,n)=(self.positive as f64,self.negative as f64);
        let z=self.coordinate();
        libm::sqrt(n*(p*n+z*z)/(p+n))
    }
    /// Fixed-conformation minimizer of channel deformation plus live bank.
    pub fn conditional_preference(&self,distance:f64)->f64{
        let p=self.positive as f64;let n=self.negative as f64;
        (distance*n/(p+n)).max(libm::sqrt(p*n*n/(p+n))).min(self.preference())
    }
    pub fn available(&self)->f64{
        let z=self.coordinate();self.multiplicity as f64*self.positive as f64*z*z/2.0
    }
    pub fn remaining_stored(&self)->f64{self.stored-self.capacity+self.available()}
    pub fn debit(&mut self,work:f64){
        let after=libm::sqrt(2.0*(self.available()-work).max(0.0)/
            (self.multiplicity as f64*self.positive as f64));
        self.spent+=work;self.returned_coordinate=Some(after);
    }
}

#[derive(Clone)]
pub struct WorkBank { pub cells:Vec<Cell> }
impl WorkBank {
    pub fn available(&self)->f64{self.cells.iter().map(Cell::available).sum()}
    pub fn stored(&self)->f64{self.cells.iter().map(|c|c.stored).sum()}
    pub fn spent(&self)->f64{self.cells.iter().map(|c|c.spent).sum()}
    pub fn remaining_stored(&self)->f64{self.cells.iter().map(Cell::remaining_stored).sum()}
    /// Admission precedes mutation. Every debit retains its source and both
    /// release coordinates, so the actual released potential can be replayed.
    pub fn pay(&mut self,work:f64)->Option<serde_json::Value>{
        if !work.is_finite()||work<=0.0||work>self.available(){return None;}
        let mut need=work;
        let amounts:Vec<_>=self.cells.iter().map(|cell|{
            let debit=need.min(cell.available());need-=debit;debit
        }).collect();
        self.pay_allocated(&amounts)
    }
    /// The whole ensemble supplies these debits from its current joint
    /// potential. Validate the complete allocation before changing any cell.
    pub fn pay_allocated(&mut self,amounts:&[f64])->Option<serde_json::Value>{
        if amounts.len()!=self.cells.len()||amounts.iter().zip(&self.cells).any(|(&a,c)|
            !a.is_finite()||a<0.0||a>c.available()){return None;}
        let work=amounts.iter().sum::<f64>();
        if work<=0.0{return None;}
        let mut debits=Vec::new();let mut released=0.0;
        for (cell,&debit) in self.cells.iter_mut().zip(amounts){
            if debit==0.0{continue;}
            let before=cell.coordinate();
            cell.debit(debit);
            let after=cell.coordinate();
            let measured=cell.multiplicity as f64*cell.positive as f64*(before*before-after*after)/2.0;
            released+=measured;
            debits.push(serde_json::json!({"source":cell.source,"multiplicity":cell.multiplicity,
                "positive":cell.positive,"negative":cell.negative,
                "preference_before":libm::sqrt(cell.negative as f64*
                    (cell.positive as f64*cell.negative as f64+before*before)/
                    (cell.positive+cell.negative) as f64),
                "preference_after":cell.preference(),
                "release_coordinate_before":before,"release_coordinate_after":after,
                "charged_work":debit,"measured_released_potential":measured,
                "source_capacity":cell.capacity,"cumulative_work":cell.spent,
                "measured_cumulative_efficiency":cell.spent/cell.capacity}));
        }
        Some(serde_json::json!({"coordinate_work":work,"measured_released_potential":released,
            "signed_release_residual":released-work,"unallocated_work":0.0,"debits":debits}))
    }
    pub fn record(&self)->serde_json::Value{
        serde_json::json!({"schema":"native-glut-work-bank/v1",
            "release":"second preference banked; first potential p*z^2/2 acts from initial z=n",
            "initial_stored_energy":self.stored(),
            "initial_accessible_energy":self.cells.iter().map(|c|c.capacity).sum::<f64>(),
            "coordinate_work":self.spent(),"remaining_accessible_energy":self.available(),
            "remaining_stored_energy":self.remaining_stored(),
            "cells":self.cells.iter().map(|c|serde_json::json!({
                "source":c.source,"multiplicity":c.multiplicity,
                "positive":c.positive,"negative":c.negative,
                "initial_stored_energy":c.stored,"accessible_energy":c.capacity,
                "coordinate_work":c.spent,"release_coordinate":c.coordinate(),
                "live_preferred_distance":c.preference(),
                "banked_second_preference_energy":c.stored-c.capacity})).collect::<Vec<_>>()})
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn conditional_nest_is_idempotent_and_minimizes_coupled_energy(){
        for p in 1..=2{for n in 1..=2{for tick in 0..=32{
            let cell=Cell::new([0,1,1,2],3,p,n);let distance=tick as f64/4.0;
            let q=cell.conditional_preference(distance);
            let (p,n,c)=(p as f64,n as f64,3.0);
            let lower=libm::sqrt(p*n*n/(p+n));let upper=cell.preference();
            let energy=|x:f64|c*(p+n)*(distance-x)*(distance-x)/2.0
                +c*p*((p+n)*x*x/n-p*n)/2.0;
            assert!(energy(q)<=energy(upper)+1e-10);
            for trial in [lower,upper,(q-0.01).max(lower),(q+0.01).min(upper)]{
                assert!(energy(q)<=energy(trial)+1e-10);
            }
            let repeated=(distance*n/(p+n)).max(lower).min(q);
            assert_eq!(q,repeated);
        }}}
    }
    #[test]
    fn release_pays_for_actual_work_and_preserves_the_remaining_glut(){
        let mut bank=WorkBank{cells:alloc::vec![Cell::new([0,1,1,2],3,1,1),Cell::new([1,2,3,0],5,2,1)]};
        let capacity=bank.available();let stored=bank.stored();
        assert!(bank.pay(capacity+1.0).is_none());assert_eq!(bank.spent(),0.0);
        let work=capacity/2.0;let first=bank.pay(work).unwrap();
        assert_eq!(bank.spent(),work);
        assert!((bank.available()-(capacity-work)).abs()<f64::EPSILON*stored);
        assert!(first["signed_release_residual"].as_f64().unwrap().abs()<64.0*f64::EPSILON*stored);
        assert!(bank.cells.iter().all(|c|c.spent<=c.capacity&&c.capacity<=c.stored));
        assert_eq!(bank.stored()-bank.spent()+bank.spent(),stored);
    }
    #[test]
    fn payment_consumes_the_returned_coordinate_instead_of_recreating_the_initial_bank(){
        let mut cell=Cell::new([0,1,1,2],3,1,1);
        cell.returned_coordinate=Some(0.75);
        let mut bank=WorkBank{cells:alloc::vec![cell]};
        assert!(bank.pay(bank.cells[0].capacity).is_none());
        let before=bank.available();let payment=bank.pay(before/2.0).unwrap();
        let debit=&payment["debits"][0];
        assert_eq!(debit["release_coordinate_before"],0.75);
        assert!(debit["release_coordinate_after"].as_f64().unwrap()<0.75);
        assert!((bank.available()-before/2.0).abs()<f64::EPSILON*bank.stored());
    }
}
