//! Lossless immutable observations, separate from the live executing fields.
use alloc::{string::{String,ToString},vec::Vec};
use base64::{Engine,engine::general_purpose::STANDARD};
use serde_json::Value;

pub const ENCODING:&str="zlib-base64-json-records/v1";

pub fn encode_text(text:&str)->String{
    if text.starts_with("z:"){return text.into();}
    let bytes=miniz_oxide::deflate::compress_to_vec_zlib(text.as_bytes(),1);
    let encoded=STANDARD.encode(bytes);
    if encoded.len()+2>=text.len(){text.into()}else{alloc::format!("z:{encoded}")}
}
pub fn encode(value:&Value)->String{encode_text(&value.to_string())}
pub fn decode(text:&str)->Result<Value,String>{
    if let Some(payload)=text.strip_prefix("z:"){
        let compressed=STANDARD.decode(payload).map_err(|e|e.to_string())?;
        let bytes=miniz_oxide::inflate::decompress_to_vec_zlib(&compressed).map_err(|e|e.to_string())?;
        serde_json::from_slice(&bytes).map_err(|e|e.to_string())
    }else{serde_json::from_str(text).map_err(|e|e.to_string())}
}

#[derive(Default)]
pub struct History {
    records:Vec<String>,current:Option<Value>,pub reentered_preferences:u64,
}
impl History {
    fn flush(&mut self){if let Some(value)=self.current.take(){self.records.push(encode(&value));}}
    pub fn push(&mut self,value:Value){
        self.flush();
        self.reentered_preferences+=value["reentered_preferences"].as_u64().unwrap_or(0);
        self.current=Some(value);
    }
    pub fn len(&self)->usize{self.records.len()+usize::from(self.current.is_some())}
    pub fn last(&self)->Option<&Value>{self.current.as_ref()}
    pub fn last_mut(&mut self)->Option<&mut Value>{self.current.as_mut()}
    pub fn from_held(value:&Value,reentered_preferences:u64)->Result<Self,String>{
        let records=value.as_array().ok_or("missing retained transformation history")?.iter()
            .map(|v|if let Some(text)=v.as_str(){encode_text(text)}else{encode(v)}).collect();
        Ok(Self{records,current:None,reentered_preferences})
    }
    pub fn into_records(mut self)->Vec<String>{self.flush();self.records}
}

/// Consume expanded legacy observations one record at a time before executing.
/// The checkpoint's live fields and every decoded observation stay unchanged.
pub fn compact_held(held:&mut Value){
    fn compact(value:&mut Value){
        if let Some(rows)=value.as_array_mut(){
            let source=core::mem::take(rows);
            *rows=source.into_iter().map(|v|Value::String(match v{
                Value::String(text)=>encode_text(&text),other=>encode(&other),
            })).collect();
        }
    }
    compact(&mut held["transition_history"]);
    held["transition_history_encoding"]=Value::String(ENCODING.into());
    for (section,key) in [("sic_carrier","returns"),("self_description","models")]{
        compact(&mut held[section][key]);
        held[section]["record_encoding"]=Value::String(ENCODING.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observations_recover_every_float_bit_and_current_mutation(){
        let value=serde_json::json!({"word":"∈⊙⊤⊥⊞∋⊙","values":
            (0..4096).map(|i|[f64::from_bits(0x3ff0000000000000+i),-0.0]).collect::<Vec<_>>()});
        let encoded=encode(&value);assert!(encoded.starts_with("z:"));
        let restored=decode(&encoded).unwrap();assert_eq!(restored,value);
        for (a,b) in restored["values"].as_array().unwrap().iter().zip(value["values"].as_array().unwrap()){
            for slot in 0..2{assert_eq!(a[slot].as_f64().unwrap().to_bits(),b[slot].as_f64().unwrap().to_bits());}
        }
        let mut history=History::default();history.push(serde_json::json!({"kind":"step","reentered_preferences":3}));
        history.last_mut().unwrap()["return"]=serde_json::json!(15);
        history.push(value.clone());assert_eq!(history.len(),2);assert_eq!(history.reentered_preferences,3);
        let rows=history.into_records();assert_eq!(decode(&rows[0]).unwrap()["return"],15);
        assert_eq!(decode(&rows[1]).unwrap(),value);
        assert!(decode("z:broken").is_err());
    }
}
