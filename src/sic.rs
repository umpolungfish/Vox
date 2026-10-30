//! Bind the canonical portable SIC sources into the resident Vox carrier.
use alloc::string::{String, ToString};
#[path="../../G-mOMonadOS/src/sic/frame.rs"] pub mod frame;
#[path="../../G-mOMonadOS/src/sic/wh.rs"] pub mod wh;
#[path="../../G-mOMonadOS/src/sic/carrier.rs"] pub mod carrier;

#[derive(Debug, Clone)]
pub struct SicError(pub String);
impl core::fmt::Display for SicError {
    fn fmt(&self, f:&mut core::fmt::Formatter<'_>)->core::fmt::Result{f.write_str(&self.0)}
}
impl core::error::Error for SicError {}
pub(crate) fn error(message:&str)->SicError{SicError(message.to_string())}
