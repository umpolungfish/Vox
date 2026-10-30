//! Native-unit Glut realization. Inputs are the retained residue/form words.
//! All stiffnesses and preferred separations come from SIXTEEN_3 memberships;
//! geometry comes from FOUR's three nontrivial characters. No physical tables.
use alloc::{string::{String,ToString}, vec, vec::Vec};
use crate::{belnap::B4, belnap_relation::BelnapSet, phase_word,
    protein_lattice::{Response, Spectrum}, protein_sites::{ProgramSpec, SiteCarrier},
    protein_orbital::{self, Transfer}, protein_site_geometry::Frames,
    protein_glut_work::{Cell,WorkBank}};

use crate::protein_records::{self,History};

const FOUR: [B4; 4] = [B4::N, B4::T, B4::F, B4::B];
type Point = (f64,f64,f64);
fn add(a:Point,b:Point)->Point{(a.0+b.0,a.1+b.1,a.2+b.2)}
fn sub(a:Point,b:Point)->Point{(a.0-b.0,a.1-b.1,a.2-b.2)}
fn scale(a:Point,s:f64)->Point{(a.0*s,a.1*s,a.2*s)}
fn dot(a:Point,b:Point)->f64{a.0*b.0+a.1*b.1+a.2*b.2}
fn cross(a:Point,b:Point)->Point{(a.1*b.2-a.2*b.1,a.2*b.0-a.0*b.2,a.0*b.1-a.1*b.0)}
fn norm(a:Point)->f64{libm::sqrt(dot(a,a))}
fn unit(a:Point)->Point{scale(a,1.0/norm(a))}

/// The character frame, with Gram diagonal 3 and off-diagonal -1.
pub fn vertex(value:B4)->Point{
    let (t,f)=value.to_wh2();
    let t=if t{-1.0}else{1.0};let f=if f{-1.0}else{1.0};(t,f,t*f)
}
fn route_support_names(mask:u8)->Vec<&'static str>{
    FOUR.into_iter().filter(|&value|mask&(1u8<<value.to_u8())!=0).map(B4::name).collect()
}

#[derive(Clone,Copy,Debug)]
pub struct NativeGlut { pub positive:usize, pub negative:usize }
impl NativeGlut {
    pub fn from_state(state:BelnapSet)->Self{
        let (p,n,_,_)=protein_orbital::slot_counts(state.mask());
        Self{positive:p,negative:n}
    }
    pub fn response(self,q:f64)->Response{
        let p=self.positive as f64;let n=self.negative as f64;let span=p+n;
        Response{energy:(p*q*q+n*(q-span)*(q-span))/2.0,
            radial_derivative:p*q+n*(q-span)}
    }
    pub fn retained_strain(self)->f64{
        let p=self.positive as f64;let n=self.negative as f64;p*n*(p+n)/2.0
    }
}

/// Compute all three orders from their native positive membership sets.
pub fn pair_contexts(left:u8,right:u8)->[u8;6]{
    let t=BelnapSet::singleton(B4::T).union(BelnapSet::singleton(B4::B)).mask();
    let f=BelnapSet::singleton(B4::F).union(BelnapSet::singleton(B4::B)).mask();
    let join=left|right;let meet=left&right;let all=BelnapSet::COMPLETE.mask();
    [join,meet,(join&t)|(meet&(all^t)),(meet&t)|(join&(all^t)),
        (join&f)|(meet&(all^f)),(meet&f)|(join&(all^f))]
}

/// Inclusion is the information order. Equal maxima keep every source context;
/// membership count or mechanical energy never ranks the candidates.
pub fn information_maximum(contexts:&[u8])->u8{
    contexts.iter().fold(0,|held,&offered|held|offered)
}

#[derive(Clone)]
struct Channel {
    left:usize,right:usize,count:u64,positive:usize,negative:usize,
    stiffness:f64,preferred:f64,strain:f64,
}
#[derive(Clone)]
struct Pair {
    left:usize,right:usize,counts:[[u64;16];6],cycles:Vec<(usize,usize,usize,usize)>,
    selected_counts:[u64;16],maximizers:[u64;6],information_state:u8,
    source_pair_counts:Vec<u64>,orbital_maximizers:u64,orbital_expansions:u64,
    channels:Vec<Channel>,
    information_gains:Vec<(usize,usize,usize,usize,u8,u8,u8,u8,u8)>,
    stiffness:f64,preferred:f64,strain:f64,
}
/// T/F support from the configured GB1 route, joined at residue-pair address.
/// Only support enters native preparation; configured coefficients stay out.
#[derive(Clone,Debug)]
pub struct RouteBranch {
    pub source:String,
    pub glut_sha256:String,
    pub pdb_sha256:String,
    pub residue_pairs:Vec<(usize,usize,u8)>,
}
fn gcd(mut a:usize,mut b:usize)->usize{while b!=0{let r=a%b;a=b;b=r;}a}
#[cfg(test)]
fn prepare_pair(i:usize,j:usize,paths:&[Vec<Vec<Vec<u8>>>],sites:&SiteCarrier,transfers:&[Transfer;256])->Pair{
    prepare_pair_with_branch(i,j,paths,sites,transfers,None)
}
fn prepare_pair_with_branch(i:usize,j:usize,paths:&[Vec<Vec<Vec<u8>>>],sites:&SiteCarrier,
    transfers:&[Transfer;256],branch:Option<&RouteBranch>)->Pair{
    let mut counts=[[0u64;16];6];let mut cycles=Vec::new();
    let mut selected_counts=[0u64;16];let mut maximizers=[0u64;6];
    let mut information_state=0u8;let mut information_gains=Vec::new();
    let mut source_pair_counts=vec![0;256];let mut source_families:Vec<u16>=transfers.iter().map(|t|t.family).collect();
    let mut orbital_maximizers=0;let mut orbital_expansions=0;
    for (a,left_form) in paths[i].iter().enumerate(){for (b,right_form) in paths[j].iter().enumerate(){
      for (part,(left,right)) in left_form.iter().zip(right_form).enumerate(){
        let period=left.len()/gcd(left.len(),right.len())*right.len();
        cycles.push((a,b,part,period));
        for tick in 0..period{
          let lh=sites.program(i,a,part).histogram(left[(tick+i)%left.len()]);
          let rh=sites.program(j,b,part).histogram(right[(tick+j)%right.len()]);
          for (x,&lc) in lh.iter().enumerate(){if lc==0{continue;}
           for (y,&rc) in rh.iter().enumerate(){if rc==0{continue;}
            // Exact contraction of addressed edge pairs by their register value.
            // The edge arrays and binding tree retain every source port.
            let multiplicity=lc*rc;
            let index=16*x+y;source_pair_counts[index]+=multiplicity;
            let transfer=transfers[index];
            let states=pair_contexts(x as u8,y as u8);
            for (order,state) in states.into_iter().enumerate(){counts[order][state as usize]+=multiplicity;}
            let branch_support=branch.and_then(|route|route.residue_pairs.iter()
                .find(|&&(left,right,_)|left==i&&right==j).map(|&(_,_,mask)|mask)).unwrap_or(0);
            let maximum=transfer.selected|branch_support;
            if branch_support!=0{
                source_families[index]|=Transfer::prepare(maximum,maximum).family;
            }
            if transfer.selected!=information_maximum(&states){orbital_expansions+=multiplicity;}
            if transfer.marginal==maximum{orbital_maximizers+=multiplicity;}
            // The transfer family adds correlated, spin-conserving placements.
            // The six source contexts remain carried separately from that lift.
            selected_counts[maximum as usize]+=multiplicity;
            for (order,&state) in states.iter().enumerate(){
                if state==maximum{maximizers[order]+=multiplicity;}
            }
            let next=information_state|maximum;
            if next!=information_state{
                information_gains.push((a,b,part,tick,x as u8,y as u8,information_state,maximum,next));
                information_state=next;
            }
           }
          }
        }
    }}}
    let(mut stiffness,mut first,mut second)=(0.0,0.0,0.0);
    for (mask,&count) in selected_counts.iter().enumerate(){
        let g=NativeGlut::from_state(BelnapSet::from_mask(mask as u8).unwrap());
        let p=g.positive as f64;let n=g.negative as f64;let span=p+n;let count=count as f64;
        stiffness+=count*span;first+=count*n*span;second+=count*n*span*span;
    }
    let preferred=if stiffness==0.0{0.0}else{first/stiffness};
    let strain=if stiffness==0.0{0.0}else{(second-first*preferred)/2.0};
    let mut branch_counts=[0u64;16];
    for (index,&count) in source_pair_counts.iter().enumerate(){
        for (member,branch_count) in branch_counts.iter_mut().enumerate(){
        if source_families[index]&(1<<member)!=0{*branch_count+=count;}
        }
    }
    let channels=branch_counts.iter().enumerate().filter_map(|(member,&count)|{
        if count==0{return None;}
        let (a,b)=(member/4,member%4);
        let positive=(a&1)+(b&1);let negative=(a>>1)+(b>>1);
        let cross=protein_orbital::shell_cross_glut(&[a as u8],&[b as u8]);
        Some(Channel{left:a,right:b,count,positive,negative,
            stiffness:count as f64*(positive+negative) as f64,
            preferred:negative as f64,strain:count as f64*cross as f64})
    }).collect();
    Pair{left:i,right:j,counts,cycles,selected_counts,maximizers,information_state,
        source_pair_counts,orbital_maximizers,orbital_expansions,
        channels,information_gains,stiffness,preferred,strain}
}

