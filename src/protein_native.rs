//! Native-unit Glut realization. Inputs are the retained residue/form words.
//! All stiffnesses and preferred separations come from SIXTEEN_3 memberships;
//! geometry comes from FOUR's three nontrivial characters. No physical tables.
use alloc::{collections::BTreeMap, fmt::Write as _, string::{String,ToString}, vec, vec::Vec};
use crate::{belnap::B4, belnap_relation::BelnapSet, phase_word,
    protein_lattice::{Response, Spectrum}, protein_sites::{ProgramSpec, SiteCarrier},
    protein_orbital::{self, Transfer}, protein_site_geometry::Frames,
    protein_glut_work::{Cell,WorkBank}};

use crate::protein_records::{self,History};
#[path="protein_native_algebraic.rs"]
mod algebraic;

const FOUR: [B4; 4] = [B4::N, B4::T, B4::F, B4::B];
pub fn joint_root_return(value:&serde_json::Value)->Result<serde_json::Value,String>{
    if value["schema"]!="joint-source-mechanical-root-carrier/v1"
        ||value["root_certificate"]["complete_mechanical_root_enclosed"]!=true
        ||value["joint_relation"]["source_sha256"]!=value["source_sha256"]{
        return Err("joint root return lacks the enclosed source carrier".into());
    }
    let mut result=algebraic::joint_root_sic(value)?;
    let mut reconstructed=value.clone();
    reconstructed["algebraic_positions"]=result["returned_algebraic_positions"].clone();
    reconstructed["algebraic_bank"]=result["returned_algebraic_bank"].clone();
    reconstructed["four_state_character_images"]=result["returned_four_state_character_images"].clone();
    if reconstructed!=*value{return Err("reconstructed full joint carrier differs from its input".into());}
    result["full_carrier_input_equals_return"]=serde_json::json!(true);
    result["source_relation_sha256"]=value["source_relation_sha256"].clone();
    Ok(result)
}
pub fn joint_frame_return(source:&serde_json::Value,response:&serde_json::Value)
    ->Result<serde_json::Value,String>{
    let parse=|v:&serde_json::Value|->Result<Point,String>{
        let row=v.as_array().filter(|r|r.len()==3).ok_or("incomplete coordinate")?;
        let mut x=[0.0;3];for i in 0..3{x[i]=row[i].as_f64().filter(|x|x.is_finite())
            .ok_or("invalid coordinate")?;}Ok((x[0],x[1],x[2]))
    };
    let points=source["positions"].as_array().ok_or("missing source geometry")?
        .iter().map(parse).collect::<Result<Vec<_>,_>>()?;
    let rows=response["site_energy_gradient"].as_array().filter(|r|r.len()==points.len())
        .ok_or("joint gradient does not cover source residues")?;
    let mut gradient=Vec::new();
    for row in rows{
        let row=row.as_array().filter(|r|r.len()==4).ok_or("incomplete FOUR gradient")?;
        gradient.push([parse(&row[0])?,parse(&row[1])?,parse(&row[2])?,parse(&row[3])?]);
    }
    let frames=Frames::new(&points);
    let expected=source["spatial_sites"]["positions"].as_array().ok_or("missing source FOUR sites")?;
    if expected.len()!=points.len(){return Err("source FOUR frame count differs".into());}
    for (i,row) in frames.sites.iter().enumerate(){for (v,&point) in row.iter().enumerate(){
        if parse(&expected[i][v])?!=point{return Err("joint response source frame changed".into());}
    }}
    let pulled=frames.pullback(&points,&gradient);
    let torque=torsion_gradient(&points,&pulled);
    let mut result=response.clone();
    let residues=source["source"]["residues"].as_array().ok_or("missing source residue inventory")?;
    let chain=residues.iter().map(|r|r["aa"].as_str().ok_or("missing source residue name"))
        .collect::<Result<Vec<_>,_>>()?;
    result["source_ca_exclusion_relations_A"]=serde_json::json!(crate::peptide_closure::ca_exclusion_relations(&chain));
    result["coordinate_readout_A_per_native_unit"]=serde_json::json!(3.8);
    result["backbone_energy_gradient"]=serde_json::json!(pulled.iter().map(|p|[p.0,p.1,p.2]).collect::<Vec<_>>());
    result["torsion_energy_gradient"]=serde_json::json!(torque);
    if response["include_frame_jacobian"]==true{
        let mut jacobian=Vec::new();
        for residue in 0..points.len(){let mut vertices=Vec::new();for vertex in 0..4{
            let mut axes=Vec::new();for axis in 0..3{
                let mut basis=vec![[(0.0,0.0,0.0);4];points.len()];
                basis[residue][vertex]=match axis{0=>(1.0,0.0,0.0),1=>(0.0,1.0,0.0),_=>(0.0,0.0,1.0)};
                axes.push(torsion_gradient(&points,&frames.pullback(&points,&basis)));
            }vertices.push(axes);
        }jacobian.push(vertices);}
        result["native_site_torsion_pullback"]=serde_json::json!(jacobian);
    }
    result["frame_pullback_pending"]=serde_json::json!(false);
    result["mechanical_bank_return_pending"]=serde_json::json!(true);
    result["carried_source_positions"]=source["positions"].clone();
    result["carried_source_bank"]=source["glut_work"].clone();
    let bank_rows=source["glut_work"]["cells"].as_array().ok_or("joint return lacks source bank cells")?;
    let mut cells=Vec::new();
    for row in bank_rows{
        let mut address=[0usize;4];for i in 0..4{address[i]=row["source"][i].as_u64()
            .ok_or("invalid source bank address")? as usize;}
        if address[0]>=points.len()||address[1]>=points.len()||address[2]>=4||address[3]>=4{
            return Err("bank address exceeds carried FOUR frame".into());
        }
        let mut cell=Cell::new(address,row["multiplicity"].as_u64().ok_or("missing bank mass")?,
            row["positive"].as_u64().filter(|x|*x>0).ok_or("invalid bank positive rank")? as usize,
            row["negative"].as_u64().filter(|x|*x>0).ok_or("invalid bank negative rank")? as usize);
        cell.spent=row["coordinate_work"].as_f64().filter(|x|x.is_finite()&&*x>=0.0)
            .ok_or("invalid carried bank work")?;
        cell.returned_coordinate=Some(row["release_coordinate"].as_f64().filter(|x|x.is_finite()&&*x>=0.0)
            .ok_or("invalid carried release coordinate")?);
        cells.push(cell);
    }
    let bank=WorkBank{cells};let plan=joint_release_plan(&points,&bank);
    let mut candidate=bank.clone();let payment=candidate.pay_allocated(&plan);
    result["conditional_bank_candidate"]=candidate.record();
    result["conditional_bank_work_receipt"]=serde_json::json!(payment);
    result["conditional_bank_admission"]=serde_json::json!({"status":"held",
        "reason":"complete joint mechanical work admission required before mutation",
        "source_addresses_preserved":bank.cells.iter().zip(&candidate.cells).all(|(a,b)|a.source==b.source)});
    result["joint_mechanical_sic"]=algebraic::joint_mechanical_sic(&result)?;
    Ok(result)
}
pub fn whole_algebraic_return(value:&serde_json::Value)->Result<serde_json::Value,String>{
    if value["closure_verified"]!=true{return Err("whole source relation is not closed".into());}
    let source_binding=value["source_sha256"].as_str().filter(|x|x.len()==64)
        .ok_or("whole return lacks its prepared-source digest")?;
    let construction=&value["construction"];
    let occurrence=&construction["source_occurrence_tensor"];
    let events=&construction["shared_source_event_tensor"];
    if construction["shared_residue_port_contraction"]["kind"].as_str()
        .is_some_and(|kind|kind.contains("rank-one")){
        return Err("whole-chain closure requires executed joint source ancestry; rank-one marginal contraction is incomplete".into());
    }
    let event_rows=events["events"].as_array().ok_or("whole return lacks the shared source-event tensor")?;
    let event_factors=events["factorized_event_tensors"].as_array()
        .ok_or("whole return lacks the factorized 20-residue source tensor")?;
    let source_four=events["source_four_coordinates"].as_array()
        .ok_or("whole return lacks its source-bound FOUR coordinate family")?;
    let factor_sha=events["factorized_event_tensor_sha256"].as_str()
        .filter(|x|x.len()==64).ok_or("whole return lacks the factorized source-tensor digest")?;
    let event_count=events["event_count"].as_u64().ok_or("whole return lacks its source-event count")? as usize;
    let pair_uses=events["pair_tensor_uses"].as_u64().ok_or("whole return lacks its pair-tensor use count")? as usize;
    let expected_pairs=event_rows.first().and_then(|event|event[0].as_array())
        .map(|forms|forms.len()*forms.len().saturating_sub(1)/2).unwrap_or(0);
    if event_count==0||event_rows.len()!=event_count||pair_uses!=event_count*190
        ||event_factors.len()!=event_count||source_four.len()!=20||source_four.iter().any(|port|
            port.as_array().map_or(true,|coordinates|coordinates.len()!=3||coordinates.iter().any(|x|
                x["denominator_power_two"]!=128||x["lower_numerator"].as_str().is_none()
                    ||x["upper_numerator"].as_str().is_none())))
        ||event_factors.iter().any(|event|
            event.as_array().is_none_or(|ports|ports.len()!=20||ports.iter().any(|port|{
                let denominator=port[0].as_u64().unwrap_or(0);
                let states=port[1].as_array();
                let four=port[2].as_array();
                denominator==0||states.is_none_or(|rows|rows.iter().try_fold(0u128,|sum,row|
                    Some(sum+row[1].as_u64().unwrap_or(0) as u128))
                    .is_none_or(|sum|sum!=denominator as u128))
                    ||four.is_none_or(|coordinates|coordinates.len()!=3
                        ||coordinates.iter().any(|coordinate|coordinate.as_i64().is_none()))
            })))
        ||expected_pairs!=190||event_rows.iter().any(|event|event[4].as_array()
            .is_none_or(|refs|refs.len()!=expected_pairs||refs.iter().any(|ordinal|ordinal.as_u64().is_none())))
        ||events["all_occurrences_rejoined"]!=true
        ||events["windings_sum_to_residue_return"]!=true
        ||occurrence["address_fields"]!=serde_json::json!(["residue_pair","left_form","right_form","component","tick"])
        ||occurrence["occurrence_count"].as_u64().unwrap_or(0)==0{
        return Err("whole return source ancestry is incomplete".into());
    }
    let formula=construction["formula"].as_str().ok_or("whole return lacks its source-bound equation family")?;
    if !formula.contains(source_binding){return Err("whole equation family is not bound to the prepared source".into());}
    let carried_bank=&construction["carried_bank_tensor"];
    let carried_bank_count=carried_bank["channels"].as_u64()
        .ok_or("whole return lacks the source-addressed carried bank count")? as usize;
    let carried_bank_sha=carried_bank["records_sha256"].as_str()
        .filter(|x|x.len()==64).ok_or("whole return lacks the carried bank tensor digest")?;
    let roots=value["root_observations"].as_array().filter(|rows|!rows.is_empty())
        .ok_or("whole return lacks its certified root carrier")?;
    let contracted_four=&roots[0]["root_certificate"]["shared_port_contraction"]["contracted_source_four_coordinates"];
    if roots.iter().any(|root|root["algebraic_positions"].as_array().is_none()
        ||root["algebraic_bank"].as_array().is_none_or(|rows|rows.len()!=carried_bank_count)
        ||root["root_certificate"].is_null()
        ||root["root_certificate"]["carried_bank_channels"].as_u64()!=Some(carried_bank_count as u64)
        ||root["root_certificate"]["carried_bank_tensor_sha256"]!=carried_bank_sha
        ||root["root_certificate"]["factorized_event_tensor_sha256"]!=factor_sha
        ||root["source_four_coordinates"]!=*contracted_four
        ||root["root_certificate"]["shared_port_contraction_verified"]!=true
        ||root["root_certificate"]["shared_port_contraction"]["pair_tensor_occurrence_uses"]!=pair_uses
        ||root["root_certificate"]["shared_port_contraction"]["rank_one_occurrence_fibres_verified"]!=true
        ||root["root_certificate"]["shared_port_contraction"]["response_coefficients_sha256"].as_str().is_none_or(|sha|sha.len()!=64)
        ||root["root_certificate"]["shared_residue_port_factorization_verified"]!=true){
        return Err("whole root carrier lost its coordinate, bank or certificate boundary".into());
    }
    let mut parent=phase_word::Continuation::<usize>::default();
    let prepared=construction["prepared_state"]["self_description"]["executed_word"].as_str().unwrap_or("⊢⊙∈");
    parent.advance_with(prepared,&0,|_,_|Ok(()))?;
    let mut returns=Vec::new();
    for observation in [0,1]{
        parent.advance_with("∈⊤⊥⊞∋⊙",&observation,|mark,_|{
            if mark=='⊤'{returns.push(algebraic::whole_sic(value)?);}Ok(())
        })?;
    }
    if returns.len()!=2||returns[0]!=returns[1]{return Err("whole algebraic re-entry changed its closed SIC return".into());}
    if returns[0]["input_scalar_bounds"]!=returns[0]["returned_scalar_bounds"]
        ||returns[1]["input_scalar_bounds"]!=returns[1]["returned_scalar_bounds"]{
        return Err("whole SIC return changed a source coordinate or bank bound".into());
    }
    let source_bound_carrier=serde_json::json!({"source_sha256":source_binding,"formula":formula,
        "source_occurrence_tensor":occurrence,"shared_source_event_tensor":events,
        "carried_bank_tensor":carried_bank,
        "factorized_event_tensor_sha256":factor_sha,"source_four_coordinates":source_four,
        "root_observations":roots,"scalar_bounds":returns[0]["input_scalar_bounds"]});
    let returned_full_carrier=serde_json::json!({"source_sha256":source_binding,"formula":formula,
        "source_occurrence_tensor":occurrence,"shared_source_event_tensor":events,
        "carried_bank_tensor":carried_bank,
        "factorized_event_tensor_sha256":factor_sha,"source_four_coordinates":source_four,
        "root_observations":roots,"scalar_bounds":returns[0]["returned_scalar_bounds"]});
    if source_bound_carrier!=returned_full_carrier{
        return Err("whole SIC return differs from the source-bound full carrier".into());
    }
    let returned_carrier=serde_json::json!({"source_sha256":source_binding,"formula":formula,
        "source_occurrence_tensor":occurrence,"shared_source_event_tensor":events,
        "carried_bank_tensor":carried_bank,
        "factorized_event_tensor_sha256":factor_sha,"source_four_coordinates":source_four,
        "root_observations":roots,"sic_return":returns[0]});
    let repeated_carrier=serde_json::json!({"source_sha256":source_binding,"formula":formula,
        "source_occurrence_tensor":occurrence,"shared_source_event_tensor":events,
        "carried_bank_tensor":carried_bank,
        "factorized_event_tensor_sha256":factor_sha,"source_four_coordinates":source_four,
        "root_observations":roots,"sic_return":returns[1]});
    if returned_carrier!=repeated_carrier{return Err("whole source-bound carrier changed across fixed-point re-entry".into());}
    parent.advance_with("∋⊡⊣",&0,|_,_|Ok(()))?;
    let read=parent.readout();
    Ok(serde_json::json!({"executed_word":"⊢⊙∈∈⊤⊥⊞∋⊙∈⊤⊥⊞∋⊙∋⊡⊣",
        "register":read.register,"fixed":parent.is_fixed(),
        "sic_return":returns[0],"same_parent_reentry_equal":true,
        "full_carrier_fixed_point":{"verified":true,"source_sha256":source_binding,
            "formula_binding":formula,"shared_source_events":event_count,
            "shared_pair_tensor_uses":pair_uses,"source_occurrences":occurrence["occurrence_count"],
            "carried_bank_channels":carried_bank_count,"carried_bank_tensor_sha256":carried_bank_sha,
            "root_observations":roots.len(),"coordinates_and_bank_bounds_unchanged":true,
            "source_event_tensor_preserved":true,"source_bound_carrier_equals_returned_full_carrier":true,
            "sic_returns_compared":2},
        "family_binding":construction["formula"],
        "source_binding":value["source_sha256"],"reentry":value["reentry"]}))
}
pub fn whole_source_event_sic(value:&serde_json::Value)->Result<serde_json::Value,String>{
    let events=&value["construction"]["shared_source_event_tensor"];
    let factors=events["factorized_event_tensors"].as_array()
        .ok_or("source-event SIC lacks factorized shared event tensors")?;
    let rows=events["events"].as_array().ok_or("source-event SIC lacks event addresses")?;
    let event_count=events["event_count"].as_u64().unwrap_or(0) as usize;
    let pair_uses=events["pair_tensor_uses"].as_u64().unwrap_or(0) as usize;
    let source=&value["construction"]["source_occurrence_tensor"];
    if event_count==0||factors.len()!=event_count||rows.len()!=event_count
        ||pair_uses!=event_count*190||events["all_occurrences_rejoined"]!=true
        ||events["windings_sum_to_residue_return"]!=true
        ||events["factorized_event_tensor_sha256"].as_str().is_none_or(|sha|sha.len()!=64)
        ||source["address_fields"]!=serde_json::json!(["residue_pair","left_form","right_form","component","tick"])
        ||source["occurrence_count"].as_u64().unwrap_or(0)==0
        ||rows.iter().any(|event|event.as_array().is_none_or(|row|row.len()!=5
            ||row[4].as_array().is_none_or(|refs|refs.len()!=190
                ||refs.iter().any(|index|index.as_u64().is_none()))))
        ||factors.iter().any(|event|event.as_array().is_none_or(|ports|ports.len()!=20
            ||ports.iter().any(|port|{
                let denominator=port[0].as_u64().unwrap_or(0);
                denominator==0||port[1].as_array().is_none_or(|states|states.iter()
                    .try_fold(0u128,|sum,state|Some(sum+state[1].as_u64()? as u128))
                    .is_none_or(|sum|sum!=denominator as u128))
                    ||port[2].as_array().is_none_or(|moments|moments.len()!=3
                        ||moments.iter().any(|value|value.as_i64().is_none()))
            }))) {
        return Err("source-event SIC factorization or occurrence ancestry is incomplete".into());
    }
    algebraic::source_event_sic(value)
}
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
    joint_terms:Vec<(usize,i128)>,
    stiffness:f64,preferred:f64,strain:f64,
}
#[derive(Clone,PartialEq,Eq)]
struct SourceOccurrence {
    left_form:usize,right_form:usize,component:usize,tick:usize,
    joint_counts:Vec<(usize,u64)>,
    mobius_terms:Vec<(usize,i128)>,
}
fn encode_source_occurrences(occurrences:&[SourceOccurrence])->String{
    let mut payload=String::from("[");
    for (ordinal,occurrence) in occurrences.iter().enumerate(){
        if ordinal!=0{payload.push(',');}
        write!(&mut payload,"[{},{},{},{},[",occurrence.left_form,occurrence.right_form,
            occurrence.component,occurrence.tick).expect("string write");
        for (cell,(index,count)) in occurrence.joint_counts.iter().enumerate(){
            if cell!=0{payload.push(',');}
            write!(&mut payload,"[{index},{count}]").expect("string write");
        }
        payload.push_str("],[");
        for (term,(index,coefficient)) in occurrence.mobius_terms.iter().enumerate(){
            if term!=0{payload.push(',');}
            write!(&mut payload,"[{index},{coefficient}]").expect("string write");
        }
        payload.push_str("]]");
    }
    payload.push(']');
    protein_records::encode_text(&payload)
}
#[derive(Clone)]
struct Pair {
    left:usize,right:usize,counts:[[u64;16];6],cycles:Vec<(usize,usize,usize,usize)>,
    selected_counts:[u64;16],maximizers:[u64;6],information_state:u8,
    source_pair_counts:Vec<u64>,mobius_pair_interactions:Vec<(usize,i128)>,
    source_occurrences:Vec<SourceOccurrence>,
    source_mechanical_moments:[f64;3],
    orbital_maximizers:u64,orbital_expansions:u64,
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

/// Möbius inversion on the eight named membership bits of one SIXTEEN_3 pair.
/// Flat state address 16*left+right gives four left bits and four right bits.
fn mobius_pair_interactions(values:&[u64])->Vec<i128>{
    assert_eq!(values.len(),256);
    let mut coefficients=values.iter().map(|&x|x as i128).collect::<Vec<_>>();
    for bit in 0..8{
        for mask in 0..256{
            if mask&(1<<bit)!=0{coefficients[mask]-=coefficients[mask^(1<<bit)];}
        }
    }
    coefficients
}

/// Unique multilinear extension of the full membership response. At every
/// Boolean corner this evaluates the original joint-state response exactly.
fn joint_response_polynomial(coefficients:&[i128],memberships:&[f64;8])->f64{
    joint_response_differential(coefficients,memberships).0
}
fn joint_response_differential(coefficients:&[i128],memberships:&[f64;8])->(f64,[f64;8]){
    let mut value=0.0;let mut gradient=[0.0;8];
    for (mask,&coefficient) in coefficients.iter().enumerate(){
        if coefficient==0{continue;}
        let product=|omitted:Option<usize>|{
            let mut term=coefficient as f64;
            for (bit,&membership) in memberships.iter().enumerate(){
                if mask&(1<<bit)!=0&&omitted!=Some(bit){term*=membership;}
            }
            term
        };
        value+=product(None);
        for (bit,partial) in gradient.iter_mut().enumerate(){
            if mask&(1<<bit)!=0{*partial+=product(Some(bit));}
        }
    }
    (value,gradient)
}

/// Exact inverse accumulation for the pair's 256 joint support states.
fn reconstruct_pair_interactions(coefficients:&[i128])->Vec<i128>{
    assert_eq!(coefficients.len(),256);
    let mut values=coefficients.to_vec();
    for bit in 0..8{
        for mask in 0..256{
            if mask&(1<<bit)!=0{values[mask]+=values[mask^(1<<bit)];}
        }
    }
    values
}

/// Native information-join pushforward on the full joint distribution.
/// Distinct source states remain available before the shell observation.
fn join_pair_source(counts:&[u64],support:usize)->Vec<u64>{
    assert_eq!(counts.len(),256);assert!(support<256);
    let mut returned=vec![0u64;256];
    for (state,&count) in counts.iter().enumerate(){returned[state|support]+=count;}
    returned
}

/// The FOUR-character image of a support subset: its affine coordinate is
/// retained as cardinality plus the sum of its tetrahedral character vertices.
fn tetrahedral_subset_coordinates(mask:usize)->(usize,Point){
    let mut count=0;
    let mut point=(0.0,0.0,0.0);
    for (axis,value) in FOUR.into_iter().enumerate(){
        if mask&(1<<axis)!=0{count+=1;point=add(point,vertex(value));}
    }
    (count,point)
}

/// Invert the affine FOUR character address, retaining the constant character.
fn support_from_tetrahedral_address(rank:usize,point:Point)->usize{
    let mut mask=0;
    for (axis,value) in FOUR.into_iter().enumerate(){
        let numerator=rank as f64+dot(point,vertex(value));
        assert!(numerator==0.0||numerator==4.0,
            "affine FOUR address must recover Boolean membership");
        if numerator==4.0{mask|=1<<axis;}
    }
    mask
}

/// Contract the constant character (support cardinality) with FOUR's three
/// tetrahedral characters. This is the affine 4D wedge's spatial component.
fn affine_frame_wedge(a:u8,b:u8)->[i128;3]{
    let (ar,ap)=tetrahedral_subset_coordinates(a as usize);
    let (br,bp)=tetrahedral_subset_coordinates(b as usize);
    [
        ar as i128*bp.0 as i128-br as i128*ap.0 as i128,
        ar as i128*bp.1 as i128-br as i128*ap.1 as i128,
        ar as i128*bp.2 as i128-br as i128*ap.2 as i128,
    ]
}

/// The Lean one-shot pair operator: retain all six SIXTEEN_3 join/meet
/// boundaries, pair them by information/truth/falsity order, then contract
/// each affine FOUR wedge into the spatial character coordinates.
fn pair_context_contraction(left:u8,right:u8)->[i128;3]{
    let contexts=pair_contexts(left,right);
    let mut result=[0i128;3];
    for (join,meet) in [(contexts[0],contexts[1]),(contexts[2],contexts[3]),(contexts[4],contexts[5])]{
        for (target,value) in result.iter_mut().zip(affine_frame_wedge(join,meet)){
            *target+=value;
        }
    }
    result
}

/// Transport coefficients through the lossless affine character frame before
/// inverse accumulation. Channel preparation consumes this returned carrier.
fn frame_transport_pair_interactions(coefficients:&[i128])->Vec<i128>{
    let mut returned=vec![0;256];
    for (index,&coefficient) in coefficients.iter().enumerate(){
        let (lr,lp)=tetrahedral_subset_coordinates(index/16);
        let (rr,rp)=tetrahedral_subset_coordinates(index%16);
        let address=16*support_from_tetrahedral_address(lr,lp)
            +support_from_tetrahedral_address(rr,rp);
        assert_eq!(address,index,"FOUR frame transport changed pair ancestry");
        returned[address]=coefficient;
    }
    returned
}

#[cfg(test)]
fn prepare_pair(i:usize,j:usize,paths:&[Vec<Vec<Vec<u8>>>],sites:&SiteCarrier,transfers:&[Transfer;256])->Pair{
    prepare_pair_with_branch(i,j,paths,sites,transfers,None)
}
fn prepare_pair_with_branch(i:usize,j:usize,paths:&[Vec<Vec<Vec<u8>>>],sites:&SiteCarrier,
    transfers:&[Transfer;256],branch:Option<&RouteBranch>)->Pair{
    let mut counts=[[0u64;16];6];let mut cycles=Vec::new();let mut source_occurrences=Vec::new();
    let mut selected_counts=[0u64;16];let mut maximizers=[0u64;6];
    let mut information_state=0u8;let mut information_gains=Vec::new();
    let mut source_pair_counts=vec![0;256];let mut source_families:Vec<u16>=transfers.iter().map(|t|t.family).collect();
    let mut orbital_maximizers=0;let mut orbital_expansions=0;
    for (a,left_form) in paths[i].iter().enumerate(){for (b,right_form) in paths[j].iter().enumerate(){
      for (part,(left,right)) in left_form.iter().zip(right_form).enumerate(){
        let period=left.len()/gcd(left.len(),right.len())*right.len();
        cycles.push((a,b,part,period));
        for tick in 0..period{
          let mut occurrence_counts=vec![0u64;256];
          let lh=sites.program(i,a,part).histogram(left[(tick+i)%left.len()]);
          let rh=sites.program(j,b,part).histogram(right[(tick+j)%right.len()]);
          for (x,&lc) in lh.iter().enumerate(){if lc==0{continue;}
           for (y,&rc) in rh.iter().enumerate(){if rc==0{continue;}
            // Exact contraction of addressed edge pairs by their register value.
            // The edge arrays and binding tree retain every source port.
            let multiplicity=lc*rc;
            let index=16*x+y;source_pair_counts[index]+=multiplicity;
            occurrence_counts[index]+=multiplicity;
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
          let occurrence_mobius=frame_transport_pair_interactions(
              &mobius_pair_interactions(&occurrence_counts));
          let mobius_terms=occurrence_mobius.into_iter().enumerate()
              .filter_map(|(index,coefficient)|(coefficient!=0).then_some((index,coefficient)))
              .collect::<Vec<_>>();
          if !mobius_terms.is_empty(){
              source_occurrences.push(SourceOccurrence{left_form:a,right_form:b,component:part,tick,
                  joint_counts:occurrence_counts.iter().enumerate()
                      .filter_map(|(index,&count)|(count!=0).then_some((index,count))).collect(),
                  mobius_terms});
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
    let mobius_pair_interactions=mobius_pair_interactions(&source_pair_counts);
    let framed_interactions=frame_transport_pair_interactions(&mobius_pair_interactions);
    let mut occurrence_sum=[0i128;256];
    for occurrence in &source_occurrences{
        for &(index,coefficient) in &occurrence.mobius_terms{occurrence_sum[index]+=coefficient;}
    }
    assert_eq!(occurrence_sum.as_slice(),framed_interactions.as_slice(),
        "source occurrence Möbius tensors do not sum to the pair tensor");
    // Integer reconstruction remains the authoritative exact check. The live
    // polynomial evaluator is also exercised on each represented corner.
    for (state,&count) in source_pair_counts.iter().enumerate().filter(|(_,count)|**count!=0).take(1){
        let (left_rank,left_point)=tetrahedral_subset_coordinates(state/16);
        let (right_rank,right_point)=tetrahedral_subset_coordinates(state%16);
        let corner=core::array::from_fn(|bit|{
            let (rank,point)=if bit<4{(right_rank,right_point)}else{(left_rank,left_point)};
            (rank as f64+dot(point,vertex(FOUR[bit%4])))/4.0
        });
        if framed_interactions.iter().map(|c|c.unsigned_abs()).sum::<u128>()<=1u128<<53{
            assert_eq!(joint_response_polynomial(&framed_interactions,&corner),count as f64);
        }
    }
    let reconstructed=reconstruct_pair_interactions(&framed_interactions);
    assert!(reconstructed.iter().zip(&source_pair_counts).all(|(&a,&b)|a==b as i128),
        "SIXTEEN_3 pair Möbius reconstruction changed the source relation");
    let source_pair_counts=reconstructed.into_iter().map(|x|
        u64::try_from(x).expect("reconstructed pair multiplicity is nonnegative and bounded"))
        .collect::<Vec<_>>();
    for (index,&count) in source_pair_counts.iter().enumerate(){
        for (member,branch_count) in branch_counts.iter_mut().enumerate(){
        if source_families[index]&(1<<member)!=0{*branch_count+=count;}
        }
    }
    // Push the signed joint terms directly through the shell-family map.
    // A coefficient at K contributes at every containing support S. Keep
    // cancellation exact until the complete associative sum has returned.
    let mut joint_channels=[0i128;16];
    let mut grouped_channels=[[0i128;16];3];
    let mut channel_terms:Vec<Vec<(usize,i128)>>=vec![Vec::new();16];
    for (support,&coefficient) in framed_interactions.iter().enumerate(){
        if coefficient==0{continue;}
        let mut contribution=[0i128;16];
        let complement=255^support;let mut extra=complement;
        loop{
            let family=source_families[support|extra];
            for (member,total) in joint_channels.iter_mut().enumerate(){
                if family&(1<<member)!=0{
                    *total+=coefficient;grouped_channels[support%3][member]+=coefficient;
                    contribution[member]+=coefficient;
                }
            }
            if extra==0{break;}
            extra=(extra-1)&complement;
        }
        for (member,amount) in contribution.into_iter().enumerate(){
            if amount!=0{channel_terms[member].push((support,amount));}
        }
    }
    for member in 0..16{
        let [a,b,c]=grouped_channels.map(|group|group[member]);
        assert_eq!((a+b)+c,a+(b+c),"signed shell grouping changed channel");
        assert_eq!(a+(b+c),joint_channels[member],"grouped shell return lost a joint term");
    }
    let joint_channels=joint_channels.map(|count|u64::try_from(count)
        .expect("complete signed shell contraction must be nonnegative"));
    assert_eq!(joint_channels,branch_counts,
        "Möbius shell pushforward differs from reconstructed source contraction");
    let branch_counts=joint_channels;
    let channels=branch_counts.iter().enumerate().filter_map(|(member,&count)|{
        if count==0{return None;}
        let (a,b)=(member/4,member%4);
        let positive=(a&1)+(b&1);let negative=(a>>1)+(b>>1);
        let cross=protein_orbital::shell_cross_glut(&[a as u8],&[b as u8]);
        Some(Channel{left:a,right:b,count,positive,negative,
            joint_terms:channel_terms[member].clone(),
            stiffness:count as f64*(positive+negative) as f64,
            preferred:negative as f64,strain:count as f64*cross as f64})
    }).collect();
    let mut source_mechanical_moments=[0.0;3];
    for (state,&count) in source_pair_counts.iter().enumerate(){
        for mask in [state/16,state%16]{
            let g=NativeGlut::from_state(BelnapSet::from_mask(mask as u8).unwrap());
            let span=(g.positive+g.negative) as f64;let n=g.negative as f64;
            for (target,value) in source_mechanical_moments.iter_mut().zip([span,n*span,n*span*span]){
                *target+=count as f64*value;
            }
        }
    }
    Pair{left:i,right:j,counts,cycles,selected_counts,maximizers,information_state,
        source_pair_counts,source_mechanical_moments,source_occurrences,
        mobius_pair_interactions:mobius_pair_interactions.iter().enumerate()
            .filter_map(|(i,&c)|(c!=0).then_some((i,c))).collect(),
        orbital_maximizers,orbital_expansions,
        channels,information_gains,stiffness,preferred,strain}
}

fn initial_winding(paths:&[Vec<Vec<u8>>])->[i128;3]{
    let mut carried=[0i128;3];
    let mut clock=0i128;
    for form in paths{for path in form{for &state in path{
        clock+=1;
        for v in FOUR{if state&(1<<(v as u8))!=0{
            let (a,b)=v.to_wh2();let a=if a{-1}else{1};let b=if b{-1}else{1};
            for (target,value) in carried.iter_mut().zip([a,b,a*b]){*target+=clock*value;}
        }}
    }}}
    carried
}
fn winding_direction(winding:[i128;3])->(f64,f64){
    let [x,y,_]=winding;
    if x==0&&y==0{return (1.0,0.0);}
    let (x,y)=(x as f64,y as f64);let radius=libm::sqrt(x*x+y*y);
    (x/radius,y/radius)
}

/// Contract the complete signed pair Möbius tensors through the three native
/// join/meet orders and the affine FOUR character frame in one incidence-dual
/// pass. This is the Rust image of Lean oneShotPairContraction.
fn contract_pair_tensors(pairs:&[Pair],source:&[[i128;3]],paths:&[Vec<Vec<Vec<u8>>>],sites:&SiteCarrier)
    ->Result<(Vec<[i128;3]>,serde_json::Value),String>{
    let n=source.len();let expected=n*(n-1)/2;
    if pairs.len()!=expected{return Err(format!("whole peptide tensor has {} pairs; expected {expected}",pairs.len()));}
    let mut winding=source.to_vec();let(mut cells,mut observations)=(0usize,0u128);
    let(mut occurrence_count,mut occurrence_states,mut occurrence_terms)=(0usize,0usize,0usize);
    let mut occurrence_contractions:BTreeMap<(usize,usize,usize,usize,usize,usize),[i128;3]>=BTreeMap::new();
    // Lean wholeChainContraction: compose every source pair quadratic at
    // each incident residue, retaining exact integer stiffness/moment sums.
    let(mut chain_stiffness,mut chain_first)=(vec![0i128;n],vec![0i128;n]);
    for pair in pairs{
        if pair.left>=n||pair.right>=n||pair.left>=pair.right||pair.source_pair_counts.len()!=256{
            return Err("invalid source-addressed peptide pair tensor".into());
        }
        let mut spectrum=[0i128;256];
        for &(index,coefficient) in &pair.mobius_pair_interactions{spectrum[index]=coefficient;}
        if reconstruct_pair_interactions(&spectrum).iter().zip(&pair.source_pair_counts)
            .any(|(&a,&b)|a!=b as i128){
            return Err(format!("source pair tensor did not close at residues {} and {}",pair.left+1,pair.right+1));
        }
        let mut pair_winding=[0i128;3];
        let mut occurrence_total=[0u64;256];
        for occurrence in &pair.source_occurrences{
            let mut occurrence_spectrum=[0i128;256];
            for &(index,coefficient) in &occurrence.mobius_terms{
                occurrence_spectrum[index]=coefficient;
            }
            let distribution=reconstruct_pair_interactions(&occurrence_spectrum);
            let mut occurrence_source=[0i128;256];
            for &(state,count) in &occurrence.joint_counts{
                occurrence_source[state]=count as i128;
            }
            if distribution!=occurrence_source{
                return Err(format!("source occurrence Möbius tensor failed at pair ({},{}) form ({},{}) component {} tick {}",
                    pair.left+1,pair.right+1,occurrence.left_form,occurrence.right_form,
                    occurrence.component,occurrence.tick));
            }
            let mut event_winding=[0i128;3];
            occurrence_count+=1;occurrence_terms+=occurrence.mobius_terms.len();
            for (state,&count) in distribution.iter().enumerate(){
                if count==0{continue;}
                occurrence_states+=1;
                occurrence_total[state]+=u64::try_from(count)
                    .map_err(|_|"source occurrence count is outside the native register")?;
                let mass=count;
                let(mut stiffness,mut first)=(0i128,0i128);
                for mask in [state/16,state%16]{
                    let positive=((mask>>1)&1)+((mask>>3)&1);
                    let negative=((mask>>2)&1)+((mask>>3)&1);
                    let span=(positive+negative) as i128;
                    stiffness+=span;first+=negative as i128*span;
                }
                for residue in [pair.left,pair.right]{
                    chain_stiffness[residue]+=mass*stiffness;
                    chain_first[residue]+=mass*first;
                }
                let radial=pair_radial_numerator(state);
                for (target,component) in event_winding.iter_mut()
                    .zip(pair_context_contraction((state/16) as u8,(state%16) as u8)){
                    *target+=mass*radial*component;
                }
            }
            for axis in 0..3{pair_winding[axis]+=event_winding[axis];}
            occurrence_contractions.insert((pair.left,pair.right,occurrence.left_form,
                occurrence.right_form,occurrence.component,occurrence.tick),event_winding);
        }
        if occurrence_total.as_slice()!=pair.source_pair_counts.as_slice(){
            return Err(format!("source occurrence fibres do not close to pair ({},{})",
                pair.left+1,pair.right+1));
        }
        for &count in &pair.source_pair_counts{
            if count!=0{cells+=1;observations+=count as u128;}
        }
        let _=pair_winding;
    }
    // Rejoin the pair fibres into complete source events. Each event fixes one
    // form at every residue, one shared component, and one global tick. Every
    // one of the 190 pair tensors is then contracted against that same event
    // before its 20 residue ports return together.
    fn enumerate_forms(paths:&[Vec<Vec<Vec<u8>>>],at:usize,current:&mut Vec<usize>,out:&mut Vec<Vec<usize>>){
        if at==paths.len(){out.push(current.clone());return;}
        for form in 0..paths[at].len(){current.push(form);enumerate_forms(paths,at+1,current,out);current.pop();}
    }
    let mut form_assignments=Vec::new();
    enumerate_forms(paths,0,&mut Vec::new(),&mut form_assignments);
    let mut source_events=Vec::new();let mut event_pair_uses=0usize;
    let mut joint_graph_events=Vec::new();
    let mut occurrence_event_uses:BTreeMap<(usize,usize,usize,usize,usize,usize),usize>=BTreeMap::new();
    let mut occurrence_ordinals:BTreeMap<(usize,usize,usize,usize,usize,usize),usize>=BTreeMap::new();
    for pair in pairs{
        for (ordinal,occurrence) in pair.source_occurrences.iter().enumerate(){
            occurrence_ordinals.insert((pair.left,pair.right,occurrence.left_form,
                occurrence.right_form,occurrence.component,occurrence.tick),ordinal);
        }
    }
    for forms in &form_assignments{
        let components=forms.iter().enumerate().map(|(i,&form)|paths[i][form].len())
            .min().unwrap_or(0);
        for component in 0..components{
            let joint_sequence=sites.joint_sequence_event(forms,component)?;
            let lengths=forms.iter().enumerate().map(|(i,&form)|paths[i][form][component].len())
                .collect::<Vec<_>>();
            let mut global_period=1usize;
            for &length in &lengths{
                if length==0{return Err("empty source path in shared-event tensor".into());}
                global_period=global_period.checked_mul(length/gcd(global_period,length))
                    .ok_or_else(||"shared source-event period overflow".to_string())?;
            }
            for tick in 0..global_period{
                let seeds=forms.iter().enumerate().map(|(i,&form)|
                    paths[i][form][component][(tick+i)%lengths[i]]).collect::<Vec<_>>();
                joint_graph_events.push(serde_json::json!({"forms":forms,"component":component,
                    "tick":tick,"ports":sites.joint_event_ports(forms,component,&seeds)?,
                    "shared_sequence_return":joint_sequence}));
                let mut event_winding=vec![[0i128;3];n];
                let mut pair_occurrences=Vec::with_capacity(expected);
                for left in 0..n{for right in left+1..n{
                    let pair_period=lengths[left]/gcd(lengths[left],lengths[right])*lengths[right];
                    let address=(left,right,forms[left],forms[right],component,tick%pair_period);
                    let ordinal=occurrence_ordinals.get(&address).copied().ok_or_else(||
                        format!("shared source event lacks pair occurrence ({left},{right}) at tick {tick}"))?;
                    pair_occurrences.push(ordinal);
                    if let Some(contribution)=occurrence_contractions.get(&address){
                        for axis in 0..3{
                            event_winding[left][axis]+=contribution[axis];
                            event_winding[right][axis]-=contribution[axis];
                        }
                        *occurrence_event_uses.entry(address).or_default()+=1;
                    }
                    event_pair_uses+=1;
                }}
                for residue in 0..n{for axis in 0..3{
                    winding[residue][axis]+=event_winding[residue][axis];
                }}
                source_events.push(serde_json::json!([forms,component,tick,event_winding,pair_occurrences]));
            }
        }
    }
    if occurrence_contractions.keys().any(|address|occurrence_event_uses.get(address).copied().unwrap_or(0)==0){
        return Err("a source occurrence tensor did not rejoin a complete chain event".into());
    }
    // The whole-chain radial carrier is the exact pair-tensor minimizer
    // first/stiffness at each residue. Its source-defined return is idempotent.
    let radial_map=|incoming:&[(i128,i128)]| -> Vec<(i128,i128)> {
        chain_first.iter().zip(&chain_stiffness).zip(incoming)
            .map(|((&first,&stiffness),&q)|if stiffness==0{q}else{(first,stiffness)})
            .collect()
    };
    let radial_input=winding.iter().map(|w|(w[0],1i128)).collect::<Vec<_>>();
    let radial_fixed_point=radial_map(&radial_input);
    let radial_return=radial_map(&radial_fixed_point);
    let radial_fixed=radial_return==radial_fixed_point;
    if !radial_fixed{return Err("whole-chain radial tensor failed idempotent return".into());}
    Ok((winding.clone(),serde_json::json!({"operator":"Lean NativePairFoldOperator one-shot over source-addressed signed Möbius tensors: reconstruct each occurrence, apply its three SIXTEEN_3 join-meet boundaries and FOUR wedge, rejoin every pair fibre at the same whole-chain form/component/tick event, then contract all 20 residue ports together",
        "residue_tensors":n,"pair_tensors":pairs.len(),"expected_pair_tensors":expected,
        "source_occurrence_tensors":occurrence_count,"source_occurrence_nonzero_joint_cells":occurrence_states,
        "source_occurrence_signed_terms":occurrence_terms,"occurrence_reconstruction_closed":true,
        "occurrence_pair_fibres_closed":true,"shared_residue_port_contractions":pairs.len(),
        "global_source_event_count":source_events.len(),"global_event_pair_tensor_uses":event_pair_uses,
        "all_source_occurrences_rejoined":true,"source_initial_windings":source,
        "shared_source_event_readout":source_events,
        "joint_source_graph_events":joint_graph_events,
        "nonzero_joint_cells":cells,"joint_observations":observations,
        "source_pair_tensors_closed":true,"contraction_calls":1,
        "residue_character_windings":winding.clone(),
        "whole_chain_radial_contraction":{"composition":"sum over all incident source pair tensors; exact lattice first moment / stiffness",
            "residue_values":radial_fixed_point.iter().map(|&(first,stiffness)|serde_json::json!({"numerator":first,"denominator":stiffness})).collect::<Vec<_>>(),
            "idempotent_return_closed":radial_fixed,"feeds_spatial_backbone":false}})))
}
fn backbone(torsions:&[f64])->Vec<Point>{
    backbone_from_directions(&torsions.iter().map(|&a|(libm::cos(a),libm::sin(a))).collect::<Vec<_>>())
}
fn backbone_from_directions(directions:&[(f64,f64)])->Vec<Point>{
    let mut points=Vec::with_capacity(directions.len());
    // The unit bond is NativeGlut({B}).qStar; the bend is the FOUR Gram ratio.
    let bond=NativeGlut::from_state(BelnapSet::singleton(B4::B)).negative as f64;
    let cosine=dot(vertex(B4::N),vertex(B4::T))/dot(vertex(B4::N),vertex(B4::N));
    let sine=libm::sqrt(1.0-cosine*cosine);
    for i in 0..directions.len(){
        let next=match i{
            0=>(0.0,0.0,0.0),1=>(bond,0.0,0.0),
            2=>(bond-bond*cosine,bond*sine,0.0),
            _=>{
                let (a,b,c)=(points[i-3],points[i-2],points[i-1]);
                let e=unit(sub(c,b));let normal=unit(cross(sub(b,a),e));let tangent=cross(normal,e);
                let (cos,sin)=directions[i];
                add(c,scale(add(scale(e,-cosine),scale(add(scale(tangent,cos),
                    scale(normal,sin)),sine)),bond))
            }
        };points.push(next);
    }points
}
const SPATIAL_OPERATOR:&str="joint-support-centroids/algebraic-preparation-v2";

/// Affine FOUR subset readout. Empty support retains the frame origin and
/// its zero rank; all other supports use their own normalized character sum.
fn support_centroid(sites:&[Point;4],mask:usize)->(Point,[f64;4]){
    let rank=mask.count_ones() as f64;
    let weights=core::array::from_fn(|v|if rank==0.0{0.25}
        else if mask&(1<<v)!=0{1.0/rank}else{0.0});
    let point=sites.iter().zip(weights).fold((0.0,0.0,0.0),|s,(&p,w)|add(s,scale(p,w)));
    (point,weights)
}

fn support_stiffness(state:usize)->(f64,f64,f64){
    let mut out=(0.0,0.0,0.0);
    for mask in [state/16,state%16]{
        let g=NativeGlut::from_state(BelnapSet::from_mask(mask as u8).unwrap());
        let span=(g.positive+g.negative) as f64;let n=g.negative as f64;
        out.0+=span;out.1+=n*span;out.2+=n*span*span;
    }
    out
}

/// Numerator of Lean's radialNest one-step value at common denominator 840,
/// the lcm of the nonzero pair stiffnesses 1 through 8.
fn pair_radial_numerator(state:usize)->i128{
    let (mut stiffness,mut first)=(0i128,0i128);
    for mask in [state/16,state%16]{
        let positive=((mask>>1)&1)+((mask>>3)&1);
        let negative=((mask>>2)&1)+((mask>>3)&1);
        let span=(positive+negative) as i128;
        stiffness+=span;
        first+=negative as i128*span;
    }
    if stiffness==0{0}else{first*(840/stiffness)}
}

/// The spatial identification is explicit: the native scalar response acts
/// on the distance of the two retained support-subset centroids. Every joint
/// source state is evaluated before summation, with the frame pullback intact.
fn support_response<const GRADIENT:bool>(left:&[Point;4],right:&[Point;4],counts:&[u64])
    ->(f64,[[Point;4];2]){
    let mut energy=0.0;let mut gradient=[[(0.0,0.0,0.0);4];2];
    for (state,&count) in counts.iter().enumerate(){
        if count==0{continue;}
        let (a,wa)=support_centroid(left,state/16);
        let (b,wb)=support_centroid(right,state%16);
        let delta=sub(a,b);let distance=norm(delta);
        let (k,first,second)=support_stiffness(state);
        energy+=count as f64*(k*distance*distance-2.0*first*distance+second)/2.0;
        if GRADIENT&&distance>0.0{
            let force=scale(delta,count as f64*(k*distance-first)/distance);
            for v in 0..4{
                gradient[0][v]=add(gradient[0][v],scale(force,wa[v]));
                gradient[1][v]=sub(gradient[1][v],scale(force,wb[v]));
            }
        }
    }
    (energy,gradient)
}
fn evaluate<const WITH_GRADIENT:bool>(points:&[Point],pairs:&[Pair])->(f64,Vec<Point>){
    let frames=Frames::new(points);
    let mut energy=0.0;
    let mut gradient=if WITH_GRADIENT{vec![[(0.0,0.0,0.0);4];points.len()]}else{Vec::new()};
    for pair in pairs{
        let (i,j)=(pair.left,pair.right);
        let (source_energy,source_gradient)=support_response::<WITH_GRADIENT>(
            &frames.sites[i],&frames.sites[j],&pair.source_pair_counts);
        energy+=source_energy;
        if WITH_GRADIENT{
            for v in 0..4{
                gradient[i][v]=add(gradient[i][v],source_gradient[0][v]);
                gradient[j][v]=add(gradient[j][v],source_gradient[1][v]);
            }
        }
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
/// Difference of the same quadratic potential without subtracting two
/// large total energies. Factor each difference of squares before summing.
fn energy_change(before:&[Point],after:&[Point],pairs:&[Pair])->f64{
    let old=Frames::new(before);let new=Frames::new(after);
    let mut total=0.0;let mut correction=0.0;
    for pair in pairs{
        for (state,&count) in pair.source_pair_counts.iter().enumerate(){
            if count==0{continue;}
            let delta=|frames:&Frames|sub(
                support_centroid(&frames.sites[pair.left],state/16).0,
                support_centroid(&frames.sites[pair.right],state%16).0);
            let old_delta=delta(&old);let new_delta=delta(&new);
            let sum=norm(old_delta)+norm(new_delta);
            let change=if sum>0.0{dot(sub(new_delta,old_delta),add(new_delta,old_delta))/sum}else{0.0};
            let (k,first,_)=support_stiffness(state);
            let term=count as f64*change*(k*sum/2.0-first)-correction;
            let next=total+term;correction=(next-total)-term;total=next;
        }
        for c in &pair.channels{
        let old_delta=sub(old.sites[pair.left][c.left],old.sites[pair.right][c.right]);
        let new_delta=sub(new.sites[pair.left][c.left],new.sites[pair.right][c.right]);
        let old_distance=norm(old_delta);let new_distance=norm(new_delta);
        let sum=old_distance+new_distance;
        let distance_change=if sum>0.0{
            dot(sub(new_delta,old_delta),add(new_delta,old_delta))/sum
        }else{0.0};
        let term=0.5*c.stiffness*distance_change*(sum-2.0*c.preferred)-correction;
        let next=total+term;correction=(next-total)-term;total=next;
    }}
    total
}
fn energy_gradient(points:&[Point],pairs:&[Pair])->(f64,Vec<Point>){
    evaluate::<true>(points,pairs)
}
/// Preference re-entry at fixed geometry, factored before summation. This
/// keeps the same coupled work observable when its two total energies round
/// to the same value near a conditional-bank fixed point.
fn preference_energy_change(points:&[Point],before:&[Pair],after:&[Pair])->f64{
    let frames=Frames::new(points);let mut total=0.0;let mut correction=0.0;
    for (old,new) in before.iter().zip(after){
        for (a,b) in old.channels.iter().zip(&new.channels){
            let distance=norm(sub(frames.sites[old.left][a.left],frames.sites[old.right][a.right]));
            let term=0.5*a.stiffness*(b.preferred-a.preferred)*
                (b.preferred+a.preferred-2.0*distance)-correction;
            let next=total+term;correction=(next-total)-term;total=next;
        }
    }
    total
}
fn energy(points:&[Point],pairs:&[Pair])->f64{
    evaluate::<false>(points,pairs).0
}
fn one_shot_semantic_state(points:&[Point],torsions:&[f64],bank:&WorkBank,pairs:&[Pair],
    current:f64,phase:usize,controller:&EnsembleController,sic:&EnsembleSic)->serde_json::Value{
    serde_json::json!({"positions":points,"torsions":torsions,"bank":bank.record(),
        "pair_channels":pairs.iter().map(|pair|serde_json::json!({
            "residues":[pair.left,pair.right],"channels":pair.channels.iter().map(|channel|
                serde_json::json!({"source":[channel.left,channel.right],"count":channel.count,
                    "stiffness":channel.stiffness,"preferred":channel.preferred,
                    "strain":channel.strain,"signed_terms":channel.joint_terms})).collect::<Vec<_>>()
        })).collect::<Vec<_>>(),"deformation":current,"phase":phase,
        "source_continuation":controller.record(),
        "sic_geometry_coordinates":sic.algebraic_geometry.as_ref()
            .map(|geometry|geometry.record()["coordinates"].clone()).unwrap_or(serde_json::Value::Null)})
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
    algebraic_geometry:Option<algebraic::Geometry>,
    whole_carrier_fixed_point:Option<serde_json::Value>,
}
impl EnsembleSic {
    fn new(enabled:bool)->Self{
        Self{frame:crate::sic::wh::ququart_frame(),returns:Vec::new(),signed_energy_drift:0.0,enabled,
            algebraic_geometry:None,whole_carrier_fixed_point:None}
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
        let algebraic_return=if let Some(geometry)=&mut self.algebraic_geometry{
            let parameters=history.last().filter(|e|e["kind"]=="self-transform")
                .map(|e|serde_json::from_value::<Vec<f64>>(e["half_angle_parameters"].clone())
                    .map_err(|_|String::from("executing algebraic geometry lacks rotation parameters"))).transpose()?;
            Some(geometry.reenter(parameters.as_deref(),torsions,bank)?)
        }else{None};
        let fields:Vec<_>=if let Some((_,fields,_))=&algebraic_return{fields.clone()}
            else{returned.recovered.iter().skip(1).flat_map(|z|[z.re,z.im]).collect()};
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
        let mut executing=vec![Complex::new(1.0,0.0)];
        executing.extend(fields.chunks(2).map(|part|Complex::new(part[0],*part.get(1).unwrap_or(&0.0))));
        let record=serde_json::json!({"source_amplitudes":values(&source),
            "recovered_amplitudes":if algebraic_return.is_some(){values(&executing)}else{values(&returned.recovered)},
            "canonical_binary64_recovered_amplitudes":values(&returned.recovered),
            "binary64_certificate_scope":"canonical binary64 source/return observation; algebraic executing geometry has its own enclosing masses and coordinates",
            "algebraic_shared_anchor_return":algebraic_return.as_ref().map(|(_,_,e)|e),
            "executing_geometry_source":if algebraic_return.is_some(){"retained algebraic geometry through exact-radical WH projector analysis and dual synthesis"}else{"binary64 canonical SIC return"},
            "shared_anchor":returned.anchor,
            "gauge":[returned.gauge.re,returned.gauge.im],
            "frames":returned.frames.iter().map(|f|serde_json::json!({
                "source_indices":f.indices,"effect_masses":values(&f.masses),
                "signed_gram_residuals":values(&f.signed_gram_residuals)})).collect::<Vec<_>>(),
            "signed_amplitude_residuals":values(&returned.signed_amplitude_residuals),
            "full_gram_error_upper_bound":returned.full_gram_bound,
            "local_gram_residual":returned.local_residual,"anchor_residual":returned.anchor_residual,
            "roundoff_tolerance":returned.tolerance,"signed_energy_drift":drift,
            "signed_energy_drift_scope":if algebraic_return.is_some(){"executing algebraic SIC readout minus represented trial, including geometry readout difference"}else{"binary64 SIC return minus represented input"},
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
            "algebraic_implementation":"Vox/src/protein_native_algebraic.rs and protein_native_sic_field.rs",
            "canonical_projectors":self.frame.projectors().iter().map(|p|
                p.values().iter().map(|z|[z.re,z.im]).collect::<Vec<_>>()).collect::<Vec<_>>(),
            "record_encoding":protein_records::ENCODING,
            "returns":self.returns,"next_force_reads_recovered_coordinates":true,
            "algebraic_geometry":self.algebraic_geometry.as_ref().map(|g|g.record()),
            "whole_carrier_fixed_point":self.whole_carrier_fixed_point,
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
/// Act on the retained coordinate carrier rather than rebuilding its baseline.
fn apply_torsion_action(points:&[Point],before:&[f64],after:&[f64])->Vec<Point>{
    let parameters:Vec<_>=before.iter().zip(after).map(|(a,b)|libm::tan((b-a)/2.0)).collect();
    apply_half_angle_action(points,&parameters)
}
/// Exact rational coefficients for the represented binary parameter. These
/// describe the rotation independently of its rounded coordinate evaluation.
fn exact_rotation_coefficients(t:f64)->serde_json::Value{
    use num_bigint::BigInt;
    assert!(t.is_finite());
    let bits=t.to_bits();let exponent=((bits>>52)&0x7ff) as i32;
    let fraction=bits&((1u64<<52)-1);
    let mantissa=if exponent==0{fraction}else{fraction|(1u64<<52)};
    let power=if exponent==0{-1074}else{exponent-1023-52};
    let mut numerator=BigInt::from(mantissa);let mut denominator=BigInt::from(1u8);
    if power>=0{numerator<<=power as usize;}else{denominator<<=(-power) as usize;}
    if bits>>63!=0{numerator=-numerator;}
    let nn=&numerator*&numerator;let dd=&denominator*&denominator;
    let common=&dd+&nn;
    serde_json::json!({"parameter_numerator":numerator.to_string(),
        "parameter_denominator":denominator.to_string(),
        "cos_numerator":(&dd-&nn).to_string(),
        "sin_numerator":(BigInt::from(2u8)*&numerator*&denominator).to_string(),
        "common_denominator":common.to_string(),
        "scope":"exact rational rotation of represented parameter; coordinates remain rounded"})
}
fn apply_half_angle_action(points:&[Point],parameters:&[f64])->Vec<Point>{
    let mut returned=points.to_vec();
    for axis in 3..returned.len(){
        let t=parameters[axis];if t==0.0{continue;}
        let pivot=returned[axis-1];let direction=unit(sub(pivot,returned[axis-2]));
        let denominator=1.0+t*t;
        let c=(1.0-t*t)/denominator;let s=2.0*t/denominator;
        for point in &mut returned[axis..]{
            let v=sub(*point,pivot);
            *point=add(pivot,add(scale(v,c),add(scale(cross(direction,v),s),
                scale(direction,dot(direction,v)*(1.0-c)))));
        }
    }
    returned
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
    torques:Vec<f64>,current_torques:Vec<f64>,release_gradients:Vec<f64>,
    torsion_gates:Vec<Vec<usize>>,release_gates:Vec<Vec<usize>>,
    active_torsions:Vec<bool>,active_releases:Vec<bool>,maximum:f64,release_enabled:bool,
    frontier_active:bool,
    algebraic_geometry:Option<algebraic::Geometry>,
}
impl SelfDescription {
    fn read(points:&[Point],pairs:&[Pair],bank:&WorkBank,gates:&[SourceGate],release_enabled:bool,
        prior:Option<&serde_json::Value>)->Self{
        let (_,gradient)=energy_gradient(points,pairs);
        let mut torques=torsion_gradient(points,&gradient);
        let current_torques=torques.clone();
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
        for (i,&g) in current_torques.iter().enumerate(){if active_torsions[i]{maximum=maximum.max((quantum*g).abs());}}
        for (i,&g) in release_gradients.iter().enumerate(){if active_releases[i]{
            maximum=maximum.max(bank.cells[i].negative as f64*g);
        }}
        Self{torques,current_torques,release_gradients,torsion_gates,release_gates,active_torsions,active_releases,maximum,release_enabled,frontier_active,algebraic_geometry:None}
    }
    fn record(&self,input_sic_return:Option<usize>,binding:usize)->serde_json::Value{
        serde_json::json!({"torques":self.torques,"current_torques":self.current_torques,
            "torsion_direction_pairing":self.current_torques.iter().zip(&self.torques).map(|(a,b)|a*b).sum::<f64>(),
            "release_gradients":self.release_gradients,
            "source_binding":binding,
            "active_torsions":self.active_torsions,"active_releases":self.active_releases,
            "frontier_active":self.frontier_active,
            "maximum_normalized_gradient":self.maximum,"input_sic_return":input_sic_return,
            "release_enabled":self.release_enabled,
            "generator":if self.frontier_active{"current and retained frontier gradients jointly drive native-quarter-turn torsions and release centers"}
                else{"simultaneous native-quarter-turn torsions and negative-frame-normalized retained centers"}})
    }
}

#[derive(Clone)]
struct EnsembleController {
    carried:phase_word::Continuation<usize>,gates:Vec<SourceGate>,models:Vec<String>,word:String,
    bindings:Vec<(Vec<Vec<usize>>,Vec<Vec<usize>>)>,failed_attempts:Vec<String>,
}
impl EnsembleController {
    fn new(words:&[Vec<Vec<ProgramSpec<'_>>>])->Result<Self,String>{
        let mut carried=phase_word::Continuation::default();
        carried.advance_with("⊢⊙∈",&0,|_,_|Ok(()))?;
        Ok(Self{carried,gates:source_gates(words)?,models:Vec::new(),word:"⊢⊙∈".into(),bindings:Vec::new(),failed_attempts:Vec::new()})
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
        let held_parent=self.carried.clone();
        // One parent frame remains live across all re-entries. Every inner
        // return deposits its description address back into that same frame.
        let body="∈⊙⊤⊥⊞∋⊙";
        self.carried.advance_with(body,&revision,|mark,_|{
            if mark=='⊤'{performed=action()?;}Ok(())
        })?;
        model["performed"]=serde_json::json!(performed);
        if performed{
            self.word.push_str(body);
            self.models.push(protein_records::encode(&model));
        }else{
            let evidence=protein_records::encode(&model);
            if !self.failed_attempts.contains(&evidence){self.failed_attempts.push(evidence); }
            self.carried=held_parent;
        }
        Ok(performed)
    }
    fn record(&self)->serde_json::Value{
        let read=self.carried.readout();
        serde_json::json!({"schema":"native-ensemble-self-description/v1",
            "source_gates":self.gates.iter().map(|g|serde_json::json!({
                "address":g.address,"opcode_index":g.opcode,"enclosing_depth":g.depth,"word":g.word})).collect::<Vec<_>>(),
            "record_encoding":protein_records::ENCODING,
            "models":self.models,"failed_attempts":self.failed_attempts,"source_bindings":self.bindings.iter().map(|(a,b)|serde_json::json!({
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
    let total_before=*current+bank.remaining_stored();
    // The same joint carrier admits a geometric action with its bank held.
    // A rounded conditional-bank update must not hide that successor when
    // every simultaneous candidate was rejected.
    for couple_bank in [true,false]{
    let mut step=1.0/model.maximum;
    for _ in 0..f64::MANTISSA_DIGITS{
        let parameters:Vec<_>=model.current_torques.iter().enumerate().map(|(i,&g)|
            if model.active_torsions[i]{-step*g}else{0.0}).collect();
        let candidate:Vec<_>=torsions.iter().zip(&parameters).map(|(&a,&t)|a+2.0*libm::atan(t)).collect();
        let trial=apply_half_angle_action(points,&parameters);
        let trial_frames=Frames::new(&trial);
        let amounts:Vec<_>=bank.cells.iter().enumerate().map(|(i,c)|{
            if !couple_bank||!model.active_releases[i]{return 0.0;}
            let p=c.positive as f64;let n=c.negative as f64;let count=c.multiplicity as f64;
            let [left,right,a,b]=c.source;
            let distance=norm(sub(trial_frames.sites[left][a],trial_frames.sites[right][b]));
            let center=c.conditional_preference(distance);
            let available=(count*p*((p+n)*center*center/n-p*n)/2.0).max(0.0).min(c.available());
            c.available()-available
        }).collect();
        let mut next_bank=bank.clone();
        let has_release=amounts.iter().any(|&v|v>0.0);
        let payment=if has_release{next_bank.pay_allocated(&amounts)}else{None};
        if has_release&&payment.is_none(){step/=2.0;continue;}
        let mut next_pairs=pairs.clone();
        let changed=if couple_bank{reenter_preferences(&mut next_pairs,&next_bank)}else{0};
        let score=energy(&trial,&next_pairs);let total_after=score+next_bank.remaining_stored();
        let signed_deformation_change=energy_change(points,&trial,&next_pairs)
            +if changed==0{0.0}else{preference_energy_change(points,pairs,&next_pairs)};
        let deformation_support=signed_deformation_change<0.0;
        let retained_work_support=next_bank.remaining_stored()<bank.remaining_stored();
        let actual_released=bank.remaining_stored()-next_bank.remaining_stored();
        // Both support observations remain independent. The executed joint
        // mechanical action must also satisfy its own native work balance:
        // a positive release alone does not pay an arbitrarily larger rise.
        let coupled_work_change=signed_deformation_change-actual_released;
        if total_after.is_finite()&&(deformation_support||retained_work_support)
            &&coupled_work_change<=0.0{
            let algebraic_joint_work=if let Some(geometry)=&model.algebraic_geometry{
                let bound=geometry.joint_work(&parameters,pairs,&next_pairs,bank,&next_bank)?;
                if !bound.negative(){step/=2.0;continue;}
                Some(bound.record())
            }else{None};
            let dissipated=actual_released-signed_deformation_change;
            history.push(serde_json::json!({"kind":"self-transform","model":revision,"step":step,
                "before":*current,"after":score,"total_before":total_before,"total_after":total_after,
                "deformation_support":deformation_support,"retained_work_support":retained_work_support,
                "signed_deformation_change":signed_deformation_change,"signed_retained_work_change":
                    next_bank.remaining_stored()-bank.remaining_stored(),
                "signed_coupled_work_change":coupled_work_change,
                "retained_algebraic_joint_work_bound":algebraic_joint_work,
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
                "enclosing_frame_depth":2,"torque_and_release_from_retained_description":true,
                "executed_torsion_direction":"current derivative; accumulated frontier retained in model",
                "executed_torques":model.current_torques,
                "release_action":"one-shot native conditional quadratic minimizer, clipped to retained bank interval",
                "conditional_bank_applied":couple_bank,
                "half_angle_parameters":parameters,
                "exact_rotation_coefficients":parameters.iter().copied().map(exact_rotation_coefficients).collect::<Vec<_>>(),
                "coordinate_action":"rational half-angle action; angles are readout bookkeeping"}));
            *released+=dissipated;*torsions=candidate;*points=trial;*current=score;
            *bank=next_bank;*pairs=next_pairs;return Ok(true);
        }
        step/=2.0;
    }
    }
    // Resolve small work signs in the actual algebraic operator before
    // declaring its rounded admission boundary held. The bank stays resident.
    for exponent in 24..44{
        let step=libm::pow(2.0,-(exponent as f64))/model.maximum;
        let parameters=model.current_torques.iter().enumerate().map(|(i,&g)|
            if model.active_torsions[i]{-step*g}else{0.0}).collect::<Vec<_>>();
        let trial=apply_half_angle_action(points,&parameters);
        if trial==*points{continue;}
        let rotation_work=if let Some(g)=&model.algebraic_geometry{g.rotation_work(&parameters,pairs)?}
            else{algebraic::rotation_work(points,&parameters,pairs)?};
        if !rotation_work.negative(){continue;}
        let work=algebraic::work(points,&trial,pairs)?;
        if model.algebraic_geometry.is_none()&&!work.negative(){continue;}
        let signed_work=if model.algebraic_geometry.is_some(){rotation_work.midpoint()}else{work.midpoint()};
        let score=energy(&trial,pairs);
        let candidate=torsions.iter().zip(&parameters).map(|(&a,&t)|a+2.0*libm::atan(t)).collect();
        history.push(serde_json::json!({"kind":"self-transform","model":revision,
            "before":*current,"after":score,"step":step,
            "deformation_support":true,"retained_work_support":false,
            "actual_released_glut":0.0,"signed_deformation_change":signed_work,
            "signed_coupled_work_change":signed_work,"deformation_released":-signed_work,
            "algebraic_work_bound":work.record(),"conditional_bank_applied":false,
            "algebraic_rotation_work_bound":rotation_work.record(),
            "work_admission_source":if model.algebraic_geometry.is_some(){"retained algebraic geometry"}else{"represented rational snapshots"},
            "algebraic_rotation_scope":"exact half-angle action on the represented rational input; output evaluated before coordinate rounding",
            "algebraic_input_geometry":model.algebraic_geometry.as_ref().map(|g|g.record()),
            "algebraic_rotation_output":if let Some(g)=&model.algebraic_geometry{g.rotation_output(&parameters)?}
                else{algebraic::rotation_output(points,&parameters)?},
            "algebraic_input_coordinates":points,"algebraic_output_coordinates":trial,
            "algebraic_input_preferences":pairs.iter().map(|p|p.channels.iter().map(|c|c.preferred).collect::<Vec<_>>()).collect::<Vec<_>>(),
            "direction_source":"strictly negative outward algebraic work interval on the retained joint carrier",
            "half_angle_parameters":parameters,
            "exact_rotation_coefficients":parameters.iter().copied().map(exact_rotation_coefficients).collect::<Vec<_>>(),
            "executed_opcode":"⊤","enclosing_frame_depth":2}));
        *released-=signed_work;*points=trial;*torsions=candidate;*current=score;
        return Ok(true);
    }
    // Search the finite native FOUR orbit before holding the carrier. These
    // candidates use the existing quarter-turn action and the same potential.
    for axis in 3..torsions.len(){
        if model.torsion_gates[axis].is_empty(){continue;}
        for turn in 1..FOUR.len(){
            let mut candidate=torsions.clone();candidate[axis]+=quantum*turn as f64;
            let trial=apply_torsion_action(points,torsions,&candidate);let score=energy(&trial,pairs);
            if score.is_finite()&&score<*current{
                let parameters=torsions.iter().zip(&candidate).map(|(a,b)|libm::tan((b-a)/2.0)).collect::<Vec<_>>();
                let exact=if let Some(g)=&model.algebraic_geometry{
                    let w=g.joint_work(&parameters,pairs,pairs,bank,bank)?;
                    if !w.negative(){continue;}Some(w.record())
                }else{None};
                history.push(serde_json::json!({"kind":"self-transform","model":revision,
                    "step":quantum*turn as f64,"before":*current,"after":score,
                    "deformation_support":true,"retained_work_support":false,
                    "direction_source":"finite FOUR torsion orbit","axis":axis,"turn":turn,
                    "half_angle_parameters":parameters,"retained_algebraic_joint_work_bound":exact,
                    "actual_released_glut":0.0,"deformation_released":*current-score,
                    "executed_opcode":"⊤","enclosing_frame_depth":2}));
                *released+=*current-score;*torsions=candidate;*points=trial;*current=score;
                return Ok(true);
            }
        }
    }
    // Retain coupled native actions: an energy-lowering pair need not have
    // either energy-lowering singleton as a prefix.
    for left in 3..torsions.len(){for right in left+1..torsions.len(){
        if model.torsion_gates[left].is_empty()||model.torsion_gates[right].is_empty(){continue;}
        for a in 1..FOUR.len(){for b in 1..FOUR.len(){
            let mut candidate=torsions.clone();
            candidate[left]+=quantum*a as f64;candidate[right]+=quantum*b as f64;
            let trial=apply_torsion_action(points,torsions,&candidate);let score=energy(&trial,pairs);
            if score.is_finite()&&score<*current{
                let parameters=torsions.iter().zip(&candidate).map(|(a,b)|libm::tan((b-a)/2.0)).collect::<Vec<_>>();
                let exact=if let Some(g)=&model.algebraic_geometry{
                    let w=g.joint_work(&parameters,pairs,pairs,bank,bank)?;
                    if !w.negative(){continue;}Some(w.record())
                }else{None};
                history.push(serde_json::json!({"kind":"self-transform","model":revision,
                    "step":quantum,"before":*current,"after":score,
                    "half_angle_parameters":parameters,"retained_algebraic_joint_work_bound":exact,
                    "deformation_support":true,"retained_work_support":false,
                    "direction_source":"joint FOUR torsion orbit","axes":[left,right],"turns":[a,b],
                    "actual_released_glut":0.0,"deformation_released":*current-score,
                    "executed_opcode":"⊤","enclosing_frame_depth":2}));
                *released+=*current-score;*torsions=candidate;*points=trial;*current=score;
                return Ok(true);
            }
        }}
    }}
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
    if steps==Some(0){return Err("observation interval requires at least one step".into());}
    fold_programs_with_dynamics(words,true,true,true,true,steps,None,branch,false)
}
pub fn prepare_whole_programs(words:&[Vec<Vec<ProgramSpec<'_>>>],branch:Option<&RouteBranch>)->Result<serde_json::Value,String>{
    fold_programs_with_dynamics(words,true,true,true,true,Some(0),None,branch,false)
}
pub fn fold_programs_one_shot(words:&[Vec<Vec<ProgramSpec<'_>>>],branch:Option<&RouteBranch>)->Result<serde_json::Value,String>{
    fold_programs_with_dynamics(words,true,true,true,true,Some(0),None,branch,true)
}

/// Continue the saved physical and banked state without another preparation
/// measurement or replay of its physical transformations.
pub fn resume_programs(words:&[Vec<Vec<ProgramSpec<'_>>>],held:&serde_json::Value,steps:Option<usize>)->Result<serde_json::Value,String>{
    resume_programs_with_branch(words,held,steps,None)
}
fn resume_programs_with_branch(words:&[Vec<Vec<ProgramSpec<'_>>>],held:&serde_json::Value,steps:Option<usize>,
    branch:Option<&RouteBranch>)->Result<serde_json::Value,String>{
    fold_programs_with_dynamics(words,true,true,true,true,steps,Some(held),branch,false)
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
    fold_programs_with_dynamics(words,enable_work,feedback,coherence,false,None,None,None,false)
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
    controller:&mut EnsembleController,initial:f64,_ticks:usize)->Result<usize,String>{
    use serde_json::json;
    if held["schema"]!="native-glut-protein-fold/v10"{
        return Err("resume requires an open native v10 observation return".into());
    }
    if held["pair_interaction_operator"]["spatial_operator"]!=SPATIAL_OPERATOR{
        return Err("held spatial operator differs; prepare a fresh source carrier".into());
    }
    if held["source_paths"]!=serde_json::to_value(paths).map_err(|e|e.to_string())?
        ||held["site_graphs"]!=sites.record()||held_number(&held["initial_deformation"])?!=initial{
        return Err("held ensemble preparation differs from the supplied source".into());
    }
    let next=held["continuation"]["next_clock"].as_u64().and_then(|n|usize::try_from(n).ok())
        .ok_or("invalid held ensemble clock")?;
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
    sic.algebraic_geometry=Some(if held["sic_carrier"]["algebraic_geometry"].is_object(){
        algebraic::Geometry::restore(&held["sic_carrier"]["algebraic_geometry"])?
    }else{algebraic::Geometry::snapshot(points)});
    if held["sic_carrier"]["algebraic_geometry"].is_object()
        &&sic.algebraic_geometry.as_ref().unwrap().readout()!=*points{
        return Err("retained algebraic geometry and executing readout disagree".into());
    }
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
    if !held_controller["failed_attempts"].is_null(){
        controller.failed_attempts=held_records(&held_controller["failed_attempts"])?;
    }
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

fn fold_programs_with_dynamics(words:&[Vec<Vec<ProgramSpec<'_>>>],enable_work:bool,feedback:bool,coherence:bool,self_generated:bool,observation_steps:Option<usize>,resume:Option<&serde_json::Value>,branch:Option<&RouteBranch>,one_shot:bool)->Result<serde_json::Value,String>{
    if observation_steps==Some(0)&&resume.is_some(){return Err("observation interval requires at least one step".into());}
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
    let pair_addresses=(0..paths.len()).flat_map(|i|(i+1..paths.len()).map(move |j|(i,j))).collect::<Vec<_>>();
    #[cfg(feature="hosted")]
    let pair_chunks={
    let worker_count=std::thread::available_parallelism().map(usize::from).unwrap_or(1).min(pair_addresses.len().max(1));
    let chunk_size=pair_addresses.len().div_ceil(worker_count).max(1);
    let pair_chunks=std::thread::scope(|scope|{
        let paths_ref=&paths;let sites_ref=&sites;let transfers_ref=&transfers;let branch_ref=&branch;
        let handles=pair_addresses.chunks(chunk_size).map(|chunk|{
            let (paths,sites,transfers,branch)=(paths_ref,sites_ref,transfers_ref,branch_ref);
            scope.spawn(move||chunk.iter().map(|&(i,j)|
                prepare_pair_with_branch(i,j,paths,sites,transfers,*branch)).collect::<Vec<_>>())
        }).collect::<Vec<_>>();
        handles.into_iter().map(|handle|handle.join().expect("native pair worker panicked")).collect::<Vec<_>>()
    });
    pair_chunks
    };
    #[cfg(not(feature="hosted"))]
    let pair_chunks=vec![pair_addresses.iter().map(|&(i,j)|
        prepare_pair_with_branch(i,j,&paths,&sites,&transfers,branch)).collect::<Vec<_>>()];
    let mut pairs=pair_chunks.into_iter().flatten().collect::<Vec<_>>();
    let source_windings=paths.iter().map(|p|initial_winding(p)).collect::<Vec<_>>();
    let (fold_windings,one_shot_contraction)=if one_shot{
        let (windings,record)=contract_pair_tensors(&pairs,&source_windings,&paths,&sites)?;(windings,Some(record))
    }else{(source_windings.clone(),None)};
    let directions=fold_windings.iter().map(|&w|winding_direction(w)).collect::<Vec<_>>();
    // One-shot folding constructs the backbone in the exact FOUR algebraic
    // carrier. The binary64 positions are only its terminal coordinate readout.
    let one_shot_geometry=if one_shot{Some(algebraic::Geometry::prepared(&fold_windings)?)}else{None};
    let mut points=if let Some(geometry)=&one_shot_geometry{geometry.readout()}
        else{backbone_from_directions(&directions)};
    let mut torsions=fold_windings.iter().map(|w|libm::atan2(w[1] as f64,w[0] as f64)).collect::<Vec<_>>();
    let initial=energy(&points,&pairs);
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
    // The whole algebraic compiler consumes the source winding and complete
    // pair carrier directly. At its zero-action preparation boundary, do not
    // build the legacy interval Frenet walk first: long chains can widen those
    // dependent intervals before the whole source equations are assembled.
    // The later certified root return executes its own shared-anchor SIC.
    if one_shot&&coherence{
        sic.algebraic_geometry=one_shot_geometry.clone();
    }else if self_generated&&coherence{
        sic.algebraic_geometry=Some(algebraic::Geometry::prepared(&source_windings)?);
    }
    let mut controller=EnsembleController::new(words)?;
    let source_occurrence_snapshot=pairs.iter().map(|pair|pair.source_occurrences.clone()).collect::<Vec<_>>();
    let source_pair_mobius_snapshot=pairs.iter().map(|pair|pair.mobius_pair_interactions.clone()).collect::<Vec<_>>();
    let source_bound_pre_sic=if one_shot&&coherence{Some(one_shot_semantic_state(
        &points,&torsions,&bank,&pairs,current,phase,&controller,&sic))}else{None};
    let start=if let Some(held)=resume{
        restore_ensemble(held,&paths,&sites,&mut pairs,&mut bank,&mut torsions,
            &mut points,&mut current,&mut history,&mut family,&mut accepted,
            &mut deformation_released,&mut sic,&mut controller,initial,ticks)?
    }else{
        if !one_shot||coherence{sic.reenter(&mut points,&mut torsions,&mut bank,&mut pairs,&mut current,&mut history,feedback)?;}
        0
    };
    if let Some(source_bound)=source_bound_pre_sic{
        let first_return=one_shot_semantic_state(&points,&torsions,&bank,&pairs,current,phase,&controller,&sic);
        sic.reenter(&mut points,&mut torsions,&mut bank,&mut pairs,&mut current,&mut history,feedback)?;
        let second_return=one_shot_semantic_state(&points,&torsions,&bank,&pairs,current,phase,&controller,&sic);
        let source_tensors_preserved=source_occurrence_snapshot==pairs.iter()
            .map(|pair|pair.source_occurrences.clone()).collect::<Vec<_>>()
            &&source_pair_mobius_snapshot==pairs.iter()
                .map(|pair|pair.mobius_pair_interactions.clone()).collect::<Vec<_>>();
        sic.whole_carrier_fixed_point=Some(serde_json::json!({
            "verified":source_bound==first_return&&first_return==second_return&&source_tensors_preserved,
            "source_bound_pre_return_equals_first_return":source_bound==first_return,
            "first_return_equals_second_return":first_return==second_return,
            "source_occurrence_tensors_preserved":source_tensors_preserved,
            "comparison":"positions, torsions, live bank, pair channels, source continuation, phase and FOUR geometry coordinates",
            "source_occurrence_address_fields":["residue_pair","left_form","right_form","component","tick"],
            "sic_returns_compared":2}));
    }
    let mut nesting_probes=resume.and_then(|h|h["nesting_probes"].as_array()).cloned().unwrap_or_default();
    let mut clock_completed=start;
    let quantum=2.0*core::f64::consts::PI/FOUR.len() as f64;
    // The source-word size supplies an observation interval, not a bound
    // on the number of re-entries required by a self-generated successor.
    let requested_end=start.saturating_add(observation_steps.unwrap_or(ticks));
    let end=if self_generated{requested_end}else{requested_end.min(ticks)};
    for tick in start..end{
        clock_completed=tick+1;
        if self_generated{
            let prior=controller.models.last().map(|record|protein_records::decode(record)).transpose()?;
            let mut description=SelfDescription::read(&points,&pairs,&bank,&controller.gates,enable_work,prior.as_ref());
            description.algebraic_geometry=sic.algebraic_geometry.clone();
            // A zero generator holds the executing parent unchanged. Appending
            // a failed evaluation would itself change the retained ancestry.
            if description.maximum==0.0{clock_completed=tick;stalled=true;break;}
            if !description.maximum.is_finite(){
                return Err("nonfinite full-carrier successor generator".into());
            }
            let revision=controller.models.len();
            let binding=controller.bind(&description);
            let model=description.record(sic.returns.len().checked_sub(1),binding);
            // Pre-nest probe of the named mechanical action on retained state.
            // This is separate from the enclosing controller/SIC operator.
            let mut probe_pairs=pairs.clone();let mut probe_bank=bank.clone();
            let mut probe_torsions=torsions.clone();let mut probe_points=points.clone();
            let mut probe_energy=current;let mut probe_history=History::default();let mut probe_release=0.0;
            let mut probe_controller=controller.clone();
            let probe_parent_before=probe_controller.record();
            let mut probe_moved=probe_controller.execute(model.clone(),||self_transform(&description,
                &mut probe_pairs,&mut probe_bank,&mut probe_torsions,&mut probe_points,
                &mut probe_energy,&mut probe_history,&mut probe_release,revision))?;
            let mut probe_sic=EnsembleSic::new(sic.enabled);
            probe_sic.returns=sic.returns.clone();probe_sic.signed_energy_drift=sic.signed_energy_drift;
            probe_sic.algebraic_geometry=sic.algebraic_geometry.clone();
            if !probe_moved&&description.frontier_active{
                let mut current_description=SelfDescription::read(&probe_points,&probe_pairs,&probe_bank,
                    &probe_controller.gates,enable_work,None);
                current_description.algebraic_geometry=probe_sic.algebraic_geometry.clone();
                let binding=probe_controller.bind(&current_description);
                let mut model=current_description.record(probe_sic.returns.len().checked_sub(1),binding);
                model["retained_frontier_attempt"]=description.record(probe_sic.returns.len().checked_sub(1),binding);
                model["direction_source"]=serde_json::json!("current-state gradient after retained frontier rejection");
                probe_moved=probe_controller.execute(model,||self_transform(&current_description,&mut probe_pairs,&mut probe_bank,
                    &mut probe_torsions,&mut probe_points,&mut probe_energy,&mut probe_history,&mut probe_release,revision))?;
            }
            if probe_moved{
                let event=probe_history.last().unwrap();
                probe_controller.attach_transition_support(serde_json::json!({
                    "deformation_support":event["deformation_support"],
                    "retained_work_support":event["retained_work_support"]}))?;
                probe_sic.reenter(&mut probe_points,&mut probe_torsions,&mut probe_bank,
                    &mut probe_pairs,&mut probe_energy,&mut probe_history,feedback)?;
            }
            let coordinate_residual=points.iter().zip(&probe_points).map(|(&a,&b)|norm(sub(a,b))).fold(0.0f64,f64::max);
            let torsion_residual=torsions.iter().zip(&probe_torsions).map(|(a,b)|(a-b).abs()).fold(0.0f64,f64::max);
            let nesting_probe=serde_json::json!({
                "outer_action":"parent EVALT/self_transform, transition support deposit, then SIC re-entry",
                "parent_equal":probe_parent_before==probe_controller.record(),
                "sic_returns_equal":sic.returns==probe_sic.returns,
                "inner_state":"coordinates, torsions, source-bound pairs and release bank",
                "coordinate_residual":coordinate_residual,"torsion_residual":torsion_residual,
                "bank_equal":bank.record()==probe_bank.record(),"admitted_successor":probe_moved,
                "prediction":if probe_moved{"attracted-or-open"}else{"held mechanical action; full nesting class unresolved"},
                "basin_certified":false,"includes_enclosing_controller_and_sic":true,
                "scope":"primary and fallback action, parent and SIC; outer observation journal compared separately"});
            if !nesting_probes.contains(&nesting_probe){nesting_probes.push(nesting_probe);}
            let mut moved=controller.execute(model,||self_transform(&description,&mut pairs,&mut bank,
                &mut torsions,&mut points,&mut current,&mut history,&mut deformation_released,revision))?;
            if !moved&&description.frontier_active{
                let mut current_description=SelfDescription::read(&points,&pairs,&bank,
                    &controller.gates,enable_work,None);
                current_description.algebraic_geometry=sic.algebraic_geometry.clone();
                let binding=controller.bind(&current_description);
                let mut model=current_description.record(sic.returns.len().checked_sub(1),binding);
                model["retained_frontier_attempt"]=description.record(sic.returns.len().checked_sub(1),binding);
                model["direction_source"]=serde_json::json!("current-state gradient after retained frontier rejection");
                moved=controller.execute(model,||self_transform(&current_description,&mut pairs,&mut bank,
                    &mut torsions,&mut points,&mut current,&mut history,&mut deformation_released,revision))?;
            }
            if moved!=probe_moved{return Err("pre-nest composed action disagrees with execution".into());}
            if !moved{
                if points!=probe_points||torsions!=probe_torsions||bank.record()!=probe_bank.record()
                    ||controller.record()!=probe_controller.record()||sic.returns!=probe_sic.returns{
                    return Err("held pre-nest return differs from actual retained state".into());
                }
                clock_completed=tick;stalled=true;break;
            }
            let event=history.last().unwrap();
            controller.attach_transition_support(serde_json::json!({
                "deformation_support":event["deformation_support"],
                "retained_work_support":event["retained_work_support"]}))?;
            accepted.push(serde_json::json!({"tick":tick,"model":revision,"kind":"self-transform",
                "step":event["step"],"before":event["before"],"after":event["after"]}));
            sic.reenter(&mut points,&mut torsions,&mut bank,&mut pairs,&mut current,&mut history,feedback)?;
            if points!=probe_points||torsions!=probe_torsions||bank.record()!=probe_bank.record()
                ||controller.record()!=probe_controller.record()||sic.returns!=probe_sic.returns{
                return Err("pre-nest parent/SIC return differs from actual executing state".into());
            }
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
    let returned_readout=serde_json::json!({"kind":"returned-readout","history_length":history.len(),
        "deformation":current,"glut_work":bank.spent(),"torsions":torsions,"positions":points});
    if family.last()!=Some(&returned_readout){family.push(returned_readout);}
    let mut max_bond_error=0.0f64;let mut max_gram_error=0.0f64;
    let cosine=dot(vertex(B4::N),vertex(B4::T))/dot(vertex(B4::N),vertex(B4::N));
    for i in 1..points.len(){max_bond_error=max_bond_error.max((norm(sub(points[i],points[i-1]))-1.0).abs());}
    for i in 1..points.len().saturating_sub(1){
        let gram=dot(unit(sub(points[i-1],points[i])),unit(sub(points[i+1],points[i])));
        max_gram_error=max_gram_error.max((gram-cosine).abs());
    }
    let spatial=Frames::new(&points);
    let modulus=paths.len();
    let pair_records:Vec<_>=pairs.iter().map(|p|serde_json::json!({"residues":[p.left,p.right],
        "modulus_addresses":[p.left%modulus,p.right%modulus],
        "native_join_continuations":(0..8).map(|bit|{
            let support=1usize<<bit;
            let next=join_pair_source(&p.source_pair_counts,support);
            let mut observed=[0u64;16];
            for (state,&count) in next.iter().enumerate(){
                for (channel,total) in observed.iter_mut().enumerate(){
                    if transfers[state].family&(1<<channel)!=0{*total+=count;}
                }
            }
            serde_json::json!({"join_support":support,
                "joint_source_counts":next.iter().enumerate().filter_map(|(state,&count)|
                    (count!=0).then_some((state,count))).collect::<Vec<_>>(),
                "native_shell_channel_counts":observed,
                "scope":"one native information join on retained source; no configured route additions"})
        }).collect::<Vec<_>>(),
        "contexts":["join_i","meet_i","join_t","meet_t","join_f","meet_f"],
        "state_counts":p.counts,"form_cycles":p.cycles,
        "information_maximum_counts":p.selected_counts,"maximizing_context_counts":p.maximizers,
        "information_state":p.information_state,"information_gains":p.information_gains,
        "source_register_pair_counts":p.source_pair_counts,
        // Compact lossless carrier: [left form, right form, component, tick,
        // [[joint support address, signed Möbius coefficient], ...]]. The
        // native contraction checks every exact inverse; the Python fold
        // batches those exact inversions before applying the geometry.
        "source_occurrence_encoding":protein_records::ENCODING,
        "source_occurrence_mobius_interactions":encode_source_occurrences(&p.source_occurrences),
        "mobius_joint_interactions":p.mobius_pair_interactions.iter().map(|&(index,coefficient)|{
            let left=index>>4;let right=index&15;
            let (left_rank,left_frame)=tetrahedral_subset_coordinates(left);
            let (right_rank,right_frame)=tetrahedral_subset_coordinates(right);
            serde_json::json!({"left_support_subset":left,"right_support_subset":right,
                "coefficient":coefficient,"left_tetrahedral_subset":{"cardinality":left_rank,"coordinate":left_frame},
                "right_tetrahedral_subset":{"cardinality":right_rank,"coordinate":right_frame}})
        }).collect::<Vec<_>>(),
        "orbital_maximizing_counts":p.orbital_maximizers,"orbital_expanded_observations":p.orbital_expansions,
        "source_mechanical_moments":p.source_mechanical_moments,
        "source_marginal_readout":{"stiffness":p.stiffness,"preferred":p.preferred,"retained_strain":p.strain},
        "retained_strain":p.channels.iter().map(|c|c.strain).sum::<f64>(),
        "spatial_channels":p.channels.iter().map(|c|serde_json::json!({
            "occupancies":[c.left,c.right],"count":c.count,
            "signed_joint_terms":c.joint_terms,
            "spin_counts":[c.positive,c.negative],"stiffness":c.stiffness,
            "preferred":c.preferred,"retained_strain":c.strain,
            "returned_distance":norm(sub(spatial.sites[p.left][c.left],spatial.sites[p.right][c.right]))})).collect::<Vec<_>>(),
        "returned_distance":norm(sub(points[p.left],points[p.right]))})).collect();
    let (_, final_gradient)=energy_gradient(&points,&pairs);
    let final_torque=torsion_gradient(&points,&final_gradient);
    let final_gradient_max=final_torque.iter().fold(0.0f64,|a,&b|a.max(b.abs()));
    let bank_fixed_point_rows=bank.cells.iter().map(|c|{
        let [i,j,a,b]=c.source;
        let distance=norm(sub(spatial.sites[i][a],spatial.sites[j][b]));
        let q=c.preference();let next=c.conditional_preference(distance);
        serde_json::json!({"source":c.source,"positive":c.positive,"negative":c.negative,
            "distance":distance,"preference":q,"conditional_return":next,
            "signed_preference_residual":next-q,
            "interior_equation_residual":(c.positive+c.negative) as f64*q-c.negative as f64*distance})
    }).collect::<Vec<_>>();
    let bank_fixed_point_max=bank_fixed_point_rows.iter().map(|r|
        r["signed_preference_residual"].as_f64().unwrap().abs()).fold(0.0f64,f64::max);
    let torsion_derivative_probe=if stalled{
        let rebuilt=backbone(&torsions);
        let drift=points.iter().zip(&rebuilt).map(|(&a,&b)|norm(sub(a,b))).fold(0.0f64,f64::max);
        let rows=(3..torsions.len()).map(|axis|{
            let samples=[1e-4,1e-5,1e-6].into_iter().map(|h|{
                let mut high=torsions.clone();let mut low=torsions.clone();
                high[axis]+=h;low[axis]-=h;
                let derivative=energy_change(&apply_torsion_action(&points,&torsions,&low),
                    &apply_torsion_action(&points,&torsions,&high),&pairs)/(2.0*h);
                serde_json::json!({"step":h,"central_derivative":derivative})
            }).collect::<Vec<_>>();
            serde_json::json!({"axis":axis,"analytic":final_torque[axis],"samples":samples})
        }).collect::<Vec<_>>();
        serde_json::json!({"returned_vs_rebuilt_max_distance":drift,"axes":rows})
    }else{serde_json::Value::Null};
    let samples=core::array::from_fn(|mask|{
        let g=NativeGlut::from_state(BelnapSet::from_mask(mask as u8).unwrap());
        Response{energy:g.retained_strain(),radial_derivative:0.0}
    });
    let spectrum=Spectrum::from_samples(samples);
    let continuation=serde_json::json!({"boundary":"open retained native conformation",
            "reason":if stalled{if self_generated{"no admitted coupled successor in retained description"}
                else{"no representable torsion descent in this frame"}}else if self_generated||clock_completed<ticks{
                    "observation-step interval ended"}else{"source-clock interval ended"},
            "observation_step_limit":observation_steps,"interval_start_clock":start,
            "completed_clock_steps":clock_completed,"next_clock":clock_completed,
            "next_work_phase":if self_generated{None}else{Some(phase)},
            "source_phase_interval":if self_generated{None}else{Some(phase_interval)},
            "retained":["source_paths","source_continuations","site_graphs","orbital_filling","orbital_transfer","spatial_sites","glut_work","ensemble_reentry","sic_carrier","self_description","work_attempts","transition_history","conformation_family","torsions","positions","pairs","accepted_steps"]});
    let reentered_preferences=history.reentered_preferences;
    let history=history.into_records();
    let mut returned=serde_json::json!({"schema":if self_generated{"native-glut-protein-fold/v10"}else{"native-glut-protein-fold/v9"},
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
        "residue_modulus":{"modulus":modulus,
            "coupled_fixed_point_equations":{
                "torsion_equation":"pullback of the complete joint spatial derivative to every source-admitted bond rotation equals zero",
                "bank_equation":"q=clamp(sqrt(p*n^2/(p+n)), q, n*d/(p+n)) on every retained bank cell",
                "distance_equation":"d>=0 and d^2=dot(left_site-right_site,left_site-right_site)",
                "max_torsion_residual":final_gradient_max,
                "max_conditional_bank_residual":bank_fixed_point_max,
                "bank_rows":bank_fixed_point_rows,
                "coordinate_evaluation":"binary64; zero residual alone does not certify exact algebraic closure",
                "full_carrier_obligation":"source, bank, parent, continuation and SIC must return together"},
            "addressing":"residue indices in Z/nZ; every unordered native pair retained once",
            "addresses":(0..modulus).map(|i|i%modulus).collect::<Vec<_>>(),
            "terminal_cut":[modulus.saturating_sub(1),0],
            "paired_residue_count":pairs.len(),
            "coordinate_stationary":{"reached":final_gradient_max<=1e-12,
                "max_torsion_gradient":final_gradient_max,
                "stalled_derivative_probe":torsion_derivative_probe,
                "criterion":"native pair-energy gradient at or below 1e-12"},
            "whole_carrier_fixed_point":sic.whole_carrier_fixed_point.clone().unwrap_or_else(||serde_json::json!({
                "verified":false,"criterion":"apply the complete re-entry operator and compare source, banks, pair channels, phase, ancestry and FOUR/SIC coordinates"}))},
        "pair_interaction_operator":{"source_domain":"256 joint states of two SIXTEEN_3 residues",
            "source_occurrence_address":["left_form","right_form","component","tick"],
            "source_occurrence_operator":"each addressed occurrence retains its signed Möbius tensor and SIXTEEN_3 join/meet corner values; summing occurrence tensors must recover the pair tensor exactly",
            "spatial_operator":SPATIAL_OPERATOR,
            "spatial_identification":"native support quadratic acts on distance between FOUR-frame support centroids; empty support uses frame origin; physical identification under test",
            "decomposition":"eight successive membership differences, four N/T/F/B bits per residue",
            "inverse":"eight membership accumulations",
            "runtime_check":"every Möbius spectrum is reconstructed and compared exactly before channel preparation",
            "frame_map":"each left and right support-subset term is carried as cardinality plus its FOUR tetrahedral character-vertex sum",
            "frame_inverse":"membership(v)=(cardinality+dot(coordinate,vertex(v)))/4",
            "frame_transport_checked_addresses_per_pair":256,
            "frame_transport_consumed_by":"signed Möbius subset-incidence contraction into shell channels",
            "shell_channel_check":"direct signed contraction equals reconstructed source contraction exactly",
            "geometry_consumption":"full reconstructed source distribution acts at support-subset centroids before summation, alongside orbital channels; both return forces through moving frames",
            "shell_composition":"associative native shell fusion adds every cross-shell pair once"},
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
        "native_strain_interaction_coefficients":spectrum.coefficients.iter().map(|r|r.energy).collect::<Vec<_>>()});
    returned["nesting_probes"]=serde_json::json!(nesting_probes);
    if let Some(mut contraction)=one_shot_contraction {
        contraction["contraction_calls"]=serde_json::json!(1);
        contraction["coordinate_count"]=serde_json::json!(points.len());
        contraction["unit_bond_max_error"]=serde_json::json!(max_bond_error);
        contraction["four_gram_max_error"]=serde_json::json!(max_gram_error);
        contraction["all_source_pair_tensors_reconstructed"]=serde_json::json!(true);
        contraction["geometry_closure"]=serde_json::json!("exact retained algebraic FOUR unit-bond and tetrahedral-bend construction");
        contraction["algebraic_geometry"]=sic.algebraic_geometry.as_ref().map(|geometry|geometry.record()).unwrap_or(serde_json::Value::Null);
        contraction["whole_carrier_fixed_point"]=sic.whole_carrier_fixed_point.clone().unwrap_or_else(||serde_json::json!({
            "verified":false,"criterion":"source-bound full-carrier SIC re-entry equality"}));
        contraction["source_pair_and_geometry_closed"]=serde_json::json!(
            contraction["source_pair_tensors_closed"]==true
            &&contraction["pair_tensors"]==contraction["expected_pair_tensors"]
            &&sic.algebraic_geometry.as_ref().is_some_and(|geometry|geometry.record()["coordinates"].as_array().is_some_and(|rows|rows.len()==points.len()))
            &&max_bond_error<=1e-12&&max_gram_error<=1e-12);
        contraction["full_chain_lattice_fold_closed"]=serde_json::json!(false);
        contraction["closure_criterion"]=serde_json::json!("shared source-event contraction and source-bound full-carrier SIC re-entry are checked; the whole-chain algebraic force root remains open until its separate readout closes");
        returned["one_shot_lattice_contraction"]=contraction;
    }
    returned["algebraic_preparation"]=serde_json::json!({
        "source_windings":source_windings.iter().map(|w|w.map(|n|n.to_string())).collect::<Vec<_>>(),
        "direction_equations":"r>0, r^2=X^2+Y^2, cos=X/r, sin=Y/r; X=Y=0 selects (1,0)",
        "backbone_bend":{"cos_numerator":-1,"cos_denominator":3,"sin_squared_numerator":8,"sin_squared_denominator":9},
        "executed_input":if one_shot{"source-pair join/meet FOUR winding normalized in the retained exact algebraic geometry"}else{"source integer winding normalized algebraically; angles are readout bookkeeping"},
        "coordinate_evaluation":"binary64 evaluation of algebraic expressions; exact root carrier not yet implemented"});
    returned["source_mechanics_coupling"]=serde_json::json!({"status":"executed hypothesis",
        "coordinate":"distance between the two support-subset centroids in moving FOUR frames",
        "response":"full reconstructed joint distribution weights native endpoint quadratics before summation, plus existing orbital channels",
        "parameters":"native membership counts and source multiplicities; no fitted coefficient",
        "exact_fold_theorem":false});
    if let Some(held)=resume{
        let fields=["source_paths","source_continuations","site_graphs","orbital_filling",
            "orbital_transfer","spatial_sites","glut_work","ensemble_reentry","sic_carrier",
            "self_description","work_attempts","transition_history","conformation_family",
            "torsions","positions","pairs","accepted_steps","continuation","nesting_probes",
            "algebraic_preparation","pair_interaction_operator"];
        let comparisons=fields.into_iter().map(|field|serde_json::json!({
            "field":field,"equal":held[field]==returned[field]})).collect::<Vec<_>>();
        let equal=comparisons.iter().all(|row|row["equal"]==true);
        returned["finite_precision_nesting_status"]=serde_json::json!({
            "all_compared_executing_fields_fixed":equal,
            "classification":if equal{"fixed under this finite-precision return action"}else{"attracted-or-open"},
            "scope":"listed retained executing fields, including nesting evidence; source execution journal and comparison observations remain outside this equality",
            "basin_certified":false,"exact_algebraic_closure":false});
        returned["reentry_state_comparison"]=serde_json::json!({
            "scope":"listed executing fields of native return; CLI source/execution journal is outside this comparison",
            "field_comparisons":comparisons,"all_listed_fields_equal":equal,
            "exact_algebraic_fixed_point_proved":false});
    }
    Ok(returned)
}

#[cfg(test)]
mod tests{
    use super::*;
    use alloc::string::ToString;
    #[test]
    fn integer_source_winding_prepares_character_directions_without_angle_roundtrip(){
        let paths=vec![vec![vec![0,2,6,8,15]]];
        let winding=initial_winding(&paths);
        let mut independent=[0i128;3];
        for (index,&mask) in paths[0][0].iter().enumerate(){
            for v in FOUR{if mask&(1<<(v as u8))!=0{
                let q=vertex(v);
                for (target,component) in independent.iter_mut().zip([q.0,q.1,q.2]){
                    *target+=(index as i128+1)*component as i128;
                }
            }}
        }
        assert_eq!(winding,independent);
        assert_eq!(winding_direction([0,0,9]),(1.0,0.0));
        assert_eq!(winding_direction([3,4,0]),(0.6,0.8));
        let source=vec![winding;8];
        let directions=source.iter().map(|&w|winding_direction(w)).collect::<Vec<_>>();
        let actual=backbone_from_directions(&directions);
        let angles=source.iter().map(|w|libm::atan2(w[1] as f64,w[0] as f64)).collect::<Vec<_>>();
        for (a,b) in actual.iter().zip(backbone(&angles)){assert!(norm(sub(*a,b))<1e-13);}
        for adjacent in actual.windows(2){assert!((norm(sub(adjacent[1],adjacent[0]))-1.0).abs()<1e-14);}
    }
    #[test]
    fn joint_support_spatial_action_preserves_response_and_frame_derivatives(){
        let points=vec![(0.0,0.0,0.0),(1.0,0.0,0.0),(1.3,0.8,0.1),
            (1.8,1.2,0.9),(2.4,0.5,1.4)];
        let frames=Frames::new(&points);
        let counts=(0..256).map(|s|((s*17+s*s)%7) as u64).collect::<Vec<_>>();
        let (value,g)=support_response::<true>(&frames.sites[1],&frames.sites[4],&counts);
        let mut site_gradient=vec![[(0.0,0.0,0.0);4];points.len()];
        site_gradient[1]=g[0];site_gradient[4]=g[1];
        let analytic=frames.pullback(&points,&site_gradient);
        let mut reconstructed=0.0;
        let spectrum=frame_transport_pair_interactions(&mobius_pair_interactions(&counts));
        // Dual incidence action: sum_S f(S) R(S) = sum_K c(K) sum_{S>=K} R(S).
        for (mask,&coefficient) in spectrum.iter().enumerate(){
            if coefficient==0{continue;}
            let complement=255^mask;let mut extra=complement;
            loop{
                let mut singleton=vec![0;256];singleton[mask|extra]=1;
                reconstructed+=coefficient as f64*support_response::<false>(
                    &frames.sites[1],&frames.sites[4],&singleton).0;
                if extra==0{break;}extra=(extra-1)&complement;
            }
        }
        assert!((value-reconstructed).abs()<1e-8*(1.0+value.abs()));
        let h=1e-5;
        for i in 0..points.len(){for axis in 0..3{
            let mut high=points.clone();let mut low=points.clone();
            let shift=|p:&mut Point,d:f64|match axis{0=>p.0+=d,1=>p.1+=d,_=>p.2+=d};
            shift(&mut high[i],h);shift(&mut low[i],-h);
            let read=|p:&[Point]|{let f=Frames::new(p);
                support_response::<false>(&f.sites[1],&f.sites[4],&counts).0};
            let measured=(read(&high)-read(&low))/(2.0*h);
            let expected=match axis{0=>analytic[i].0,1=>analytic[i].1,_=>analytic[i].2};
            assert!((measured-expected).abs()<1e-4*(1.0+expected.abs()));
        }}
        // A retained correlation witness must act before the shell contraction.
        let mut a=vec![0;256];let mut b=a.clone();a[24]=1;a[30]=1;b[26]=1;b[28]=1;
        assert!((support_response::<false>(&frames.sites[1],&frames.sites[4],&a).0
            -support_response::<false>(&frames.sites[1],&frames.sites[4],&b).0).abs()>1e-6);
    }
    #[test]
    fn every_joint_state_requires_distinction_under_native_join_continuations(){
        let transfers=protein_orbital::table();
        let mut collisions=0;let mut distinguished=0;
        for a in 0..256{for b in a+1..256{
            if transfers[a].family==transfers[b].family{
                collisions+=1;
                let witness=(0..256).find(|&support|
                    transfers[a|support].family!=transfers[b|support].family);
                assert!(witness.is_some(),"merged states lack a differing join future: {a}, {b}");
                distinguished+=1;
            }
        }}
        assert!(collisions>0);assert_eq!(distinguished,collisions);
    }
    #[test]
    fn joint_differential_matches_native_membership_difference_and_frame_pullback(){
        let samples=(0..256).map(|s|((s*17+s*s)%29) as u64).collect::<Vec<_>>();
        let coefficients=mobius_pair_interactions(&samples);
        let z=[0.2,0.3,0.4,0.5,0.6,0.7,0.8,0.9];
        let (_,gradient)=joint_response_differential(&coefficients,&z);
        for bit in 0..8{
            let mut low=z;let mut high=z;low[bit]=0.0;high[bit]=1.0;
            let difference=joint_response_polynomial(&coefficients,&high)-joint_response_polynomial(&coefficients,&low);
            assert!((gradient[bit]-difference).abs()<1e-10);
        }
        // Affine inverse membership=(rank+vertex dot coordinate)/4.
        let frame_gradient=FOUR.iter().enumerate().fold((0.0,0.0,0.0),|g,(i,&v)|
            add(g,scale(vertex(v),gradient[i]/4.0)));
        let h=1e-5;let mut high=z;let mut low=z;
        for i in 0..4{high[i]+=h*vertex(FOUR[i]).0/4.0;low[i]-=h*vertex(FOUR[i]).0/4.0;}
        let numerical=(joint_response_polynomial(&coefficients,&high)-joint_response_polynomial(&coefficients,&low))/(2.0*h);
        assert!((numerical-frame_gradient.0).abs()<1e-6);
    }
    #[test]
    fn source_native_mechanics_distinguishes_shell_invisible_mixtures(){
        let moments=|states:&[usize]|{
            let mut total=[0usize;3];
            for &state in states{for mask in [state/16,state%16]{
                let g=NativeGlut::from_state(BelnapSet::from_mask(mask as u8).unwrap());
                let span=g.positive+g.negative;
                total[0]+=span;total[1]+=g.negative*span;total[2]+=g.negative*span*span;
            }}
            total
        };
        assert_eq!(moments(&[24,30]),[6,10,36]);
        assert_eq!(moments(&[26,28]),[6,9,27]);
    }
    #[test]
    fn joint_polynomial_distinguishes_shell_invisible_correlations(){
        let mut a=vec![0;256];let mut b=vec![0;256];
        a[24]=1;a[30]=1;b[26]=1;b[28]=1;
        let ca=mobius_pair_interactions(&a);let cb=mobius_pair_interactions(&b);
        let corner=core::array::from_fn(|bit|if 24&(1<<bit)!=0{1.0}else{0.0});
        assert_eq!(joint_response_polynomial(&ca,&corner),1.0);
        assert_eq!(joint_response_polynomial(&cb,&corner),0.0);
        for state in 0..256{
            let corner=core::array::from_fn(|bit|if state&(1<<bit)!=0{1.0}else{0.0});
            assert_eq!(joint_response_polynomial(&ca,&corner),a[state] as f64);
            assert_eq!(joint_response_polynomial(&cb,&corner),b[state] as f64);
        }
    }
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
    fn retained_torsion_action_matches_backbone_and_preserves_identity(){
        let before=vec![0.0,0.0,0.0,0.3,-0.7,1.2];
        let mut after=before.clone();after[3]+=0.27;after[5]-=0.41;
        let points=backbone(&before);
        assert_eq!(apply_torsion_action(&points,&before,&before),points);
        let acted=apply_torsion_action(&points,&before,&after);
        for (a,b) in acted.into_iter().zip(backbone(&after)){
            assert!(norm(sub(a,b))<1e-12);
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
        let mut changed_pairs=pairs.clone();
        for p in &mut changed_pairs{for c in &mut p.channels{c.preferred*=0.9;}}
        let mut trial_angles=angles.clone();trial_angles[4]+=0.03;
        let trial=backbone(&trial_angles);
        let factored=energy_change(&points,&trial,&changed_pairs)
            +preference_energy_change(&points,&pairs,&changed_pairs);
        let direct=energy(&trial,&changed_pairs)-energy(&points,&pairs);
        assert!((factored-direct).abs()<1e-10*(1.0+direct.abs()));
        let exact_work=algebraic::work(&points,&trial,&pairs).unwrap();
        let direct_work=energy(&trial,&pairs)-energy(&points,&pairs);
        assert!((exact_work.midpoint()-direct_work).abs()<1e-8*(1.0+direct_work.abs()));
        assert_eq!(exact_work.negative(),direct_work<0.0);
        let reverse=algebraic::work(&trial,&points,&pairs).unwrap();
        assert_ne!(exact_work.negative(),reverse.negative());
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
