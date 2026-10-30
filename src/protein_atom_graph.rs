//! Connected slot-pairing families from native atomic filling capacities.
//! These are structural candidates; electronic force determines their readout.
use alloc::{string::String,vec::Vec};

pub struct AtomFamily {
    pub charges:Vec<usize>,pub capacities:Vec<usize>,pub graphs:Vec<Vec<usize>>,
}
fn connected(matrix:&[usize],n:usize)->bool {
    if n==0{return false;}
    let mut seen=alloc::vec![false;n];seen[0]=true;
    for _ in 0..n {
        let mut changed=false;
        for i in 0..n {if seen[i] {for j in 0..n {
            if matrix[i*n+j]>0&&!seen[j] {seen[j]=true;changed=true;}
        }}}
        if !changed {break;}
    }
    seen.iter().all(|s|*s)
}
fn fill(n:usize,i:usize,j:usize,left:&mut [usize],matrix:&mut [usize],graphs:&mut Vec<Vec<usize>>)->Result<(),String> {
    if i==n {
        if left.iter().all(|x|*x==0)&&connected(matrix,n) {graphs.push(matrix.to_vec());}
        if graphs.len()>100_000 {return Err("slot-pairing return exceeds explicit graph boundary".into());}
        return Ok(());
    }
    if j==n {
        if left[i]==0 {return fill(n,i+1,i+2,left,matrix,graphs);}
        return Ok(());
    }
    for paired in 0..=left[i].min(left[j]) {
        left[i]-=paired;left[j]-=paired;
        matrix[i*n+j]=paired;matrix[j*n+i]=paired;
        fill(n,i,j+1,left,matrix,graphs)?;
        left[i]+=paired;left[j]+=paired;
    }
    matrix[i*n+j]=0;matrix[j*n+i]=0;
    Ok(())
}
impl AtomFamily {
    pub fn prepare(charges:&[usize])->Result<Self,String> {
        if charges.is_empty()||charges.len()>8 {return Err("explicit slot-pairing family requires one through eight atoms".into());}
        let mut capacities=Vec::new();
        for &z in charges {
            let atom=crate::protein_orbital::atomic_frontier(z)?;
            capacities.push(atom["available_single_capacity"].as_u64().ok_or("missing native single capacity")? as usize);
        }
        let n=charges.len();let mut graphs=Vec::new();
        if capacities.iter().sum::<usize>()%2==0 {
            fill(n,0,1,&mut capacities.clone(),&mut alloc::vec![0;n*n],&mut graphs)?;
        }
        Ok(Self{charges:charges.to_vec(),capacities,graphs})
    }
    pub fn record(&self)->serde_json::Value {
        serde_json::json!({"schema":"native-atomic-slot-graph-family/v1",
            "nuclear_sources":self.charges,"native_single_capacities":self.capacities,
            "candidate_graphs":self.graphs,
            "representation":"symmetric integer slot-pair counts, source-addressed atoms, zero diagonal",
            "selection":"all connected complete pairings held; no unique chemical bond graph asserted",
            "scope":"neutral paired frontier; ionic, incomplete and higher-winding branches remain separate source families"})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn small_molecule_graphs_come_from_native_slot_counts() {
        for (source,expected) in [(&[1,1][..],1),(&[8,1,1][..],1),(&[7,1,1,1][..],1),(&[6,1,1,1,1][..],1)] {
            let family=AtomFamily::prepare(source).unwrap();assert_eq!(family.graphs.len(),expected);
            for graph in family.graphs {
                let n=source.len();assert!(connected(&graph,n));
                for i in 0..n {assert_eq!(graph[i*n+i],0);assert_eq!(graph[i*n..(i+1)*n].iter().sum::<usize>(),family.capacities[i]);}
            }
        }
        let family=AtomFamily::prepare(&[6,8,7,1,1,1]).unwrap();let n=family.charges.len();
        assert!(family.graphs.len()>1);
        let mut amide=alloc::vec![0;n*n];
        for (i,j,paired) in [(0,1,2),(0,2,1),(0,3,1),(2,4,1),(2,5,1)] {
            amide[i*n+j]=paired;amide[j*n+i]=paired;
        }
        assert!(family.graphs.contains(&amide));
        for graph in &family.graphs {for i in 0..n {
            assert_eq!(graph[i*n..(i+1)*n].iter().sum::<usize>(),family.capacities[i]);
        }}
        assert!(AtomFamily::prepare(&[1,1,1]).unwrap().graphs.is_empty());
    }
}
