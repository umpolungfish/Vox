use vox::membrane_state::MembraneState;
use vox::{morphism_factor as mf, vox as vx};

fn synth_word() -> Vec<char> {
    let mk = |addr: u64, fall: Option<u64>, mn: &str, op: &str| vx::Instruction {
        address: addr,
        fallthrough: fall,
        mnemonic: mn.into(),
        op_str: op.into(),
    };
    let ins = vec![
        mk(0x0, Some(0x2), "cmp", "eax, ebx"),
        mk(0x2, Some(0x4), "jz", "0x10"),
        mk(0x4, Some(0x6), "add", "eax, 1"),
        mk(0x6, Some(0x8), "mov", "ecx, eax"),
        mk(0x8, None, "ret", ""),
    ];
    vx::recompile_function(&ins)
}

fn main() {
    let argv: Vec<String> = std::env::args().collect();
    let word = synth_word();
    let (n_s, bases): (String, Vec<u64>) = if argv.len() >= 3 {
        let n = argv[1].clone();
        let mut bs: Vec<u64> = Vec::new();
        for i in 2..argv.len() {
            for part in argv[i].split(',') {
                if let Ok(b) = part.parse::<u64>() { bs.push(b); }
            }
        }
        if bs.is_empty() { bs.push(2); }
        (n, bs)
    } else {
        ("15".into(), vec![7u64])
    };
    println!("seed word={} verdict={}", vx::glyphs(&word), vx::verdict(&word));
    let n = mf::decimal_to_tape(&n_s).expect("N decimal");
    println!("N bits={}", n.len());
    let rep = MembraneState::period_readout_bases(word, n, &bases);
    println!("{}", rep);
}