fn initial_torsion(paths:&[Vec<Vec<u8>>])->f64{
    let mut carried=(0.0,0.0,0.0);
    let mut clock=0usize;
    for form in paths{for path in form{for &state in path{
        clock+=1;
        for v in FOUR{if state&(1<<(v as u8))!=0{carried=add(carried,scale(vertex(v),clock as f64));}}
    }}}
    libm::atan2(carried.1,carried.0)
}
fn backbone(torsions:&[f64])->Vec<Point>{
    let mut points=Vec::with_capacity(torsions.len());
    // The unit bond is NativeGlut({B}).qStar; the bend is the FOUR Gram ratio.
    let bond=NativeGlut::from_state(BelnapSet::singleton(B4::B)).negative as f64;
    let cosine=dot(vertex(B4::N),vertex(B4::T))/dot(vertex(B4::N),vertex(B4::N));
    let sine=libm::sqrt(1.0-cosine*cosine);
    for i in 0..torsions.len(){
        let next=match i{
            0=>(0.0,0.0,0.0),1=>(bond,0.0,0.0),
            2=>(bond-bond*cosine,bond*sine,0.0),
            _=>{
                let (a,b,c)=(points[i-3],points[i-2],points[i-1]);
                let e=unit(sub(c,b));let normal=unit(cross(sub(b,a),e));let tangent=cross(normal,e);
                add(c,scale(add(scale(e,-cosine),scale(add(scale(tangent,libm::cos(torsions[i])),
                    scale(normal,libm::sin(torsions[i]))),sine)),bond))
            }
        };points.push(next);
    }points
}
fn evaluate<const WITH_GRADIENT:bool>(points:&[Point],pairs:&[Pair])->(f64,Vec<Point>){
    let frames=Frames::new(points);
    let mut energy=0.0;
    let mut gradient=if WITH_GRADIENT{vec![[(0.0,0.0,0.0);4];points.len()]}else{Vec::new()};
    for pair in pairs{
        let (i,j)=(pair.left,pair.right);
        for channel in &pair.channels{
            if channel.stiffness==0.0{continue;}
            let (a,b)=(channel.left,channel.right);
            let delta=sub(frames.sites[i][a],frames.sites[j][b]);let distance=norm(delta);
            let error=distance-channel.preferred;
            energy+=channel.stiffness*error*error/2.0;
            if WITH_GRADIENT{
                let direction=if distance>0.0{scale(delta,1.0/distance)}else{
                    unit(vertex(FOUR[(i^j^a^b)%FOUR.len()]))};
                let g=scale(direction,channel.stiffness*error);
                gradient[i][a]=add(gradient[i][a],g);gradient[j][b]=sub(gradient[j][b],g);
            }
        }
    }
    (energy,if WITH_GRADIENT{frames.pullback(points,&gradient)}else{Vec::new()})
}
fn energy_gradient(points:&[Point],pairs:&[Pair])->(f64,Vec<Point>){
    evaluate::<true>(points,pairs)
}
fn energy(points:&[Point],pairs:&[Pair])->f64{
    evaluate::<false>(points,pairs).0
}
/// Pull the retained internal release state back into the whole interaction
/// readout. Source contexts and correlated placements remain untouched.
fn reenter_preferences(pairs:&mut [Pair],bank:&WorkBank)->usize{
    let mut changed=0;
    for cell in &bank.cells{
        let pair=&mut pairs.iter_mut().find(|p|
            [p.left,p.right]==cell.source[..2]).expect("retained release pair");
        let channel=pair.channels.iter_mut().find(|c|
            [c.left,c.right]==cell.source[2..]).expect("retained release channel");
        let next=cell.preference();
        changed+=usize::from(next!=channel.preferred);
        channel.preferred=next;
        channel.strain=cell.remaining_stored();
    }
    changed
}

struct EnsembleSic {
    frame:crate::sic::frame::SicFrame,
    returns:Vec<String>,
    signed_energy_drift:f64,
    enabled:bool,
}
impl EnsembleSic {
    fn new(enabled:bool)->Self{
        Self{frame:crate::sic::wh::ququart_frame(),returns:Vec::new(),signed_energy_drift:0.0,enabled}
    }
    /// Pair the actual scalar fields into one complex coordinate carrier.
    /// The unit anchor is the native singleton-B length. Its common Gram
    /// column retains phase between every residue and every release bank.
    fn reenter(&mut self,points:&mut [Point],torsions:&mut [f64],bank:&mut WorkBank,
        pairs:&mut [Pair],current:&mut f64,history:&mut History,feedback:bool)->Result<(),String>{
        if !self.enabled{return Ok(());}
        use crate::sic::{frame::Complex,carrier::recover_ray};
        let mut scalars=Vec::new();
        for (p,&angle) in points.iter().zip(torsions.iter()){
            scalars.extend([p.0,p.1,p.2,angle]);
        }
        for cell in &bank.cells{scalars.push(cell.coordinate());}
        let mut source=vec![Complex::new(1.0,0.0)];
        source.extend(scalars.chunks(2).map(|pair|Complex::new(pair[0],*pair.get(1).unwrap_or(&0.0))));
        let returned=recover_ray(&self.frame,&source,0,None).map_err(|e|e.to_string())?;
        let before_glut:Vec<_>=bank.cells.iter().map(Cell::remaining_stored).collect();
        let fields:Vec<_>=returned.recovered.iter().skip(1).flat_map(|z|[z.re,z.im]).collect();
        for (i,(p,angle)) in points.iter_mut().zip(torsions.iter_mut()).enumerate(){
            *p=(fields[4*i],fields[4*i+1],fields[4*i+2]);*angle=fields[4*i+3];
        }
        let offset=4*points.len();
        for (i,cell) in bank.cells.iter_mut().enumerate(){
            // z is the nonnegative radial readout of the returned coordinate.
            // The signed complex reconstruction error remains in the receipt.
            let value=fields[offset+i];cell.returned_coordinate=Some(libm::sqrt(value*value));
        }
        if feedback{reenter_preferences(pairs,bank);}
        let glut_drifts:Vec<_>=bank.cells.iter().zip(&before_glut).map(|(c,b)|c.remaining_stored()-b).collect();
        let glut_drift=glut_drifts.iter().sum::<f64>();
        let before=*current;*current=energy(points,pairs);let deformation_drift=*current-before;
        let drift=deformation_drift+glut_drift;
        self.signed_energy_drift+=drift;
        let record_index=self.returns.len();
        let values=|x:&[Complex]|x.iter().map(|z|[z.re,z.im]).collect::<Vec<_>>();
        let record=serde_json::json!({"source_amplitudes":values(&source),
            "recovered_amplitudes":values(&returned.recovered),"shared_anchor":returned.anchor,
            "gauge":[returned.gauge.re,returned.gauge.im],
            "frames":returned.frames.iter().map(|f|serde_json::json!({
                "source_indices":f.indices,"effect_masses":values(&f.masses),
                "signed_gram_residuals":values(&f.signed_gram_residuals)})).collect::<Vec<_>>(),
            "signed_amplitude_residuals":values(&returned.signed_amplitude_residuals),
            "full_gram_error_upper_bound":returned.full_gram_bound,
            "local_gram_residual":returned.local_residual,"anchor_residual":returned.anchor_residual,
            "roundoff_tolerance":returned.tolerance,"signed_energy_drift":drift,
            "signed_deformation_drift":deformation_drift,"signed_glut_drift":glut_drift,
            "source_release_energy_drifts":glut_drifts});
        // Immutable evidence stays exact while the hot carrier avoids one
        // separately allocated JSON value for every retained scalar.
        self.returns.push(protein_records::encode(&record));
        history.push(serde_json::json!({"kind":"sic-reentry","before":before,"after":*current,
            "return_index":record_index,"signed_energy_drift":drift,
            "signed_deformation_drift":deformation_drift,"signed_glut_drift":glut_drift}));
        Ok(())
    }
    fn record(&self,points:usize,bank:&WorkBank)->serde_json::Value{
        serde_json::json!({"schema":"native-ensemble-sic-carrier/v1","enabled":self.enabled,
            "carrier":"one complex coordinate vector for positions, torsions and live release coordinates",
            "scalar_layout":"per residue x,y,z,torsion; then source-addressed bank release coordinates; adjacent scalars form real/imaginary parts",
            "residue_count":points,"release_sources":bank.cells.iter().map(|c|c.source).collect::<Vec<_>>(),
            "anchor":"positive native unit, index zero shared by every ququart frame",
            "canonical_source":"G-mOMonadOS/src/sic/{frame,wh,carrier}.rs",
            "canonical_projectors":self.frame.projectors().iter().map(|p|
                p.values().iter().map(|z|[z.re,z.im]).collect::<Vec<_>>()).collect::<Vec<_>>(),
            "record_encoding":protein_records::ENCODING,
            "returns":self.returns,"next_force_reads_recovered_coordinates":true,
            "signed_energy_drift":self.signed_energy_drift})
    }
}

/// At a proposed conformation the same joint quadratic has its internal
/// minimum at qStar = distance*n/(p+n). Retain the banked lower boundary and
/// every source support. All eligible cells act together; source iteration
/// order does not choose which part can fund the ensemble's next action.
fn joint_release_plan(points:&[Point],bank:&WorkBank)->Vec<f64>{
    let frames=Frames::new(points);
    bank.cells.iter().map(|cell|{
        let [i,j,a,b]=cell.source;
        let (p,n,c)=(cell.positive as f64,cell.negative as f64,cell.multiplicity as f64);
        let distance=norm(sub(frames.sites[i][a],frames.sites[j][b]));
        let lower=libm::sqrt(p*n*n/(p+n));
        let target=(distance*n/(p+n)).max(lower).min(cell.preference());
        let remaining=(c*p*((p+n)*target*target/n-p*n)/2.0).max(0.0).min(cell.available());
        cell.available()-remaining
    }).collect()
}

fn release_for_move(pairs:&[Pair],bank:&WorkBank,trial:&[Point],current:f64,
    feedback:bool)->Option<(WorkBank,Vec<Pair>,serde_json::Value,f64,usize)>{
    let work=energy(trial,pairs)-current;
    if !work.is_finite()||work<=0.0{return None;}
    let plan=if feedback{joint_release_plan(trial,bank)}else{
        if work>bank.available(){return None;}
        let mut need=work;
        bank.cells.iter().map(|c|{let d=need.min(c.available());need-=d;d}).collect()
    };
    let at=|fraction:f64|{
        let mut next=bank.clone();
        for (c,&amount) in next.cells.iter_mut().zip(&plan){
            if amount>0.0{c.debit(fraction*amount);}
        }
        let mut channels=pairs.to_vec();
        let changed=if feedback{reenter_preferences(&mut channels,&next)}else{0};
        let score=energy(trial,&channels);
        let paid=bank.remaining_stored()-next.remaining_stored();
        (next,channels,score,paid,changed)
    };
    let mut high=1.0;
    if feedback{
        let full=at(high);
        if full.3<=0.0||full.2>current+full.3{return None;}
        let mut low=0.0;
        for _ in 0..f64::MANTISSA_DIGITS{
            let middle=(low+high)/2.0;let next=at(middle);
            if next.2<=current+next.3{high=middle;}else{low=middle;}
        }
    }
    let amounts:Vec<_>=plan.iter().map(|v|high*v).collect();
    let mut next=bank.clone();let payment=next.pay_allocated(&amounts)?;
    let mut channels=pairs.to_vec();
    let changed=if feedback{reenter_preferences(&mut channels,&next)}else{0};
    let score=energy(trial,&channels);
    if score+next.remaining_stored()>current+bank.remaining_stored(){return None;}
    Some((next,channels,payment,score,changed))
}
fn torsion_gradient(points:&[Point],gradient:&[Point])->Vec<f64>{
    let mut out=vec![0.0;points.len()];let mut force=(0.0,0.0,0.0);let mut moment=force;
    for i in (0..points.len()).rev(){
        force=add(force,gradient[i]);moment=add(moment,cross(points[i],gradient[i]));
        if i>=3{
            let pivot=points[i-1];let axis=unit(sub(pivot,points[i-2]));
            out[i]=dot(axis,sub(moment,cross(pivot,force)));
        }
    }out
}

