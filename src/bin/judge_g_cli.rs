// judge_g_cli — the judge as a GValue-native function agrees, mark for mark,
// with meta_router::judge, and the numerals-as-marks carrier round-trips.
extern crate alloc;
use ::vox::meta_router::{judge, Repr, Judg};
use ::vox::nested_frame::isqrt_u64;
use ::vox::carrier::GValue;
use ::vox::judge_g::{
    encode_u64, decode_u64, repr_value, repr_decode, judge_g, judge_mark, judg_to_mark,
    REPR_TAG_SYM, REPR_TAG_RES, REPR_TAG_MUL, J_F,
};

fn main() {
    // ---- TEST 1: numerals-as-marks round-trip ----
    let xs: [u64; 7] = [0, 1, 2, 255, 256, 106545994355809, u64::MAX];
    let mut rt = true;
    for &x in &xs {
        let m = encode_u64(x);
        if m.len() != 64 { rt = false; }
        if decode_u64(&m) != x { rt = false; }
    }
    println!("TEST1 numeral round-trip (0..u64::MAX) true = {}", rt);

    // ---- TEST 2: carrier round-trip ----
    let v = repr_value(REPR_TAG_SYM, 106545994355809, 10326048);
    let (tag, n, param) = repr_decode(&v).unwrap();
    let car_rt = tag == REPR_TAG_SYM && n == 106545994355809 && param == 10326048;
    println!("TEST2 carrier round-trip (tag,N,param) true = {}", car_rt);

    // ---- TEST 3: judge_g agrees with meta_router::judge, mark for mark ----
    let mut total = 0usize;
    let mut agree = 0usize;
    let mut by_mark = [0usize; 4]; // T B N F
    for n in 2u64..=300 {
        let sym = Repr::Symmetric { a: isqrt_u64(n) + 1 };
        let res = Repr::Residue { k: 1 };
        let mul = Repr::Multiplicative;
        for repr in [&sym, &res, &mul] {
            let (j, _, _) = judge(n, repr);
            let (tag, param) = match repr {
                Repr::Symmetric { a } => (REPR_TAG_SYM, *a),
                Repr::Residue { k } => (REPR_TAG_RES, *k as u64),
                Repr::Multiplicative => (REPR_TAG_MUL, 0),
            };
            let carrier = repr_value(tag, n, param);
            let mark = judge_g(&carrier).mark0().unwrap();
            if mark == judg_to_mark(j) { agree += 1; }
            total += 1;
            match j { Judg::T => by_mark[0]+=1, Judg::B => by_mark[1]+=1, Judg::N => by_mark[2]+=1, Judg::F => by_mark[3]+=1 }
        }
    }
    println!("TEST3 judge_g == meta_router::judge : {}/{} agree", agree, total);
    println!("       marks: T(⊤)={} B(⊞)={} N(⊙)={} F(⊥)={}", by_mark[0], by_mark[1], by_mark[2], by_mark[3]);

    // ---- TEST 4: a malformed carrier (no repr tag) judges F ----
    let bad = repr_value('\u{2299}', 42, 0); // ⊙ is not a repr tag
    let badm = judge_g(&bad).mark0().unwrap();
    println!("TEST4 non-repr tag -> F(⊥) true = {}", badm == J_F);

    // ---- TEST 5: the mark-only verdict matches for a large semiprime ----
    let big = 106545994355809u64;
    let c = repr_value(REPR_TAG_SYM, big, isqrt_u64(big) + 1);
    let m_sym = judge_g(&c).mark0().unwrap();
    let m_mul = judge_g(&repr_value(REPR_TAG_MUL, big, 0)).mark0().unwrap();
    println!("TEST5 big={} sym-mark={} mul-mark={} ({} {})",
        big, m_sym, m_mul,
        judge_mark(REPR_TAG_SYM, big), judge_mark(REPR_TAG_MUL, big));

    let pass = rt && car_rt && agree == total && badm == J_F;
    println!("\nJUDGE-G {}", if pass { "PASS" } else { "FAIL" });
    let _ = GValue::single('⊤');
}
