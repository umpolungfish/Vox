//! V⊙x — the control-flow closure auditor, standalone.
//!
//! `vox_decode` is a native x86-64 decoder and ELF reader (no capstone, no
//! pefile). `vox` lifts a decoded instruction stream to a twelve-glyph IMASM
//! word and verdicts whether it closes: T closes, B holds a fork open across a
//! terminal, N never forked, F is ill-typed (a ∋ with no ∈). Both modules are
//! `no_std`+`alloc` and depend on nothing outside this crate, so a consumer
//! links `vox` the way every project links the foundation.
#![no_std]

#[macro_use]
extern crate alloc;

pub mod vox;
pub mod vox_decode;
