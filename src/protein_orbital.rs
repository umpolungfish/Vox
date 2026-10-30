//! Correlated two-orbital transfer on the native powerset carrier.
//! FOUR's two occupancy slots supply the capacities. Every admitted rewrite
//! preserves both spin counts; the returned joint family keeps the correlation.
use crate::belnap::B4;
use alloc::{string::String,vec::Vec};

/// A physical occupancy addition has a resource boundary: filling a spin
/// already present has no successor. The idempotent information join remains
/// a separate operation and does not create another electron.
pub fn create(occupancy:u8,spin:usize)->Option<u8>{
    if occupancy>=4||spin>=2{return None;}
    let bit=1u8<<spin;
    if occupancy&bit!=0{None}else{Some(occupancy|bit)}
}

/// An edge is one creation on the retained orbital tuple. The phase is the
/// exterior-basis sign of that creation; it keeps the order of occupations.
pub struct FillingEdge {pub from:usize,pub to:usize,pub slot:usize,pub phase:i8}
pub struct FillingLattice {
    pub orbitals:usize,
    pub levels:Vec<Vec<usize>>,
    pub edges:Vec<FillingEdge>,
    pub histories:Vec<u64>,
}
impl FillingLattice {
    pub fn prepare(orbitals:usize)->Result<Self,String>{
        if orbitals>4{return Err("explicit native filling is bounded to FOUR indexed orbitals".into());}
        let slots=2*orbitals;let count=1usize<<slots;
        let mut levels=alloc::vec![Vec::new();slots+1];let mut edges=Vec::new();
        let mut histories=alloc::vec![0u64;count];histories[0]=1;
        for state in 0..count{
            levels[state.count_ones() as usize].push(state);
            for slot in 0..slots{
                let orbital=slot/2;let spin=slot%2;let occupancy=((state>>(2*orbital))&3) as u8;
                if let Some(next)=create(occupancy,spin){
                    let target=(state&!(3<<(2*orbital)))|((next as usize)<<(2*orbital));
                    let phase=if (state&((1<<slot)-1)).count_ones()%2==0{1}else{-1};
                    histories[target]+=histories[state];
                    edges.push(FillingEdge{from:state,to:target,slot,phase});
                }
            }
        }
        Ok(Self{orbitals,levels,edges,histories})
    }
    pub fn placement(&self,state:usize)->Vec<u8>{
        (0..self.orbitals).map(|i|((state>>(2*i))&3) as u8).collect()
    }
    pub fn sector_states(&self,up:usize,down:usize)->Vec<usize>{
        if up+down>=self.levels.len(){return Vec::new();}
        self.levels[up+down].iter().copied().filter(|&state|{
            let p=shell_profile(&self.placement(state));p.up==up&&p.down==down
        }).collect()
    }
    pub fn record(&self)->serde_json::Value{
        let mechanical:Vec<_>=self.levels.iter().map(|level|{
            let minimum=level.iter().map(|&s|shell_glut(&self.placement(s))).min().unwrap_or(0);
            serde_json::json!({"minimum_stored_glut":minimum,
                "minimum_readouts":level.iter().copied().filter(|&s|
                    shell_glut(&self.placement(s))==minimum).collect::<Vec<_>>()})
        }).collect();
        serde_json::json!({"orbitals":self.orbitals,"spin_slots":2*self.orbitals,
            "levels":self.levels,"creation_history_counts":self.histories,
            "edges":self.edges.iter().map(|e|serde_json::json!({
                "from":e.from,"to":e.to,"slot":e.slot,"exterior_phase":e.phase})).collect::<Vec<_>>(),
            "mechanical_readout":mechanical,
            "mechanical_rule":"sum native Glut residues on each addressed orbital and orbital pair",
            "rule":"add one vacant spin slot; occupied slot has no successor",
            "state_role":"all allowed filling paths retained; no scalar ranking selects a state"})
    }
}

