use core::cmp::Ordering;

use vox::factor_extract::{extract, extract_word, FactorCarrier};
use vox::factorization_31_membrane::{UnboundedResident, WORD};
use vox::morphism_factor::{cmp, divmod, mul, smart_factor, tape_u64, zero};
use vox::router_marks::{run_mark, GStep, RouterG, M_B, M_FIX, M_N, M_T};
use vox::router_object::RouterObject;
use vox::trace_algebra::witness_valid;
use vox::trace_word::{decode_trace, encode_trace, judge_trace};

const N: u64 = 106_545_994_355_809;

fn same_unordered_pair(a: &[char], b: &[char], x: &[char], y: &[char]) -> bool {
    (cmp(a, x) == Ordering::Equal && cmp(b, y) == Ordering::Equal)
        || (cmp(a, y) == Ordering::Equal && cmp(b, x) == Ordering::Equal)
}

fn resident_program_trace() -> Vec<char> {
    let glyphs: Vec<char> = WORD.chars().collect();
    assert_eq!(glyphs.len(), 31);
    let steps: Vec<GStep> = glyphs
        .iter()
        .enumerate()
        .map(|(i, &glyph)| {
            let last = i + 1 == glyphs.len();
            GStep {
                repr: '⋈',
                judgment: if last { M_T } else { M_B },
                recognised: M_T,
                next: if last { M_FIX } else { '⋈' },
                applied_word: vec![glyph],
            }
        })
        .collect();
    encode_trace(&steps)
}

fn smart_factor_trace(log: &str) -> Vec<char> {
    let chars: Vec<char> = log.chars().collect();
    assert!(!chars.is_empty(), "smart_factor emitted no shape log");
    let chunks: Vec<&[char]> = chars.chunks(64).collect();
    let mut steps = Vec::with_capacity(chunks.len());
    for (i, chunk) in chunks.iter().enumerate() {
        let last = i + 1 == chunks.len();
        steps.push(GStep {
            repr: '⊣',
            judgment: if last { M_T } else { M_B },
            recognised: M_T,
            next: if last { M_FIX } else { '⊣' },
            applied_word: chunk.to_vec(),
        });
    }
    encode_trace(&steps)
}

#[test]
fn three_independent_producers_converge_on_the_same_factor_object() {
    let n = tape_u64(N);

    // Producer 1: the production mark router.
    let router = RouterG::from_enum(&RouterObject::initial());
    let (router_found, router_steps) = run_mark(&router, N, 8);
    let (rp_u64, rq_u64) = router_found.expect("RouterG did not produce the measured factor pair");
    let rp = tape_u64(rp_u64);
    let rq = tape_u64(rq_u64);
    assert!(witness_valid(&n, &rp, &rq));
    let router_carrier = FactorCarrier::from_trace(&n, &rp, &rq, &router_steps).unwrap();
    let router_readout = extract_word(&router_carrier.encode()).unwrap();

    // Producer 2: the arbitrary-width 31-slot resident membrane.
    let mut resident = UnboundedResident::new(n.clone());
    resident.run();
    assert!(resident.boundary_ok);
    assert!(resident.sidearm_round_trip);
    let resident_p = resident
        .factors
        .iter()
        .find(|f| cmp(f, &tape_u64(1)) == Ordering::Greater && cmp(f, &n) == Ordering::Less)
        .expect("resident produced no proper factor")
        .clone();
    let (resident_q, resident_rem) = divmod(&n, &resident_p);
    assert!(zero(&resident_rem));
    assert!(same_unordered_pair(&resident_p, &resident_q, &rp, &rq));
    let resident_carrier = FactorCarrier::new(
        n.clone(),
        resident_p,
        resident_q,
        resident_program_trace(),
    ).unwrap();
    let resident_readout = extract_word(&resident_carrier.encode()).unwrap();

    // Producer 3: the shape-routing smart factor tower.
    let (smart_parts, shape_log) = smart_factor(&n);
    assert_eq!(smart_parts.len(), 2, "measured N should be a semiprime in the smart-factor tower");
    assert!(same_unordered_pair(&smart_parts[0], &smart_parts[1], &rp, &rq));
    let smart_carrier = FactorCarrier::new(
        n.clone(),
        smart_parts[0].clone(),
        smart_parts[1].clone(),
        smart_factor_trace(&shape_log),
    ).unwrap();
    let smart_readout = extract_word(&smart_carrier.encode()).unwrap();

    // Provenance is intentionally unrelated in shape and depth.
    assert_ne!(router_carrier.trace, resident_carrier.trace);
    assert_ne!(router_carrier.trace, smart_carrier.trace);
    assert_ne!(resident_carrier.trace, smart_carrier.trace);

    // The extracted mathematical object is identical across all three producers.
    for readout in [&router_readout, &resident_readout, &smart_readout] {
        assert!(witness_valid(&n, &readout.p, &readout.q));
        assert_eq!(cmp(&mul(&readout.p, &readout.q), &n), Ordering::Equal);
        assert!(same_unordered_pair(&readout.p, &readout.q, &rp, &rq));
        assert_eq!(decode_trace(&readout.normal_form).unwrap().len(), 1);
    }

    println!(
        "cross-producer convergence: router={} records, resident={} records, smart={} records",
        decode_trace(&router_carrier.trace).unwrap().len(),
        decode_trace(&resident_carrier.trace).unwrap().len(),
        decode_trace(&smart_carrier.trace).unwrap().len(),
    );
}

fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

const REPRS: [char; 3] = ['⊢', '⊣', '⋈'];
const PAYLOAD: [char; 12] = ['⊢', '⊣', '≻', '∈', '⊤', '⋈', '≺', '⊥', '⊞', '⊙', '∋', '⊡'];

fn expanded_trace(state: &mut u64, terminal: &GStep, scaffold: usize) -> Vec<char> {
    let mut reprs = Vec::with_capacity(scaffold);
    for _ in 0..scaffold {
        reprs.push(REPRS[(xorshift64(state) as usize) % REPRS.len()]);
    }

    let mut steps = Vec::with_capacity(scaffold + 1);
    for i in 0..scaffold {
        let next = if i + 1 < scaffold { reprs[i + 1] } else { terminal.repr };
        let payload_len = 1 + (xorshift64(state) as usize % 48);
        let mut payload = Vec::with_capacity(payload_len);
        for _ in 0..payload_len {
            payload.push(PAYLOAD[(xorshift64(state) as usize) % PAYLOAD.len()]);
        }
        steps.push(GStep {
            repr: reprs[i],
            judgment: if xorshift64(state) & 1 == 0 { M_B } else { M_N },
            recognised: M_T,
            next,
            applied_word: payload,
        });
    }
    steps.push(terminal.clone());
    encode_trace(&steps)
}

#[test]
fn neutral_scaffold_expansions_converge_to_one_exact_normal_form() {
    let n = tape_u64(N);
    let router = RouterG::from_enum(&RouterObject::initial());
    let (found, trajectory) = run_mark(&router, N, 8);
    let (p_u64, q_u64) = found.expect("production route did not carry factors");
    let p = tape_u64(p_u64);
    let q = tape_u64(q_u64);

    // Establish the canonical terminal carrier from the real production route.
    let base = FactorCarrier::from_trace(&n, &p, &q, &trajectory).unwrap();
    let base_readout = extract(&base).unwrap();
    let base_steps = decode_trace(&base_readout.normal_form).unwrap();
    assert_eq!(base_steps.len(), 1);
    let terminal = base_steps[0].clone();
    let canonical = base_readout.normal_form.clone();

    let mut state = 0x434f_4e46_4c55_454eu64; // "CONFLUEN"
    let variants = 128usize;

    for case in 0..variants {
        let scaffold = xorshift64(&mut state) as usize % 64;
        let trace = expanded_trace(&mut state, &terminal, scaffold);
        assert_eq!(judge_trace(&trace), M_T, "case {case}: expanded trace did not stay closed");

        let carrier = FactorCarrier::new(n.clone(), p.clone(), q.clone(), trace).unwrap();
        let encoded = carrier.encode();
        let readout = extract_word(&encoded).unwrap();

        assert_eq!(readout.transforms, scaffold, "case {case}: wrong descent length");
        assert_eq!(readout.generations.len(), scaffold + 1, "case {case}: wrong generation count");
        assert_eq!(readout.normal_form, canonical, "case {case}: did not converge byte-for-byte");
        assert!(witness_valid(&n, &readout.p, &readout.q));
        assert_eq!(cmp(&mul(&readout.p, &readout.q), &n), Ordering::Equal);
    }

    println!("metamorphic confluence: {variants} ≡c-neutral expansions -> one exact normal form");
}