#[derive(Clone)]
struct SourceGate { address:[usize;3],opcode:usize,depth:usize,word:String }
fn source_gates(words:&[Vec<Vec<ProgramSpec<'_>>>])->Result<Vec<SourceGate>,String>{
    let mut gates=Vec::new();
    for (i,residue) in words.iter().enumerate(){for (f,form) in residue.iter().enumerate(){
        for (p,program) in form.iter().enumerate(){
            let mut carried=phase_word::Continuation::default();
            for (opcode,mark) in program.word.chars().enumerate(){
                let depth=carried.frames().len();
                carried.advance_with(&mark.to_string(),&[i,f,p],|executed,address|{
                    if executed=='⊤'&&depth>0{gates.push(SourceGate{
                        address:*address,opcode,depth,word:program.word.into()});}
                    Ok(())
                })?;
            }
        }
    }}
    Ok(gates)
}

/// This description is the actual operand of the next transformation.
/// All signed gradients are retained; enabled directions are their admitted
/// mechanical face, alongside every source gate that can supply the action.
struct SelfDescription {
    torques:Vec<f64>,release_gradients:Vec<f64>,
    torsion_gates:Vec<Vec<usize>>,release_gates:Vec<Vec<usize>>,
    active_torsions:Vec<bool>,active_releases:Vec<bool>,maximum:f64,release_enabled:bool,
    frontier_active:bool,
}
impl SelfDescription {
    fn read(points:&[Point],pairs:&[Pair],bank:&WorkBank,gates:&[SourceGate],release_enabled:bool,
        prior:Option<&serde_json::Value>)->Self{
        let (_,gradient)=energy_gradient(points,pairs);
        let mut torques=torsion_gradient(points,&gradient);
        let torsion_gates:Vec<Vec<_>>=(0..points.len()).map(|i|gates.iter().enumerate()
            .filter_map(|(id,g)|(g.address[0]==i).then_some(id)).collect()).collect();
        let release_gates:Vec<Vec<_>>=bank.cells.iter().map(|c|gates.iter().enumerate()
            .filter_map(|(id,g)|(g.address[0]==c.source[0]||g.address[0]==c.source[1]).then_some(id)).collect()).collect();
        let frames=Frames::new(points);
        let mut release_gradients:Vec<_>=bank.cells.iter().map(|c|{
            let [i,j,a,b]=c.source;let p=c.positive as f64;let n=c.negative as f64;
            let distance=norm(sub(frames.sites[i][a],frames.sites[j][b]));
            c.multiplicity as f64*(p+n)*((p+n)*c.preference()/n-distance)
        }).collect();
        let frontier_active=prior.and_then(|model|model.get("transition_support")).is_some_and(|support|
            support["deformation_support"].as_bool()!=support["retained_work_support"].as_bool());
        if frontier_active{
            if let Some(prior_torques)=prior.and_then(|model|model["torques"].as_array()){
                for (current,previous) in torques.iter_mut().zip(prior_torques){
                    *current+=previous.as_f64().unwrap_or(0.0);
                }
            }
            if let Some(prior_gradients)=prior.and_then(|model|model["release_gradients"].as_array()){
                for (current,previous) in release_gradients.iter_mut().zip(prior_gradients){
                    *current+=previous.as_f64().unwrap_or(0.0);
                }
            }
        }
        let active_torsions:Vec<_>=torques.iter().enumerate().map(|(i,&g)|
            i>=3&&g!=0.0&&!torsion_gates[i].is_empty()).collect();
        let active_releases:Vec<_>=release_gradients.iter().enumerate().map(|(i,&g)|
            {let c=&bank.cells[i];let p=c.positive as f64;let n=c.negative as f64;
            release_enabled&&g>0.0&&c.available()>0.0&&c.preference()>libm::sqrt(p*n*n/(p+n))&&!release_gates[i].is_empty()}).collect();
        let quantum=2.0*core::f64::consts::PI/FOUR.len() as f64;
        let mut maximum=0.0f64;
        for (i,&g) in torques.iter().enumerate(){if active_torsions[i]{maximum=maximum.max((quantum*g).abs());}}
        for (i,&g) in release_gradients.iter().enumerate(){if active_releases[i]{
            maximum=maximum.max(bank.cells[i].negative as f64*g);
        }}
        Self{torques,release_gradients,torsion_gates,release_gates,active_torsions,active_releases,maximum,release_enabled,frontier_active}
    }
    fn record(&self,input_sic_return:Option<usize>,binding:usize)->serde_json::Value{
        serde_json::json!({"torques":self.torques,"release_gradients":self.release_gradients,
            "source_binding":binding,
            "active_torsions":self.active_torsions,"active_releases":self.active_releases,
            "frontier_active":self.frontier_active,
            "maximum_normalized_gradient":self.maximum,"input_sic_return":input_sic_return,
            "release_enabled":self.release_enabled,
            "generator":if self.frontier_active{"current and retained frontier gradients jointly drive native-quarter-turn torsions and release centers"}
                else{"simultaneous native-quarter-turn torsions and negative-frame-normalized retained centers"}})
    }
}

struct EnsembleController {
    carried:phase_word::Continuation<usize>,gates:Vec<SourceGate>,models:Vec<String>,word:String,
    bindings:Vec<(Vec<Vec<usize>>,Vec<Vec<usize>>)>,
}
impl EnsembleController {
    fn new(words:&[Vec<Vec<ProgramSpec<'_>>>])->Result<Self,String>{
        let mut carried=phase_word::Continuation::default();
        carried.advance_with("⊢⊙∈",&0,|_,_|Ok(()))?;
        Ok(Self{carried,gates:source_gates(words)?,models:Vec::new(),word:"⊢⊙∈".into(),bindings:Vec::new()})
    }
    fn bind(&mut self,description:&SelfDescription)->usize{
        if let Some(index)=self.bindings.iter().position(|(a,b)|
            *a==description.torsion_gates&&*b==description.release_gates){return index;}
        let index=self.bindings.len();
        self.bindings.push((description.torsion_gates.clone(),description.release_gates.clone()));index
    }
    fn attach_transition_support(&mut self,support:serde_json::Value)->Result<(),String>{
        let Some(last)=self.models.last_mut() else{return Err("missing executing self-description".into());};
        let mut model=protein_records::decode(last)?;
        model["transition_support"]=support;
        *last=protein_records::encode(&model);
        Ok(())
    }
    fn execute(&mut self,mut model:serde_json::Value,mut action:impl FnMut()->Result<bool,String>)->Result<bool,String>{
        let revision=self.models.len();let mut performed=false;
        // One parent frame remains live across all re-entries. Every inner
        // return deposits its description address back into that same frame.
        let body="∈⊙⊤⊥⊞∋⊙";
        self.carried.advance_with(body,&revision,|mark,_|{
            if mark=='⊤'{performed=action()?;}Ok(())
        })?;
        self.word.push_str(body);
        model["performed"]=serde_json::json!(performed);
        self.models.push(protein_records::encode(&model));
        Ok(performed)
    }
    fn record(&self)->serde_json::Value{
        let read=self.carried.readout();
        serde_json::json!({"schema":"native-ensemble-self-description/v1",
            "source_gates":self.gates.iter().map(|g|serde_json::json!({
                "address":g.address,"opcode_index":g.opcode,"enclosing_depth":g.depth,"word":g.word})).collect::<Vec<_>>(),
            "record_encoding":protein_records::ENCODING,
            "models":self.models,"source_bindings":self.bindings.iter().map(|(a,b)|serde_json::json!({
                "torsion_source_gates":a,"release_source_gates":b})).collect::<Vec<_>>(),
            "executed_word":self.word,"runtime_register":read.register,
            "fixed":self.carried.is_fixed(),"frames":self.carried.frames().iter().map(|f|f.iter()
                .map(|d|serde_json::json!({"id":d.id,"lane":d.lane,"model":d.observation})).collect::<Vec<_>>()).collect::<Vec<_>>(),
            "restore_supports":read.restore_supports,"clock_role":"bounded source-clock observation interval",
            "action_source":"regenerated signed whole-state description; no work-phase counter"})
    }
}

fn self_transform(model:&SelfDescription,pairs:&mut Vec<Pair>,bank:&mut WorkBank,
    torsions:&mut Vec<f64>,points:&mut Vec<Point>,current:&mut f64,
    history:&mut History,released:&mut f64,revision:usize)->Result<bool,String>{
    if model.maximum==0.0||!model.maximum.is_finite(){return Ok(false);}
    let quantum=2.0*core::f64::consts::PI/FOUR.len() as f64;
    let total_before=*current+bank.remaining_stored();let mut step=1.0/model.maximum;
    for _ in 0..f64::MANTISSA_DIGITS{
        let candidate:Vec<_>=torsions.iter().enumerate().map(|(i,&a)|
            if model.active_torsions[i]{a-step*quantum*quantum*model.torques[i]}else{a}).collect();
        let trial=backbone(&candidate);
        let amounts:Vec<_>=bank.cells.iter().enumerate().map(|(i,c)|{
            if !model.active_releases[i]{return 0.0;}
            let p=c.positive as f64;let n=c.negative as f64;let count=c.multiplicity as f64;
            let lower=libm::sqrt(p*n*n/(p+n));
            let center=(c.preference()-step*n*n*model.release_gradients[i]).max(lower);
            let available=(count*p*((p+n)*center*center/n-p*n)/2.0).max(0.0).min(c.available());
            c.available()-available
        }).collect();
        let mut next_bank=bank.clone();
        let has_release=amounts.iter().any(|&v|v>0.0);
        let payment=if has_release{next_bank.pay_allocated(&amounts)}else{None};
        if has_release&&payment.is_none(){step/=2.0;continue;}
        let mut next_pairs=pairs.clone();let changed=reenter_preferences(&mut next_pairs,&next_bank);
        let score=energy(&trial,&next_pairs);let total_after=score+next_bank.remaining_stored();
        let deformation_support=score<*current;
        let retained_work_support=next_bank.remaining_stored()<bank.remaining_stored();
        if total_after.is_finite()&&(deformation_support||retained_work_support){
            let actual_released=bank.remaining_stored()-next_bank.remaining_stored();
            let dissipated=actual_released-(score-*current);
            history.push(serde_json::json!({"kind":"self-transform","model":revision,"step":step,
                "before":*current,"after":score,"total_before":total_before,"total_after":total_after,
                "deformation_support":deformation_support,"retained_work_support":retained_work_support,
                "signed_deformation_change":score-*current,"signed_retained_work_change":
                    next_bank.remaining_stored()-bank.remaining_stored(),
                "frontier_before":if deformation_support!=retained_work_support{
                    serde_json::json!({"coordinates":points,"virtual_torsions":torsions,
                        "retained_bank":bank.record()})
                }else{serde_json::Value::Null},
                "frontier_candidate":if deformation_support!=retained_work_support{
                    serde_json::json!({"coordinates":trial,"virtual_torsions":candidate,
                        "retained_bank":next_bank.record(),
                        "pair_relation_rebuilt_from_retained_source_and_bank":true})
                }else{serde_json::Value::Null},
                "actual_released_glut":actual_released,"deformation_released":dissipated,
                "payment":payment,"reentered_preferences":changed,"executed_opcode":"⊤",
                "enclosing_frame_depth":2,"torque_and_release_from_retained_description":true}));
            *released+=dissipated;*torsions=candidate;*points=trial;*current=score;
            *bank=next_bank;*pairs=next_pairs;return Ok(true);
        }
        step/=2.0;
    }
    Ok(false)
}

/// Execute a source EVALT while its banked word is live. Available Glut work
/// admits a torsion displacement before it is committed to the shared carrier.
fn productive_move(phase:usize,words:&[Vec<Vec<ProgramSpec<'_>>>],pairs:&mut Vec<Pair>,
    bank:&mut WorkBank,torsions:&mut Vec<f64>,points:&mut Vec<Point>,current:&mut f64,
    history:&mut History,family:&mut Vec<serde_json::Value>,
    attempts:&mut Vec<serde_json::Value>,released:&mut f64,feedback:bool)->Result<bool,String>{
    if torsions.len()<4||bank.available()<=0.0{
        attempts.push(serde_json::json!({"phase":phase,"performed":false,"invoked":false,
            "reason":if torsions.len()<4{"no carried torsion"}else{"release bank exhausted"}}));
        return Ok(false);
    }
    let available_before=bank.available();
    let axis=3+phase*(torsions.len()-3)/FOUR.len();
    // The enclosing self-mark may already be fixed. Its retained typed
    // components are the live mechanical operations inside that boundary.
    let parts=words[axis][0].len();
    let part=if parts>1{1+phase%(parts-1)}else{0};
    let program=words[axis][0][part];
    let source=[axis,0,part];let mut invoked=false;let mut performed=false;let mut frame_depth=0usize;
    let read=phase_word::execute_with(program.word,&source,|mark,address|{
        if mark=='∈'{frame_depth+=1;}
        if mark=='∋'{frame_depth=frame_depth.saturating_sub(1);}
        if invoked||mark!='⊤'||frame_depth==0{return Ok(());}invoked=true;
        let sign=if FOUR[phase].to_wh2().0{-1.0}else{1.0};
        for orientation in [sign,-sign]{
            let mut angle=orientation*2.0*core::f64::consts::PI/FOUR.len() as f64;
            for _ in 0..f64::MANTISSA_DIGITS{
                let mut candidate=torsions.clone();candidate[axis]+=angle;
                let trial=backbone(&candidate);let score=energy(&trial,pairs);let work=score-*current;
                if score.is_finite()&&work>0.0{
                    if let Some((next_bank,next_pairs,payment,returned,changed))=
                        release_for_move(pairs,bank,&trial,*current,feedback){
                        let actual_released=bank.remaining_stored()-next_bank.remaining_stored();
                        let feedback_release=actual_released-(returned-*current);
                        history.push(serde_json::json!({"kind":"glut_work","phase":phase,
                            "source_program_address":address,"source_word":program.word,
                            "enclosing_frame_depth":frame_depth,
                            "executed_opcode":mark.to_string(),"torsion":axis,"angle":angle,
                            "before":*current,"after":returned,"payment":payment,
                            "pre_feedback_deformation":score,"reentered_preferences":changed,
                            "requested_coordinate_work":work,
                            "actual_released_glut":actual_released,
                            "feedback_released":feedback_release}));
                        *released+=feedback_release;
                        *bank=next_bank;*pairs=next_pairs;
                        *torsions=candidate;*points=trial;*current=returned;performed=true;
                        return Ok(());
                    }
                }
                angle/=2.0;
            }
        }
        Ok(())
    })?;
    attempts.push(serde_json::json!({"phase":phase,"torsion":axis,
        "source_program_address":source,"source_word":program.word,
        "invoked":invoked,"performed":performed,"inert_marks":read.inert,
        "available_work_before":available_before,"available_work_after":bank.available(),
        "reason":if performed{"productive release"}else if !invoked{
            "no live enclosed work gate before fixation"}else{"no admitted positive work step"}}));
    if performed{
        if let Some(event)=history.last_mut(){event["source_word_return"]=serde_json::json!({
            "register":read.register,"restore_supports":read.restore_supports,
            "surviving_source_addresses":read.surviving.iter().map(|d|d.observation).collect::<Vec<_>>()});}
        family.push(serde_json::json!({"kind":"productive-release-readout","phase":phase,
            "history_length":history.len(),"deformation":*current,
            "glut_work":bank.spent(),"torsions":torsions,"positions":points}));
    }
    Ok(performed)
}

/// Run one source-word clock of the shared constrained conformation.
/// Positions are native residue centres; atom-domain lifting remains explicit.
pub fn fold(words:&[Vec<&str>])->Result<serde_json::Value,String>{
    let sources:Vec<Vec<Vec<&str>>>=words.iter().map(|r|r.iter().map(|&w|vec![w]).collect()).collect();
    fold_sources(&sources)
}

/// Keep chemical forms and their source-word/primitive-axis parts nested.
pub fn fold_sources(words:&[Vec<Vec<&str>>])->Result<serde_json::Value,String>{
    let programs:Vec<Vec<Vec<_>>>=words.iter().map(|r|r.iter().map(|f|f.iter()
        .map(|&word|ProgramSpec{word,pairs:None}).collect()).collect()).collect();
    fold_programs(&programs)
}

/// Execute exported source wiring and its edge states inside the folding path.
pub fn fold_programs(words:&[Vec<Vec<ProgramSpec<'_>>>])->Result<serde_json::Value,String>{
    fold_programs_observed(words,None)
}

/// Bound the observation interval without changing any generating action.
/// The complete open carrier is returned at the boundary, including its last
/// coherent state, source banks and executed parent continuation.
pub fn fold_programs_observed(words:&[Vec<Vec<ProgramSpec<'_>>>],steps:Option<usize>)->Result<serde_json::Value,String>{
    fold_programs_observed_with_branch(words,steps,None)
}
pub fn fold_programs_observed_with_branch(words:&[Vec<Vec<ProgramSpec<'_>>>],steps:Option<usize>,
    branch:Option<&RouteBranch>)->Result<serde_json::Value,String>{
    fold_programs_with_dynamics(words,true,true,true,true,steps,None,branch)
}
/// Continue the saved physical and banked state without another preparation
/// measurement or replay of its physical transformations.
pub fn resume_programs(words:&[Vec<Vec<ProgramSpec<'_>>>],held:&serde_json::Value,steps:Option<usize>)->Result<serde_json::Value,String>{
    resume_programs_with_branch(words,held,steps,None)
}
fn resume_programs_with_branch(words:&[Vec<Vec<ProgramSpec<'_>>>],held:&serde_json::Value,steps:Option<usize>,
    branch:Option<&RouteBranch>)->Result<serde_json::Value,String>{
    fold_programs_with_dynamics(words,true,true,true,true,steps,Some(held),branch)
}

/// Use the source programs and their declared wiring retained in a public
/// checkpoint. No residue translation or physical action is repeated here.
pub fn resume_saved(held:&serde_json::Value,steps:Option<usize>)->Result<serde_json::Value,String>{
    let residues=held["source"]["residues"].as_array().ok_or("held carrier has no source residues")?;
    let mut wiring=Vec::<Option<Vec<(usize,usize)>>>::new();
    for residue in residues{for form in residue["forms"].as_array().ok_or("held residue has no forms")?{
        for part in form["native_components"].as_array().ok_or("held form has no components")?{
            wiring.push(if let Some(pairs)=part.get("pairs"){
                Some(serde_json::from_value(pairs.clone()).map_err(|e|e.to_string())?)
            }else{None});
        }
    }}
    let mut words=Vec::new();let mut ordinal=0;
    for residue in residues{
        let mut forms=Vec::new();
        for form in residue["forms"].as_array().ok_or("held residue has no forms")?{
            let mut parts=Vec::new();
            for part in form["native_components"].as_array().ok_or("held form has no components")?{
                parts.push(ProgramSpec{word:part["word"].as_str().ok_or("held component has no word")?,
                    pairs:wiring[ordinal].as_deref()});ordinal+=1;
            }
            forms.push(parts);
        }
        words.push(forms);
    }
    let branch=if held["route_branch"].is_object(){
        let route=&held["route_branch"];
        let rows=route["residue_pairs"].as_array().ok_or("held route branch has no residue-pair support")?;
        let mut residue_pairs=Vec::with_capacity(rows.len());
        for row in rows{
            let addresses=row["residues"].as_array().filter(|x|x.len()==2).ok_or("invalid held route pair")?;
            let i=addresses[0].as_u64().filter(|&x|x>0).ok_or("invalid held route address")? as usize-1;
            let j=addresses[1].as_u64().filter(|&x|x>0).ok_or("invalid held route address")? as usize-1;
            let mask=row["support_mask"].as_u64().filter(|&x|x<=u8::MAX as u64).ok_or("invalid held route support")? as u8;
            residue_pairs.push((i.min(j),i.max(j),mask));
        }
        Some(RouteBranch{source:route["source"].as_str().unwrap_or("").into(),
            glut_sha256:route["glut_sha256"].as_str().unwrap_or("").into(),
            pdb_sha256:route["pdb_sha256"].as_str().unwrap_or("").into(),residue_pairs})
    }else{None};
    resume_programs_with_branch(&words,held,steps,branch.as_ref())
}

#[cfg(test)]
fn fold_programs_with_work(words:&[Vec<Vec<ProgramSpec<'_>>>],enable_work:bool)->Result<serde_json::Value,String>{
    fold_programs_with_feedback(words,enable_work,true)
}
#[cfg(test)]
fn fold_programs_with_feedback(words:&[Vec<Vec<ProgramSpec<'_>>>],enable_work:bool,feedback:bool)->Result<serde_json::Value,String>{
    fold_programs_with_transport(words,enable_work,feedback,true)
}
#[cfg(test)]
fn fold_programs_with_transport(words:&[Vec<Vec<ProgramSpec<'_>>>],enable_work:bool,feedback:bool,coherence:bool)->Result<serde_json::Value,String>{
    fold_programs_with_dynamics(words,enable_work,feedback,coherence,false,None,None,None)
}
fn held_number(value:&serde_json::Value)->Result<f64,String>{
    value.as_f64().filter(|v|v.is_finite()).ok_or_else(||"invalid numeric field in held ensemble".into())
}
fn held_records(value:&serde_json::Value)->Result<Vec<String>,String>{
    value.as_array().ok_or("missing held ensemble records")?.iter().map(|v|
        if let Some(text)=v.as_str(){Ok(text.into())}else if v.is_object(){Ok(v.to_string())}
        else{Err("invalid held ensemble record".into())}).collect()
}

/// Rehydrate the actual coordinates and banks. The parent journal recovers
/// only the native evaluator's deposits; physical transformations stay held.
#[allow(clippy::too_many_arguments)]
fn restore_ensemble(held:&serde_json::Value,paths:&[Vec<Vec<Vec<u8>>>],sites:&SiteCarrier,
    pairs:&mut [Pair],bank:&mut WorkBank,torsions:&mut Vec<f64>,points:&mut Vec<Point>,
    current:&mut f64,history:&mut History,family:&mut Vec<serde_json::Value>,
    accepted:&mut Vec<serde_json::Value>,released:&mut f64,sic:&mut EnsembleSic,
    controller:&mut EnsembleController,initial:f64,ticks:usize)->Result<usize,String>{
    use serde_json::json;
    if held["schema"]!="native-glut-protein-fold/v10"||held["stalled"]!=false{
        return Err("resume requires an open native v10 observation return".into());
    }
    if held["source_paths"]!=serde_json::to_value(paths).map_err(|e|e.to_string())?
        ||held["site_graphs"]!=sites.record()||held_number(&held["initial_deformation"])?!=initial{
        return Err("held ensemble preparation differs from the supplied source".into());
    }
    let next=held["continuation"]["next_clock"].as_u64().and_then(|n|usize::try_from(n).ok())
        .filter(|&n|n<=ticks).ok_or("invalid held ensemble clock")?;
    let cells=held["glut_work"]["cells"].as_array().ok_or("missing held release banks")?;
    if cells.len()!=bank.cells.len(){return Err("held release bank dimension changed".into());}
    for (cell,saved) in bank.cells.iter_mut().zip(cells){
        if saved["source"]!=json!(cell.source)||saved["multiplicity"]!=json!(cell.multiplicity)
            ||saved["positive"]!=json!(cell.positive)||saved["negative"]!=json!(cell.negative)
            ||held_number(&saved["initial_stored_energy"])?!=cell.stored
            ||held_number(&saved["accessible_energy"])?!=cell.capacity{
            return Err("held release bank lost its source binding".into());
        }
        cell.spent=held_number(&saved["coordinate_work"])?;
        let coordinate=held_number(&saved["release_coordinate"])?;
        if cell.spent<0.0||coordinate<0.0{return Err("invalid held release coordinate".into());}
        cell.returned_coordinate=Some(coordinate);
    }
    let saved_points:Vec<[f64;3]>=serde_json::from_value(held["positions"].clone()).map_err(|e|e.to_string())?;
    let saved_angles:Vec<f64>=serde_json::from_value(held["torsions"].clone()).map_err(|e|e.to_string())?;
    if saved_points.len()!=points.len()||saved_angles.len()!=torsions.len()
        ||saved_points.iter().flatten().chain(saved_angles.iter()).any(|v|!v.is_finite()){
        return Err("invalid held conformation dimensions".into());
    }
    *points=saved_points.into_iter().map(|p|(p[0],p[1],p[2])).collect();*torsions=saved_angles;
    reenter_preferences(pairs,bank);
    *current=energy(points,pairs);
    if *current!=held_number(&held["final_deformation"])?{
        return Err("held coordinates and carried potential disagree".into());
    }
    *history=History::from_held(&held["transition_history"],held["ensemble_reentry"]["changed_preference_count"].as_u64().ok_or("missing retained preference count")?)?;
    *family=held["conformation_family"].as_array().ok_or("missing held conformation family")?.clone();
    *accepted=held["accepted_steps"].as_array().ok_or("missing held accepted steps")?.clone();
    *released=held_number(&held["energy_accounting"]["deformation_released"])?;
    sic.returns=held_records(&held["sic_carrier"]["returns"])?;
    sic.signed_energy_drift=held_number(&held["sic_carrier"]["signed_energy_drift"])?;
    if !held["sic_carrier"]["enabled"].as_bool().unwrap_or(false)||sic.returns.len()!=next+1
        ||accepted.len()!=next{return Err("held coherent history and clock disagree".into());}
    let last=protein_records::decode(sic.returns.last().ok_or("missing held SIC return")?)?;
    let amplitudes:Vec<[f64;2]>=serde_json::from_value(last["recovered_amplitudes"].clone()).map_err(|e|e.to_string())?;
    let fields:Vec<_>=amplitudes.iter().skip(1).flatten().copied().collect();
    if fields.len()<4*points.len()+bank.cells.len()||fields.iter().any(|v|!v.is_finite()){
        return Err("held coherent return has invalid dimensions".into());
    }
    for (i,(point,angle)) in points.iter().zip(torsions.iter()).enumerate(){
        if [point.0,point.1,point.2,*angle]!=fields[4*i..4*i+4]{
            return Err("held conformation is not its last coherent return".into());
        }
    }
    for (i,cell) in bank.cells.iter().enumerate(){
        let value=fields[4*points.len()+i];
        if cell.coordinate()!=libm::sqrt(value*value){return Err("held release bank is not its coherent return".into());}
    }
    let held_controller=&held["self_description"];
    for binding in held_controller["source_bindings"].as_array().ok_or("missing held source gates")?{
        let torsion=serde_json::from_value(binding["torsion_source_gates"].clone()).map_err(|e|e.to_string())?;
        let release=serde_json::from_value(binding["release_source_gates"].clone()).map_err(|e|e.to_string())?;
        controller.bindings.push((torsion,release));
    }
    let models=held_records(&held_controller["models"])?;
    if models.len()!=next{return Err("held descriptions and clock disagree".into());}
    for (revision,text) in models.into_iter().enumerate(){
        let model=protein_records::decode(&text)?;
        if model["performed"]!=true{return Err("held description has no admitted successor".into());}
        let body="∈⊙⊤⊥⊞∋⊙";
        controller.carried.advance_with(body,&revision,|_,_|Ok(()))?;
        controller.word.push_str(body);controller.models.push(text);
    }
    if controller.record()!=*held_controller{
        return Err("held parent continuation differs from its retained journal".into());
    }
    Ok(next)
}

fn fold_programs_with_dynamics(words:&[Vec<Vec<ProgramSpec<'_>>>],enable_work:bool,feedback:bool,coherence:bool,self_generated:bool,observation_steps:Option<usize>,resume:Option<&serde_json::Value>,branch:Option<&RouteBranch>)->Result<serde_json::Value,String>{
    if observation_steps==Some(0){return Err("observation interval requires at least one step".into());}
    if words.is_empty()||words.iter().any(|r|r.is_empty()||r.iter().any(|f|f.is_empty())){
        return Err("native folding requires every retained residue/form word".into());
    }
    let parts=words[0][0].len();
    if words.iter().any(|r|r.iter().any(|f|f.len()!=parts)){
        return Err("native component axes differ across retained forms".into());
    }
    let mut paths=Vec::new();let mut source_continuations=Vec::new();
    for residue in words{
        let mut forms=Vec::new();let mut form_continuations=Vec::new();for form in residue{
            let mut components=Vec::new();let mut component_continuations=Vec::new();for program in form{
                let (path,carried)=phase_word::native_state_trace(program.word)?;
                components.push(path);
                let read=carried.readout();
                component_continuations.push(serde_json::json!({
                    "runtime_register":read.register,"fixed":carried.is_fixed(),
                    "frames":carried.frames().iter().map(|frame|frame.iter().map(|d|
                        serde_json::json!({"id":d.id,"lane":d.lane})).collect::<Vec<_>>()).collect::<Vec<_>>(),
                    "surviving":read.surviving.iter().map(|d|
                        serde_json::json!({"id":d.id,"lane":d.lane})).collect::<Vec<_>>(),
                    "next_deposit_id":carried.next_deposit_id(),"bound_slots":carried.bound_slots(),
                    "cleared":read.cleared,"restored":read.restored,"exposed":read.exposed,
                    "deposits":read.deposits,"seeded":read.seeded,"inert":read.inert,
                    "restore_supports":read.restore_supports}));
            }
            forms.push(components);form_continuations.push(component_continuations);
        }paths.push(forms);source_continuations.push(form_continuations);
    }
    let ticks=paths.iter().flat_map(|r|r.iter()).flat_map(|f|f.iter()).map(|p|p.len()).sum::<usize>();
    let sites=SiteCarrier::prepare(words)?;
    let transfers=protein_orbital::table();
    let mut pairs=Vec::new();for i in 0..paths.len(){for j in i+1..paths.len(){pairs.push(prepare_pair_with_branch(i,j,&paths,&sites,&transfers,branch));}}
    let mut torsions:Vec<_>=paths.iter().map(|p|initial_torsion(p)).collect();
    let mut points=backbone(&torsions);let initial=energy(&points,&pairs);
    let mut bank=WorkBank{cells:pairs.iter().flat_map(|p|p.channels.iter().filter_map(move |c|{
        if c.positive==0||c.negative==0{return None;}
        Some(Cell::new([p.left,p.right,c.left,c.right],c.count,c.positive,c.negative))
    })).collect()};
    let mut history=History::default();let mut work_attempts=Vec::new();
    let mut family=vec![serde_json::json!({"kind":"prepared-readout",
        "history_length":0,"deformation":initial,"glut_work":0.0,"torsions":torsions,"positions":points})];
    let mut phase=0;let phase_interval=ticks.div_ceil(FOUR.len()).max(1);
    let mut deformation_released=0.0;
    let mut current=initial;let mut accepted=Vec::new();let mut stalled=false;
    let mut sic=EnsembleSic::new(coherence);
    let mut controller=EnsembleController::new(words)?;
    let start=if let Some(held)=resume{
        restore_ensemble(held,&paths,&sites,&mut pairs,&mut bank,&mut torsions,
            &mut points,&mut current,&mut history,&mut family,&mut accepted,
            &mut deformation_released,&mut sic,&mut controller,initial,ticks)?
    }else{
        sic.reenter(&mut points,&mut torsions,&mut bank,&mut pairs,&mut current,&mut history,feedback)?;
        0
    };
    let mut clock_completed=start;
    let quantum=2.0*core::f64::consts::PI/FOUR.len() as f64;
    let end=start.saturating_add(observation_steps.unwrap_or(ticks.saturating_sub(start))).min(ticks);
    for tick in start..end{
        clock_completed=tick+1;
        if self_generated{
            let prior=controller.models.last().map(|record|protein_records::decode(record)).transpose()?;
            let description=SelfDescription::read(&points,&pairs,&bank,&controller.gates,enable_work,prior.as_ref());
            let revision=controller.models.len();
            let binding=controller.bind(&description);
            let model=description.record(sic.returns.len().checked_sub(1),binding);
            let moved=controller.execute(model,||self_transform(&description,&mut pairs,&mut bank,
                &mut torsions,&mut points,&mut current,&mut history,&mut deformation_released,revision))?;
            if !moved{stalled=true;break;}
            let event=history.last().unwrap();
            controller.attach_transition_support(serde_json::json!({
                "deformation_support":event["deformation_support"],
                "retained_work_support":event["retained_work_support"]}))?;
            accepted.push(serde_json::json!({"tick":tick,"model":revision,"kind":"self-transform",
                "step":event["step"],"before":event["before"],"after":event["after"]}));
            sic.reenter(&mut points,&mut torsions,&mut bank,&mut pairs,&mut current,&mut history,feedback)?;
            family.push(serde_json::json!({"kind":"self-transformed-readout","model":revision,
                "history_length":history.len(),"deformation":current,
                "glut_work":bank.spent(),"torsions":torsions,"positions":points}));
            continue;
        }
        if enable_work&&phase<FOUR.len()&&tick>0&&tick%phase_interval==0{
            productive_move(phase,words,&mut pairs,&mut bank,&mut torsions,&mut points,&mut current,&mut history,&mut family,&mut work_attempts,&mut deformation_released,feedback)?;
            phase+=1;
            sic.reenter(&mut points,&mut torsions,&mut bank,&mut pairs,&mut current,&mut history,feedback)?;
        }
        let (_,gradient)=energy_gradient(&points,&pairs);let torque=torsion_gradient(&points,&gradient);
        let maximum=torque.iter().fold(0.0f64,|a,&b|a.max(b.abs()));
        if maximum==0.0{
            let mut moved=false;
            while enable_work&&phase<FOUR.len(){
                moved=productive_move(phase,words,&mut pairs,&mut bank,&mut torsions,&mut points,&mut current,&mut history,&mut family,&mut work_attempts,&mut deformation_released,feedback)?;
                phase+=1;if moved{break;}
            }
            if moved{
                sic.reenter(&mut points,&mut torsions,&mut bank,&mut pairs,&mut current,&mut history,feedback)?;
                continue;
            }
            stalled=true;break;
        }
        let mut step=quantum/maximum;let mut moved=false;
        for _ in 0..f64::MANTISSA_DIGITS{
            let candidate:Vec<_>=torsions.iter().zip(&torque).map(|(x,g)|x-step*g).collect();
            let trial=backbone(&candidate);let score=energy(&trial,&pairs);
            if score<current{
                accepted.push(serde_json::json!({"tick":tick,"step":step,"before":current,"after":score}));
                history.push(serde_json::json!({"kind":"deformation-release","tick":tick,
                    "step":step,"before":current,"after":score}));
                deformation_released+=current-score;
                torsions=candidate;points=trial;current=score;moved=true;break;
            }step/=2.0;
        }
        if !moved{
            let mut paid=false;
            while enable_work&&phase<FOUR.len(){
                paid=productive_move(phase,words,&mut pairs,&mut bank,&mut torsions,&mut points,&mut current,&mut history,&mut family,&mut work_attempts,&mut deformation_released,feedback)?;
                phase+=1;if paid{break;}
            }
            if paid{
                sic.reenter(&mut points,&mut torsions,&mut bank,&mut pairs,&mut current,&mut history,feedback)?;
                continue;
            }
            stalled=true;break;
        }
        sic.reenter(&mut points,&mut torsions,&mut bank,&mut pairs,&mut current,&mut history,feedback)?;
    }
    family.push(serde_json::json!({"kind":"returned-readout","history_length":history.len(),
        "deformation":current,"glut_work":bank.spent(),"torsions":torsions,"positions":points}));
    let mut max_bond_error=0.0f64;let mut max_gram_error=0.0f64;
    let cosine=dot(vertex(B4::N),vertex(B4::T))/dot(vertex(B4::N),vertex(B4::N));
    for i in 1..points.len(){max_bond_error=max_bond_error.max((norm(sub(points[i],points[i-1]))-1.0).abs());}
    for i in 1..points.len().saturating_sub(1){
        let gram=dot(unit(sub(points[i-1],points[i])),unit(sub(points[i+1],points[i])));
        max_gram_error=max_gram_error.max((gram-cosine).abs());
    }
    let spatial=Frames::new(&points);
    let pair_records:Vec<_>=pairs.iter().map(|p|serde_json::json!({"residues":[p.left,p.right],
        "contexts":["join_i","meet_i","join_t","meet_t","join_f","meet_f"],
        "state_counts":p.counts,"form_cycles":p.cycles,
        "information_maximum_counts":p.selected_counts,"maximizing_context_counts":p.maximizers,
        "information_state":p.information_state,"information_gains":p.information_gains,
        "source_register_pair_counts":p.source_pair_counts,
        "orbital_maximizing_counts":p.orbital_maximizers,"orbital_expanded_observations":p.orbital_expansions,
        "source_marginal_readout":{"stiffness":p.stiffness,"preferred":p.preferred,"retained_strain":p.strain},
        "retained_strain":p.channels.iter().map(|c|c.strain).sum::<f64>(),
        "spatial_channels":p.channels.iter().map(|c|serde_json::json!({
            "occupancies":[c.left,c.right],"count":c.count,
            "spin_counts":[c.positive,c.negative],"stiffness":c.stiffness,
            "preferred":c.preferred,"retained_strain":c.strain,
            "returned_distance":norm(sub(spatial.sites[p.left][c.left],spatial.sites[p.right][c.right]))})).collect::<Vec<_>>(),
        "returned_distance":norm(sub(points[p.left],points[p.right]))})).collect();
    let samples=core::array::from_fn(|mask|{
        let g=NativeGlut::from_state(BelnapSet::from_mask(mask as u8).unwrap());
        Response{energy:g.retained_strain(),radial_derivative:0.0}
    });
    let spectrum=Spectrum::from_samples(samples);
    let continuation=serde_json::json!({"boundary":"open retained native conformation",
            "reason":if stalled{if self_generated{"no admitted coupled successor in retained description"}
                else{"no representable torsion descent in this frame"}}else if clock_completed<ticks{
                    "observation-step interval ended"}else{"source-clock interval ended"},
            "observation_step_limit":observation_steps,"interval_start_clock":start,
            "completed_clock_steps":clock_completed,"next_clock":clock_completed,
            "next_work_phase":if self_generated{None}else{Some(phase)},
            "source_phase_interval":if self_generated{None}else{Some(phase_interval)},
            "retained":["source_paths","source_continuations","site_graphs","orbital_filling","orbital_transfer","spatial_sites","glut_work","ensemble_reentry","sic_carrier","self_description","work_attempts","transition_history","conformation_family","torsions","positions","pairs","accepted_steps"]});
    let reentered_preferences=history.reentered_preferences;
    let history=history.into_records();
    Ok(serde_json::json!({"schema":if self_generated{"native-glut-protein-fold/v10"}else{"native-glut-protein-fold/v9"},
        "selection":{"order":"SIXTEEN_3 information inclusion",
            "rule":"retain all spin-conserving joint placements and maximize information over source contexts and their orbital marginals",
            "state_boundary":"each source-addressed graph-edge pair at its joint clock observation",
            "coordinate_role":"mechanical readout of retained information maxima"},
        "units":"native membership-edge units","external_numeric_parameters":[],
        "representation":"joint orbital character sites in carried residue frames",
        "glut_work":bank.record(),"work_attempts":work_attempts,
        "sic_carrier":sic.record(points.len(),&bank),
        "self_description":controller.record(),
        "ensemble_reentry":{"enabled":feedback,
            "law":"qStar^2=n*(p*n+z^2)/(p+n); z is the retained release coordinate",
            "boundary":"same source-addressed channels, release bank and conformation",
            "interpretation":"native two-preference realization with retained banked strain",
            "changed_preference_count":reentered_preferences},
        "transition_history":history,"transition_history_encoding":protein_records::ENCODING,
        "conformation_family":family,
        "energy_accounting":{"initial":initial+bank.stored(),
            "returned":current+bank.remaining_stored()+deformation_released-sic.signed_energy_drift,
            "deformation_released":deformation_released,
            "sic_signed_energy_drift":sic.signed_energy_drift,
            "signed_residual":current+bank.remaining_stored()+deformation_released-initial-bank.stored()-sic.signed_energy_drift},
        "site_graphs":sites.record(),
        "spatial_sites":{"positions":spatial.sites,
            "frame":"axis, normal and tangent of the retained backbone triple",
            "radius":"singleton-B native equilibrium; FOUR character Gram normalization",
            "occurrence_map":"residue/form/component/edge/occupancy retains its address and reads character site [residue,occupancy]",
            "frame_derivatives_in_force":true},
        "orbital_filling":protein_orbital::FillingLattice::prepare(2)?.record(),
        "shell_composition":{"law":"E(A++B)=E(A)+E(B)+U_A D_B+U_B D_A+P_A(K_B+U_B+D_B)+P_B(K_A+U_A+D_A)",
            "meaning":"indexed native shell fusion; each cross-shell pair occurs once",
            "source_boundary":"complete placements retained; counts are additive mechanical observers",
            "channel_cross_readouts":(0..16usize).map(|i|
                protein_orbital::shell_cross_glut(&[(i/4) as u8],&[(i%4) as u8])).collect::<Vec<_>>()},
        "orbital_transfer":{"basis":["empty","spinUp","spinDown","paired"],
            "joint_index":"4*left_occupancy+right_occupancy",
            "conserved":["spin-up count per joint branch","spin-down count per joint branch"],
            "source_and_closed_families":transfers.iter().map(|t|[u64::from(t.source),u64::from(t.family),u64::from(t.selected)]).collect::<Vec<_>>(),
            "slot_counts":(0..16u8).map(protein_orbital::slot_counts).collect::<Vec<_>>(),
            "expanded_observations":pairs.iter().map(|p|p.orbital_expansions).sum::<u64>()},
        "native_membership_basis":["N","T","F","B"],"source_paths":paths,
        "route_branch":branch.map(|route|serde_json::json!({
            "source":route.source,"glut_sha256":route.glut_sha256,"pdb_sha256":route.pdb_sha256,
            "semantics":"T/F support joined by residue-pair address and included in native information maxima",
            "residue_pairs":route.residue_pairs.iter().map(|&(i,j,mask)|serde_json::json!({
                "residues":[i+1,j+1],"support_mask":mask,"support":route_support_names(mask)
            })).collect::<Vec<_>>()
        })),
        "source_continuations":source_continuations,
        "information_transition_count":pairs.iter().map(|p|p.information_gains.len()).sum::<usize>(),
        "maximal_joint_observations":pairs.iter().map(|p|p.selected_counts.iter().sum::<u64>()).sum::<u64>(),
        "tetrahedral_cosine":cosine,"source_clock_ticks":ticks,"stalled":stalled,
        "continuation":continuation,
        "initial_deformation":initial,"final_deformation":current,
        "retained_strain":pairs.iter().flat_map(|p|p.channels.iter()).map(|c|c.strain).sum::<f64>(),
        "accepted_steps":accepted,"torsions":torsions,
        "positions":points.iter().map(|p|[p.0,p.1,p.2]).collect::<Vec<_>>(),
        "max_bond_error":max_bond_error,"max_gram_error":max_gram_error,"pairs":pair_records,
        "native_strain_interaction_coefficients":spectrum.coefficients.iter().map(|r|r.energy).collect::<Vec<_>>()}))
}

#[cfg(test)]
mod tests{
    use super::*;
    use alloc::string::ToString;
    #[test]
    fn information_selection_retains_incomparable_support_and_positive_strain(){
        let t=BelnapSet::singleton(B4::T).mask();
        let f=BelnapSet::singleton(B4::F).mask();
        let contexts=pair_contexts(t,f);
        let maximum=information_maximum(&contexts);
        assert_eq!(maximum,t|f);
        assert_eq!(information_maximum(&[t,f]),maximum);
        assert_eq!(information_maximum(&[f,t,f,t]),maximum);
        let chosen=NativeGlut::from_state(BelnapSet::from_mask(maximum).unwrap());
        assert!(chosen.retained_strain()>0.0);
        for source in [t,f]{
            assert_eq!(NativeGlut::from_state(BelnapSet::from_mask(source).unwrap()).retained_strain(),0.0);
        }
    }
    #[test]
    fn native_contexts_and_support_extents_match_live_lean(){
        let exported:serde_json::Value=serde_json::from_str(
            include_str!("../tests/fixtures/native_information_selection_20261028.json")).unwrap();
        for a in 0..16u8{for b in 0..16u8{
            let expected=&exported["operations"][(usize::from(a)*16+usize::from(b)).to_string()];
            let actual=pair_contexts(a,b);
            for i in 0..actual.len(){assert_eq!(actual[i] as u64,expected[i].as_u64().unwrap());}
            let maximum=information_maximum(&actual);
            assert_eq!(maximum as u64,exported["maxima"][(usize::from(a)*16+usize::from(b)).to_string()].as_u64().unwrap());
            assert!(actual.contains(&maximum));
            assert!(actual.iter().all(|&state|state&!maximum==0));
        }}
        for mask in 0..16u8{
            let expected=&exported["states"][mask.to_string()];
            let g=NativeGlut::from_state(BelnapSet::from_mask(mask).unwrap());
            assert_eq!(g.positive as u64,expected[0].as_u64().unwrap());
            assert_eq!(g.negative as u64,expected[1].as_u64().unwrap());
        }
    }

    #[test]
    fn native_torsion_work_matches_its_carried_potential(){
        let paths=vec![vec![vec![vec![0,2,6,8,15]]];8];
        let sources=vec![vec![vec![ProgramSpec{word:"⊢∈⊤∋⊣",pairs:None}]];8];
        let sites=SiteCarrier::prepare(&sources).unwrap();
        let transfers=protein_orbital::table();
        let mut pairs=Vec::new();for i in 0..paths.len(){for j in i+1..paths.len(){pairs.push(prepare_pair(i,j,&paths,&sites,&transfers));}}
        let mut angles:Vec<_>=(0..paths.len()).map(|i|i as f64).collect();
        let points=backbone(&angles);let (_,gradient)=energy_gradient(&points,&pairs);
        let analytic=torsion_gradient(&points,&gradient);
        let delta=libm::sqrt(f64::EPSILON);
        for i in 3..angles.len(){
            angles[i]+=delta;let high=energy_gradient(&backbone(&angles),&pairs).0;
            angles[i]-=2.0*delta;let low=energy_gradient(&backbone(&angles),&pairs).0;angles[i]+=delta;
            let measured=(high-low)/(2.0*delta);
            let tolerance=libm::sqrt(delta)*(1.0+measured.abs());
            assert!((analytic[i]-measured).abs()<tolerance,"{i}: {} vs {measured}",analytic[i]);
        }
    }

    #[test]
    fn site_histogram_contraction_matches_every_addressed_edge_pair(){
        let word="⊢∈⊤≺∋⊣";
        let sources=vec![vec![vec![ProgramSpec{word,pairs:None}]];2];
        let sites=SiteCarrier::prepare(&sources).unwrap();
        let path=phase_word::native_state_path(word).unwrap();
        let paths=vec![vec![vec![path.clone()]];2];
        let transfers=protein_orbital::table();
        let actual=prepare_pair(0,1,&paths,&sites,&transfers);
        let record=sites.program(0,0,0).record();
        let mut counts=[[0u64;16];6];let mut selected=[0u64;16];
        for tick in 0..path.len(){
            let left=record["seed_edge_states"][path[tick] as usize].as_array().unwrap();
            let right=record["seed_edge_states"][path[(tick+1)%path.len()] as usize].as_array().unwrap();
            for x in left{for y in right{
                let states=pair_contexts(x.as_u64().unwrap() as u8,y.as_u64().unwrap() as u8);
                for (order,state) in states.iter().enumerate(){counts[order][*state as usize]+=1;}
                selected[Transfer::prepare(x.as_u64().unwrap() as u8,y.as_u64().unwrap() as u8).selected as usize]+=1;
            }}
        }
        assert_eq!(actual.counts,counts);assert_eq!(actual.selected_counts,selected);
        assert_eq!(selected.iter().sum::<u64>(),(path.len()*sites.program(0,0,0).edge_count().pow(2)) as u64);
    }

    #[test]
    fn stored_glut_work_changes_the_executing_carrier_without_erasing_information(){
        let sources=vec![vec![vec![
            ProgramSpec{word:"⊢≻⊡⋈∈⊤⊥⊞∋⊙≺⊣",pairs:None},
            ProgramSpec{word:"⊢∈⊤≺∋⊞⊣",pairs:None}]];8];
        let held=fold_programs_with_work(&sources,false).unwrap();
        let released=fold_programs_with_work(&sources,true).unwrap();
        assert!(released["glut_work"]["coordinate_work"].as_f64().unwrap()>0.0);
        assert_ne!(held["positions"],released["positions"]);
        assert_eq!(held["source_paths"],released["source_paths"]);
        assert_eq!(held["site_graphs"],released["site_graphs"]);
        assert_eq!(held["orbital_transfer"],released["orbital_transfer"]);
        for text in released["transition_history"].as_array().unwrap(){
            let event=protein_records::decode(text.as_str().unwrap()).unwrap();
            if event["kind"]=="glut_work"{assert_eq!(event["source_program_address"][2],1);}
        }
        for (h,r) in held["pairs"].as_array().unwrap().iter().zip(released["pairs"].as_array().unwrap()){
            assert_eq!(h["information_maximum_counts"],r["information_maximum_counts"]);
            assert_eq!(h["information_gains"],r["information_gains"]);
        }
        let account=&released["energy_accounting"];
        assert!(account["signed_residual"].as_f64().unwrap().abs()<
            f64::EPSILON*account["initial"].as_f64().unwrap()*released["transition_history"].as_array().unwrap().len() as f64);
    }

    #[test]
    fn inert_work_phase_retains_and_executes_later_phase_continuations(){
        let sources=vec![vec![vec![
            ProgramSpec{word:"⊢≻⊡⋈∈⊤⊥⊞∋⊙≺⊣",pairs:None},
            ProgramSpec{word:"⊢⊡∈⊤∋⊣",pairs:None},
            ProgramSpec{word:"⊢∈⊤≺∋⊞⊣",pairs:None},
            ProgramSpec{word:"⊢∈⊤≺∋⊞⊣",pairs:None}]];8];
        let returned=fold_programs_with_transport(&sources,true,true,true).unwrap();
        let attempts=returned["work_attempts"].as_array().unwrap();
        assert_eq!(attempts[0]["phase"],0);
        assert_eq!(attempts[0]["invoked"],false);
        assert!(attempts.iter().any(|a|a["phase"].as_u64().unwrap()>0&&a["performed"]==true));
        assert!(returned["glut_work"]["coordinate_work"].as_f64().unwrap()>0.0);
        if returned["stalled"]==true{assert_eq!(returned["continuation"]["next_work_phase"],4);}
    }

    #[test]
    fn retained_release_changes_the_next_force_and_the_whole_peptide_return(){
        let sources=vec![vec![vec![
            ProgramSpec{word:"⊢≻⊡⋈∈⊤⊥⊞∋⊙≺⊣",pairs:None},
            ProgramSpec{word:"⊢∈⊤≺∋⊞⊣",pairs:None}]];5];
        let prior=fold_programs_with_feedback(&sources,true,false).unwrap();
        let joint=fold_programs(&sources).unwrap();
        assert!(joint["ensemble_reentry"]["changed_preference_count"].as_u64().unwrap()>0);
        assert_ne!(prior["positions"],joint["positions"]);
        for name in ["source_paths","source_continuations","site_graphs","orbital_transfer"]{
            assert_eq!(prior[name],joint[name]);
        }
        assert!(joint["energy_accounting"]["signed_residual"].as_f64().unwrap().abs()<
            1e-12*joint["energy_accounting"]["initial"].as_f64().unwrap());
        let paths=vec![vec![vec![phase_word::native_state_path(sources[0][0][1].word).unwrap()]];5];
        let sites=SiteCarrier::prepare(&sources.iter().map(|r|vec![vec![r[0][1]]]).collect::<Vec<_>>()).unwrap();
        let mut pairs=Vec::new();let transfers=protein_orbital::table();
        for i in 0..5{for j in i+1..5{pairs.push(prepare_pair(i,j,&paths,&sites,&transfers));}}
        let points=backbone(&[0.0,1.0,2.0,3.0,4.0]);
        let before=energy_gradient(&points,&pairs).1;
        let mut bank=WorkBank{cells:pairs.iter().flat_map(|p|p.channels.iter().filter_map(move|c|{
            (c.positive>0&&c.negative>0).then(||Cell::new([p.left,p.right,c.left,c.right],c.count,c.positive,c.negative))
        })).collect()};
        bank.pay(bank.available()/2.0).unwrap();
        assert!(reenter_preferences(&mut pairs,&bank)>0);
        assert_ne!(before,energy_gradient(&points,&pairs).1);
    }

    #[test]
    fn sic_return_drives_the_same_ensemble_and_preserves_its_retained_source(){
        let sources=vec![vec![vec![
            ProgramSpec{word:"⊢≻⊡⋈∈⊤⊥⊞∋⊙≺⊣",pairs:None},
            ProgramSpec{word:"⊢∈⊤≺∋⊞⊣",pairs:None}]];5];
        let direct=fold_programs_with_transport(&sources,true,true,false).unwrap();
        let returned=fold_programs(&sources).unwrap();
        for name in ["source_paths","source_continuations","site_graphs","orbital_transfer"]{
            assert_eq!(direct[name],returned[name]);
        }
        let sic=&returned["sic_carrier"];
        assert_eq!(sic["next_force_reads_recovered_coordinates"],true);
        let records:Vec<serde_json::Value>=sic["returns"].as_array().unwrap().iter()
            .map(|v|protein_records::decode(v.as_str().unwrap()).unwrap()).collect();assert!(records.len()>1);
        let last=records.last().unwrap();
        let values:Vec<_>=last["recovered_amplitudes"].as_array().unwrap().iter().skip(1)
            .flat_map(|z|[z[0].as_f64().unwrap(),z[1].as_f64().unwrap()]).collect();
        for i in 0..5{
            for axis in 0..3{assert_eq!(returned["positions"][i][axis].as_f64().unwrap(),values[4*i+axis]);}
            assert_eq!(returned["torsions"][i].as_f64().unwrap(),values[4*i+3]);
        }
        assert!(records.iter().any(|r|r["source_amplitudes"].as_array().unwrap().iter()
            .any(|z|z[1].as_f64().unwrap().abs()>0.0)));
        for record in records{
            assert!(record["full_gram_error_upper_bound"].as_f64().unwrap()<=record["roundoff_tolerance"].as_f64().unwrap());
        }
        // A bound on one reconstruction does not bound a nonlinear sequence
        // of branch decisions. Check the actual conserved ledger for each
        // complete execution; every SIC return has its own bound above.
        for result in [&direct,&returned]{
            let account=&result["energy_accounting"];
            let tolerance=f64::EPSILON*account["initial"].as_f64().unwrap()*
                result["transition_history"].as_array().unwrap().len().max(1) as f64;
            assert!(account["signed_residual"].as_f64().unwrap().abs()<=tolerance);
        }
    }

    #[test]
    fn retained_description_regenerates_successors_inside_one_parent(){
        let sources=vec![vec![vec![
            ProgramSpec{word:"⊢≻⊡⋈∈⊤⊥⊞∋⊙≺⊣",pairs:None},
            ProgramSpec{word:"⊢∈⊤≺∋⊞⊣",pairs:None}]];5];
        let returned=fold_programs(&sources).unwrap();
        let controller=&returned["self_description"];
        let models:Vec<serde_json::Value>=controller["models"].as_array().unwrap().iter()
            .map(|v|protein_records::decode(v.as_str().unwrap()).unwrap()).collect();assert!(models.len()>1);
        assert_eq!(controller["fixed"],false);
        assert_eq!(controller["frames"].as_array().unwrap().len(),1);
        assert_eq!(controller["frames"][0].as_array().unwrap().len(),4*models.len());
        assert_eq!(controller["source_bindings"].as_array().unwrap().len(),1);
        assert!(returned["continuation"]["next_work_phase"].is_null());
        assert!(models.iter().any(|m|m["active_releases"]!=models[0]["active_releases"]));
        assert!(returned["transition_history"].as_array().unwrap().iter().map(|e|protein_records::decode(e.as_str().unwrap()).unwrap()).any(|e|
            e["kind"]=="self-transform"&&e["actual_released_glut"].as_f64().unwrap()>0.0));
    }

    #[test]
    fn observation_boundary_returns_the_exact_executing_prefix(){
        let sources=vec![vec![vec![ProgramSpec{word:"⊢∈⊤≺∋⊞⊣",pairs:None}]];5];
        let whole=fold_programs(&sources).unwrap();
        for steps in [1,3]{
            let held=fold_programs_observed(&sources,Some(steps)).unwrap();
            assert_eq!(held["continuation"]["reason"],"observation-step interval ended");
            assert_eq!(held["continuation"]["completed_clock_steps"],steps);
            assert_eq!(held["stalled"],false);
            for (section,key) in [("sic_carrier","returns"),("self_description","models")]{
                let prefix=held[section][key].as_array().unwrap();
                assert_eq!(prefix,&whole[section][key].as_array().unwrap()[..prefix.len()]);
            }
            let history=held["transition_history"].as_array().unwrap();
            assert_eq!(history,&whole["transition_history"].as_array().unwrap()[..history.len()]);
            assert_eq!(held["self_description"]["frames"][0].as_array().unwrap().len(),4*steps);
            let last=protein_records::decode(held["sic_carrier"]["returns"]
                .as_array().unwrap().last().unwrap().as_str().unwrap()).unwrap();
            for i in 0..5{
                assert_eq!(held["positions"][i][0],last["recovered_amplitudes"][1+2*i][0]);
                assert_eq!(held["positions"][i][1],last["recovered_amplitudes"][1+2*i][1]);
            }
        }
        assert!(fold_programs_observed(&sources,Some(0)).is_err());
    }

    #[test]
    fn saved_ensemble_resumes_the_same_physics_and_parent_without_an_extra_measurement(){
        let sources=vec![vec![vec![ProgramSpec{word:"⊢∈⊤≺∋⊞⊣",pairs:None}]];5];
        let uninterrupted=fold_programs_observed(&sources,Some(7)).unwrap();
        let first=fold_programs_observed(&sources,Some(2)).unwrap();
        let held:serde_json::Value=serde_json::from_str(&first.to_string()).unwrap();
        let continued=resume_programs(&sources,&held,Some(5)).unwrap();
        for key in ["positions","torsions","pairs","glut_work","accepted_steps",
            "transition_history","sic_carrier","self_description","energy_accounting"]{
            assert_eq!(continued[key],uninterrupted[key],"{key}");
        }
        assert_eq!(continued["continuation"]["interval_start_clock"],2);
        assert_eq!(continued["continuation"]["next_clock"],7);
        assert_eq!(continued["conformation_family"].as_array().unwrap().len(),
            uninterrupted["conformation_family"].as_array().unwrap().len()+1);
        let mut changed=held.clone();changed["positions"][0][0]=serde_json::json!(0.5);
        assert!(resume_programs(&sources,&changed,Some(1)).is_err());
        let mut changed=held.clone();changed["glut_work"]["cells"][0]["source"]=serde_json::json!([99,1,0,0]);
        assert!(resume_programs(&sources,&changed,Some(1)).is_err());
    }

    #[test]
    fn source_gate_permission_changes_the_actual_joint_successor(){
        let word="⊢∈⊤≺∋⊞⊣";
        let sources=vec![vec![vec![ProgramSpec{word,pairs:None}]];5];
        let sites=SiteCarrier::prepare(&sources).unwrap();let transfers=protein_orbital::table();
        let paths=vec![vec![vec![phase_word::native_state_path(word).unwrap()]];5];
        let mut pairs=Vec::new();for i in 0..5{for j in i+1..5{pairs.push(prepare_pair(i,j,&paths,&sites,&transfers));}}
        let mut bank=WorkBank{cells:pairs.iter().flat_map(|p|p.channels.iter().filter_map(move|c|
            (c.positive>0&&c.negative>0).then(||Cell::new([p.left,p.right,c.left,c.right],c.count,c.positive,c.negative)))).collect()};
        let mut angles=vec![0.0,1.0,2.0,3.0,4.0];let mut points=backbone(&angles);
        let gates=source_gates(&sources).unwrap();
        let live=SelfDescription::read(&points,&pairs,&bank,&gates,true,None);
        let held=SelfDescription::read(&points,&pairs,&bank,&[],true,None);
        assert_eq!(live.torques,held.torques);assert_eq!(live.release_gradients,held.release_gradients);
        assert!(live.maximum>0.0);assert_eq!(held.maximum,0.0);
        let mut current=energy(&points,&pairs);let mut history=History::default();let mut released=0.0;
        let prior=points.clone();
        assert!(!self_transform(&held,&mut pairs,&mut bank,&mut angles,&mut points,&mut current,&mut history,&mut released,0).unwrap());
        assert_eq!(points,prior);
        assert!(self_transform(&live,&mut pairs,&mut bank,&mut angles,&mut points,&mut current,&mut history,&mut released,77).unwrap());
        assert_ne!(points,prior);
    }

    #[test]
    fn source_gate_frontier_preserves_fixation_and_actual_opcode_addresses(){
        let sources=vec![vec![vec![ProgramSpec{word:"⊢∈⊤∋⊡∈⊤∋⊣",pairs:None}]]];
        let gates=source_gates(&sources).unwrap();assert_eq!(gates.len(),1);
        assert_eq!(gates[0].opcode,2);assert_eq!(gates[0].depth,1);
    }
}