fn sector(left:u8,right:u8)->(u8,u8){
    ((left&1)+(right&1),((left>>1)&1)+((right>>1)&1))
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Transfer {
    pub source:u16,
    pub family:u16,
    pub marginal:u8,
    pub selected:u8,
}
impl Transfer {
    pub fn prepare(left:u8,right:u8)->Self{
        let mut source=0u16;
        for a in 0..4{for b in 0..4{
            if left&(1<<a)!=0&&right&(1<<b)!=0{source|=1<<(4*a+b);}
        }}
        let family=close(source);
        let mut marginal=0;
        for a in 0..4{for b in 0..4{
            if family&(1<<(4*a+b))!=0{marginal|=(1<<a)|(1<<b);}
        }}
        Self{source,family,marginal,selected:left|right|marginal}
    }
}

pub fn close(source:u16)->u16{
    let filling=FillingLattice::prepare(2).expect("two-orbital native filling");
    let mut family=0;
    for a in 0..4{for b in 0..4{
        if source&(1<<(4*a+b))==0{continue;}
        let counts=sector(a,b);
        // The source depairs to its two spin counts. Filling the same
        // retained sector reconstructs every possible native placement.
        for state in filling.sector_states(counts.0 as usize,counts.1 as usize){
            let placement=filling.placement(state);let (c,d)=(placement[0],placement[1]);
            debug_assert_eq!(sector(c,d),counts);
            family|=1<<(4*c+d);
        }
    }}family
}

/// Per supported FOUR occupancy, each spin slot is occupied or vacant.
pub fn slot_counts(state:u8)->(usize,usize,usize,usize){
    let has=|v:B4|usize::from(state&(1<<(v as u8))!=0);
    (has(B4::T)+has(B4::B),has(B4::F)+has(B4::B),
     has(B4::N)+has(B4::F),has(B4::N)+has(B4::T))
}

pub fn table()->[Transfer;256]{
    core::array::from_fn(|i|Transfer::prepare((i/16) as u8,(i%16) as u8))
}

/// One correlated shell placement. Each indexed orbital retains its own
/// up/down slots; a marginal union never substitutes for this joint state.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ShellProfile {
    pub up:usize,pub down:usize,pub paired:usize,pub empty:usize,pub single:usize,
}
pub fn shell_profile(orbitals:&[u8])->ShellProfile{
    let mut p=ShellProfile{up:0,down:0,paired:0,empty:0,single:0};
    for &v in orbitals{
        assert!(v<4,"shell placement requires a FOUR occupancy, not a support mask");
        p.up+=(v&1) as usize;p.down+=((v>>1)&1) as usize;
        p.paired+=usize::from(v==3);p.empty+=usize::from(v==0);
        p.single+=usize::from(v==1||v==2);
    }p
}

/// The existing Glut residue applied to conserved, indexed spin counts.
/// These are native energy units. No measured atomic energy is supplied.
fn glut(up:usize,down:usize)->u64{(up*down*(up+down)/2) as u64}

/// The cross term from fusing two indexed native shells. Each cross-shell
/// orbital pair occurs once; source placements remain independently carried.
pub fn shell_cross_profiles(left_len:usize,left:ShellProfile,right_len:usize,right:ShellProfile)->u64{
    let (u,v,p)=(left.up as u128,left.down as u128,left.paired as u128);
    let (r,s,q)=(right.up as u128,right.down as u128,right.paired as u128);
    let value=u*s+r*v+p*(right_len as u128+r+s)+q*(left_len as u128+u+v);
    u64::try_from(value).expect("native shell Glut cross term exceeds the retained integer readout")
}
pub fn shell_cross_glut(left:&[u8],right:&[u8])->u64{
    shell_cross_profiles(left.len(),shell_profile(left),right.len(),shell_profile(right))
}
pub fn shell_glut(orbitals:&[u8])->u64{
    let mut energy=0u64;
    let mut held=ShellProfile{up:0,down:0,paired:0,empty:0,single:0};
    for (length,&state) in orbitals.iter().enumerate(){
        let next=shell_profile(core::slice::from_ref(&state));
        energy=energy.checked_add(glut(next.up,next.down))
            .and_then(|v|v.checked_add(shell_cross_profiles(length,held,1,next)))
            .expect("native shell Glut exceeds the retained integer readout");
        held.up+=next.up;held.down+=next.down;held.paired+=next.paired;
        held.empty+=next.empty;held.single+=next.single;
    }energy
}

/// Spatial modes are the monomials of the even symmetric spinor degree.
/// Their two exponents retain the two branch histories and sum to 2*l.
pub fn winding_modes(l:usize)->Vec<(usize,usize)>{
    (0..=2*l).map(|a|(a,2*l-a)).collect()
}

