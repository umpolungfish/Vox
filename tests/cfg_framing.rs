use ::vox::vox::{self, Instruction};
use ::vox::vox_decode;
use std::collections::BTreeMap;

fn ins(address:u64, mnemonic:&str, target:Option<u64>, next:Option<u64>)->Instruction {
    Instruction {address, fallthrough:next, mnemonic:mnemonic.into(),
        op_str:target.map(|t|format!("0x{t:x}")).unwrap_or_default()}
}

#[test]
fn syscall_retry_loop_has_a_real_header_and_latch_frame() {
    // Exact instruction bytes of musl _Exit, stripped of relocation/address.
    let bytes=[0x48,0x63,0xff,0xb8,0xe7,0,0,0,0x0f,0x05,
        0xba,0x3c,0,0,0,0x48,0x89,0xd0,0x0f,0x05,0xeb,0xf9];
    let image=vox_decode::Image {segments:vec![(0x1000,bytes.to_vec())]};
    let walk=vox_decode::walk(&image,0x1000,&[]);
    let word=vox::recompile_function(&walk.functions[0].1);
    assert_eq!(vox::glyphs(&word),"⊢⋈⋈⊙⋈∈⋈⊙≺∋");
    assert_eq!(vox::verdict(&word),'T');
}

#[test]
fn backward_shared_tail_is_not_a_loop_and_abort_has_no_continuation() {
    // Both Get*RelBase functions: optional log arm, then shared abort tail.
    let body=vec![ins(0,"jne",Some(30),Some(10)),
        ins(10,"call",Some(100),Some(20)),ins(20,"nop",None,Some(30)),
        ins(30,"call",Some(200),Some(40)),ins(40,"jmp",Some(10),None)];
    let mut functions=vec![(0,body)];
    let symbols=BTreeMap::from([("abort".into(),100)]);
    vox_decode::mark_noreturn(&mut functions,&symbols);
    let word=vox::recompile_function(&functions[0].1);
    assert_eq!(vox::glyphs(&word),"⊢∈≻≺∋≻");
    assert_eq!(vox::verdict(&word),'T');
    // Unknown calls must not be classified as non-returning by name fragments.
    let mut unknown=vec![(0,vec![ins(0,"call",Some(100),Some(10))])];
    vox_decode::mark_noreturn(&mut unknown,&BTreeMap::from([("my_abort_logger".into(),100)]));
    assert_eq!(unknown[0].1[0].fallthrough,Some(10));
}

#[test]
fn sparse_addresses_do_not_invent_fallthrough_and_three_way_joins_keep_arity() {
    let sparse=vec![ins(0,"mov",None,Some(3)),ins(10,"jmp",Some(20),None),ins(20,"ret",None,None)];
    assert_eq!(vox::glyphs(&vox::recompile_function(&sparse)),"⊢⋈");
    let diamond=vec![ins(0,"jne",Some(30),Some(10)),ins(10,"jne",Some(40),Some(20)),
        ins(20,"jmp",Some(50),None),ins(30,"jmp",Some(50),None),
        ins(40,"jmp",Some(50),None),ins(50,"ret",None,None)];
    assert_eq!(vox::compute_merges(&diamond),vec![50,50]);
    assert_eq!(vox::verdict(&vox::recompile_function(&diamond)),'T');
}

#[test]
fn existing_verdicts_remain_strict() {
    assert_eq!(vox::verdict(&"⊢⋈⋈⊙⋈∋⋈⊙≺".chars().collect::<Vec<_>>()),'F');
    assert_eq!(vox::verdict(&"⊢∈≻⊣".chars().collect::<Vec<_>>()),'B');
}

#[test]
fn all_six_membranes_retain_the_four_runtime_functions_without_f_verdicts() {
    for name in ["binomial","lcm","divisor","landau","schutte","shor"] {
        let path=format!("{}/membranes/words/{name}.elf",env!("CARGO_MANIFEST_DIR"));
        let raw=std::fs::read(path).unwrap();
        let loaded=::vox::loader::load(&raw);
        let image=vox_decode::Image {segments:loaded.code};
        let seeds:Vec<_>=loaded.symbols.values().copied().collect();
        let mut walk=vox_decode::walk(&image,loaded.entry,&seeds);
        vox_decode::mark_noreturn(&mut walk.functions,&loaded.symbols);
        for symbol in ["_Exit","_Unwind_GetDataRelBase","_Unwind_GetTextRelBase","_Unwind_Resume"] {
            let addr=loaded.symbols[symbol];
            let body=&walk.functions.iter().find(|(a,_)|*a==addr).expect("runtime function retained").1;
            let word=vox::recompile_function(body);
            assert_eq!(vox::verdict(&word),'T',"{name} {symbol}: {}",vox::glyphs(&word));
            println!("{name} {symbol} 0x{addr:x} T {}",vox::glyphs(&word));
        }
    }
}
