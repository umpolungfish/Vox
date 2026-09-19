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
pub mod lanes;
pub mod genetic;
pub mod protein;
pub mod fold;
pub mod fold3d;
pub mod pyc;
pub mod pyc_table;
pub mod genetic_table;
pub mod x86;
pub mod imasm_module;
pub mod glyph_module;
pub mod imasm_vm;
pub mod morphism_factor;
pub mod factor_operator;
pub mod factorization_31_membrane;
pub mod complete_membrane;
pub mod sieve;
pub mod perfect_membrane;
pub mod divisor_membrane;
pub mod nested_frame;
pub mod meta_router;
pub mod router_object;
pub mod router_store;
pub mod carrier;
pub mod trace_object;
pub mod judge_g;
pub mod router_marks;
pub mod trace_word;
pub mod trace_algebra;
pub mod factor_extract;
pub mod reentry_certificate;
pub mod provenance_envelope;
pub mod producer_provenance;
pub mod dialectic_reentry;
pub mod dialectic_certificate;
pub mod tape_delete;
pub mod reducer_store;
pub mod frame_work;
pub mod loader;
pub mod safetensors;
pub mod membrane_state;
pub mod winding_readout;
pub mod shor_braid;
// pub mod shor_qft;   // bin-only: uses ::vox:: (external-crate path)
// pub mod fde_shor_membrane;  // bin-only: uses std + ::vox