/// A nuclear source supplies the integer electron budget of a neutral atom.
/// The constructor supplies capacities and the retained filling families;
/// it imports no element-specific configuration or valence table.
///
/// The ordering below is the explicit winding-degree filtration (n+l,n).
/// It is recorded as a construction boundary, not asserted to establish
/// a unique physical energy ordering from the local Frobenius law alone.
pub fn atomic_frontier(charge:usize)->Result<serde_json::Value,String>{
    if charge==0||charge>18{return Err("current atomic frontier covers nuclear sources 1 through 18".into());}
    let mut remaining=charge;let mut blocks=Vec::new();let mut outer_n=0;
    let mut outer_electrons=0;let mut outer_orbitals=0;
    'source: for degree in 1..{
        for n in 1..=degree{
            let l=degree-n;if l>=n{continue;}
            let modes=winding_modes(l);let filling=FillingLattice::prepare(modes.len())?;
            let used=remaining.min(2*modes.len());
            if n>outer_n{outer_n=n;outer_electrons=0;outer_orbitals=0;}
            if n==outer_n{outer_electrons+=used;outer_orbitals+=modes.len();}
            let states=&filling.levels[used];
            let minimum=states.iter().map(|&s|shell_glut(&filling.placement(s))).min().unwrap();
            blocks.push(serde_json::json!({"principal_winding":n,"symmetric_degree":2*l,
                "poloidal_winding":l,"winding_grade":degree,"spinor_monomials":modes,
                "orbitals":modes.len(),"spin_capacity":2*modes.len(),"electrons":used,
                "correlated_frontier":states,"minimum_stored_glut":minimum,
                "minimum_readouts":states.iter().copied().filter(|&s|
                    shell_glut(&filling.placement(s))==minimum).collect::<Vec<_>>(),
                "ordered_creation_histories_per_state":filling.histories[states[0]]}));
            remaining-=used;if remaining==0{break 'source;}
        }
    }
    let outer=FillingLattice::prepare(outer_orbitals)?;
    Ok(serde_json::json!({"schema":"native-atomic-filling/v1","source_nuclear_charge":charge,
        "source_electrons":charge,"external_numeric_parameters":[],"blocks":blocks,
        "ordering_boundary":"winding-degree filtration (n+l,n); physical cross-block energy ordering remains to check",
        "outer_principal_winding":outer_n,"outer_electrons":outer_electrons,
        "outer_active_orbitals":outer_orbitals,
        "outer_joint_frontier":outer.levels[outer_electrons],
        "available_single_capacity":outer_electrons.min(2*outer_orbitals-outer_electrons),
        "continuation":"retain cross-block and unoccupied-mode alternatives for bonding; no unique atomic conformation selected"}))
}

/// All placements reached by independent native exchanges of the up/down
/// components. This keeps the complete family, including every different
/// number of pairs. `single_capacity` is a readout, never a selection rule.
pub struct ShellFamily {
    pub source:alloc::vec::Vec<u8>,
    pub placements:alloc::vec::Vec<alloc::vec::Vec<u8>>,
    pub single_capacity:usize,
}
impl ShellFamily {
    pub fn prepare(source:&[u8])->Result<Self,alloc::string::String>{
        if source.len()>4{return Err("bounded native shell supports at most FOUR indexed orbitals".into());}
        if source.iter().any(|&x|x>=4){return Err("native shell source must contain FOUR occupancies".into());}
        let p=shell_profile(source);let filling=FillingLattice::prepare(source.len())?;
        let placements:Vec<_>=filling.sector_states(p.up,p.down).into_iter()
            .map(|s|filling.placement(s)).collect();
        let single_capacity=placements.iter().map(|s|shell_profile(s).single).max().unwrap_or(0);
        Ok(Self{source:source.to_vec(),placements,single_capacity})
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    use alloc::string::ToString;
    #[test]
    fn shell_fusion_matches_every_indexed_pair_and_retains_grouping(){
        let literal=|shell:&[u8]|{
            let mut result=0;
            for (i,&a) in shell.iter().enumerate(){
                result+=glut((a&1) as usize,(a>>1) as usize);
                for &b in &shell[i+1..]{let (u,d)=sector(a,b);result+=glut(u as usize,d as usize);}
            }result
        };
        for count in 0..=4{
            for source in 0..1usize<<(2*count){
                let shell:Vec<_>=(0..count).map(|i|((source>>(2*i))&3) as u8).collect();
                assert_eq!(shell_glut(&shell),literal(&shell));
                for cut in 0..=count{
                    let (a,b)=shell.split_at(cut);
                    assert_eq!(shell_glut(&shell),shell_glut(a)+shell_glut(b)+shell_cross_glut(a,b));
                }
            }
        }
        let shell:Vec<_>=(0..200usize).map(|i|((i*i+3*i+7)%4) as u8).collect();
        assert_eq!(shell_glut(&shell),literal(&shell));
        for cut in [0,1,17,99,200]{let (a,b)=shell.split_at(cut);
            assert_eq!(shell_glut(&shell),shell_glut(a)+shell_glut(b)+shell_cross_glut(a,b));
        }
        let (a,rest)=shell.split_at(17);let (b,c)=rest.split_at(82);
        let ab:Vec<_>=a.iter().chain(b).copied().collect();let bc:Vec<_>=b.iter().chain(c).copied().collect();
        assert_eq!(shell_cross_glut(a,b)+shell_cross_glut(&ab,c),
            shell_cross_glut(b,c)+shell_cross_glut(a,&bc));
    }
    #[test]
    fn atomic_frontiers_derive_counts_from_modes_and_retain_every_active_placement(){
        for charge in 1..=18{
            let atom=atomic_frontier(charge).unwrap();let blocks=atom["blocks"].as_array().unwrap();
            assert_eq!(blocks.iter().map(|b|b["electrons"].as_u64().unwrap()).sum::<u64>(),charge as u64);
            let mut prior=(0,0);
            for block in blocks{
                let n=block["principal_winding"].as_u64().unwrap() as usize;
                let l=block["poloidal_winding"].as_u64().unwrap() as usize;
                assert!(l<n);let key=(n+l,n);assert!(key>prior);prior=key;
                let modes=block["spinor_monomials"].as_array().unwrap();
                assert_eq!(modes.len(),2*l+1);
                for mode in modes{assert_eq!(mode[0].as_u64().unwrap()+mode[1].as_u64().unwrap(),2*l as u64);}
                let used=block["electrons"].as_u64().unwrap() as usize;
                assert!(used<=2*modes.len());
                for state in block["correlated_frontier"].as_array().unwrap(){
                    assert_eq!(state.as_u64().unwrap().count_ones() as usize,used);
                }
            }
            let used=atom["outer_electrons"].as_u64().unwrap() as usize;
            for state in atom["outer_joint_frontier"].as_array().unwrap(){
                assert_eq!(state.as_u64().unwrap().count_ones() as usize,used);
            }
        }
        // These are source-nucleus controls, not valence inputs to the builder.
        for (charge,single) in [(1,1),(6,4),(7,3),(8,2),(16,2)]{
            assert_eq!(atomic_frontier(charge).unwrap()["available_single_capacity"],single);
        }
    }
    #[test]
    fn filling_covers_are_conservative_and_retain_ordered_creation_histories(){
        for count in 0..=4{
            let filling=FillingLattice::prepare(count).unwrap();let slots=2*count;
            let mut factorial=1u64;
            for (rank,states) in filling.levels.iter().enumerate(){
                if rank>0{factorial*=rank as u64;}
                for &state in states{
                    assert_eq!(state.count_ones() as usize,rank);
                    assert_eq!(filling.histories[state],factorial);
                    assert_eq!(shell_profile(&filling.placement(state)).up+
                        shell_profile(&filling.placement(state)).down,rank);
                }
                let minimum=states.iter().map(|&s|shell_glut(&filling.placement(s))).min().unwrap();
                let expected=if rank<=count{0}else{(rank-count)*(2*count+rank-3)} as u64;
                assert_eq!(minimum,expected);
                for &state in states{
                    let placement=filling.placement(state);let p=shell_profile(&placement);
                    if shell_glut(&placement)==minimum{
                        assert_eq!(p.single,rank.min(2*count-rank));
                        assert!(p.up==0||p.down==0||p.up==count||p.down==count);
                    }
                }
            }
            for e in &filling.edges{
                assert_eq!(e.from&!e.to,0);assert_eq!(e.to^e.from,1<<e.slot);
                assert_eq!(e.to.count_ones(),e.from.count_ones()+1);
                assert_ne!(e.phase,0);
            }
            // The two creation orders have opposite exterior signs.
            for state in 0..1usize<<slots{for i in 0..slots{for j in i+1..slots{
                if state&((1<<i)|(1<<j))!=0{continue;}
                let sign=|s:usize,slot:usize|if (s&((1<<slot)-1)).count_ones()%2==0{1}else{-1};
                assert_eq!(sign(state,i)*sign(state|(1<<i),j),
                    -sign(state,j)*sign(state|(1<<j),i));
            }}}
        }
        assert_eq!(create(3,0),None);assert_eq!(create(3,1),None);
        assert_eq!(create(1,0),None);assert_eq!(create(2,1),None);
        assert_eq!(create(1,1),Some(3));assert_eq!(create(2,0),Some(3));
    }
    #[test]
    fn native_shell_family_retains_every_spin_conserving_placement(){
        for count in 0..=4{
            for code in 0..1usize<<(2*count){
                let source:alloc::vec::Vec<_>=(0..count).map(|i|((code>>(2*i))&3) as u8).collect();
                let p=shell_profile(&source);let family=ShellFamily::prepare(&source).unwrap();
                assert!(family.placements.contains(&source));
                let occupied=p.up+p.down;let vacant=2*count-occupied;
                assert_eq!(p.single+2*p.paired,occupied);
                assert_eq!(p.single+2*p.empty,vacant);
                assert_eq!(family.single_capacity,occupied.min(vacant));
                let expected:alloc::vec::Vec<_>=(0..1usize<<(2*count)).map(|c|
                    (0..count).map(|i|((c>>(2*i))&3) as u8).collect::<alloc::vec::Vec<_>>())
                    .filter(|s|{let q=shell_profile(s);(q.up,q.down)==(p.up,p.down)}).collect();
                assert_eq!(family.placements.len(),expected.len());
                assert!(expected.iter().all(|s|family.placements.contains(s)));
                // Every single native two-site move stays inside the same
                // correlated shell family; no shared orbital is duplicated.
                for s in &family.placements{for i in 0..count{for j in i+1..count{
                    let alternatives=close(1<<(4*s[i]+s[j]));
                    for a in 0..4{for b in 0..4{
                        if alternatives&(1<<(4*a+b))==0{continue;}
                        let mut next=s.clone();next[i]=a;next[j]=b;
                        assert!(family.placements.contains(&next));
                    }}
                }}}
            }
        }
        let four=ShellFamily::prepare(&[3,1,1,0]).unwrap();
        assert_eq!(four.single_capacity,4);
        // Three up components and one down component cannot all pair:
        // the retained spin imbalance leaves at least two singles.
        assert!(four.placements.iter().any(|s|shell_profile(s).single==2));
        assert!(four.placements.iter().all(|s|shell_profile(s).single>=2));
        assert!(four.placements.iter().any(|s|shell_profile(s).single==4));
        assert_eq!(ShellFamily::prepare(&[3,1,1,1]).unwrap().single_capacity,3);
        assert_eq!(ShellFamily::prepare(&[3,3,1,1]).unwrap().single_capacity,2);
        assert_eq!(ShellFamily::prepare(&[3,3,3,1]).unwrap().single_capacity,1);
        assert_eq!(ShellFamily::prepare(&[3,3,3,3]).unwrap().single_capacity,0);
    }
    #[test]
    fn retained_transfers_match_live_lean_for_every_source_family(){
        let export:serde_json::Value=serde_json::from_str(include_str!(
            "../tests/fixtures/native_orbital_transfer_20261028.json")).unwrap();
        for (i,t) in table().iter().enumerate(){
            let expected=&export["transfers"][i.to_string()];
            assert_eq!(t.source as u64,expected[0].as_u64().unwrap());
            assert_eq!(t.family as u64,expected[1].as_u64().unwrap());
            assert_eq!(t.selected as u64,expected[2].as_u64().unwrap());
            assert_eq!(t.source&!t.family,0);assert_eq!(close(t.family),t.family);
        }
    }
    #[test]
    fn opposed_spin_transfers_keep_the_joint_conservation_sector(){
        let t=Transfer::prepare(1<<B4::T as u8,1<<B4::F as u8);
        assert_eq!(t.family.count_ones(),4);
        assert_eq!(t.selected,15);
        assert_eq!(t.family&(1<<(4*B4::B as u8+B4::B as u8)),0);
        for state in 0..16u8{
            let (u,d,vu,vd)=slot_counts(state);
            assert_eq!(u+d+vu+vd,2*state.count_ones() as usize);
        }
    }
}
