//! Prepared Gödel frame separation over resident IMASM numeral tapes.
#![deny(warnings)]

use std::time::Instant;
use vox::morphism_factor;
use vox::vox::{EVALF, EVALT};

type Tape = Vec<char>;

const SOURCE_WORD: &str = match option_env!("GODEL_SOURCE_WORD") {
    Some(word) => word,
    None => "⊢⊙⊡⊣",
};

fn rotat(bits: &[char], cut: usize) -> Tape {
    if bits.is_empty() {
        return Vec::new();
    }
    let cut = cut % bits.len();
    bits[cut..].iter().chain(&bits[..cut]).copied().collect()
}

fn separate(bits: &[char]) -> (Tape, Tape) {
    let mut p = Vec::with_capacity((bits.len() + 1) / 2);
    let mut q = Vec::with_capacity(bits.len() / 2);
    for (index, &bit) in bits.iter().enumerate() {
        if index % 2 == 0 {
            p.push(bit);
        } else {
            q.push(bit);
        }
    }
    (p, q)
}

fn interlace(lanes: &[Tape], width: usize, depth: usize) -> Tape {
    let mut out = Vec::with_capacity(width);
    for position in 0..width {
        let mut lane = position % lanes.len();
        let mut reversed = 0usize;
        for _ in 0..depth {
            reversed = (reversed << 1) | (lane & 1);
            lane >>= 1;
        }
        let lane = reversed;
        let offset = position / lanes.len();
        out.push(lanes[lane].get(offset).copied().unwrap_or(EVALT));
    }
    out
}

fn product(lanes: &[Tape]) -> Tape {
    let mut out = vec![EVALF];
    for lane in lanes {
        out = morphism_factor::mul(&out, lane);
        if out.len() == 1 && out[0] == EVALT {
            break;
        }
    }
    out
}

fn value(bits: &[char]) -> String {
    morphism_factor::dec_of(bits)
}

fn main() {
    let started = Instant::now();
    let result = (|| -> Result<String, String> {
        let source = morphism_factor::parse_numeral(SOURCE_WORD)?;
        let source_value = value(&source);
        let source_bits = source.len();
        let mut level_stats: Vec<(usize, usize, usize, usize)> = Vec::new();
        let mut closures = Vec::new();
        let mut first = None;

        for cut in 0..source_bits {
            let shifted = rotat(&source, cut);
            let mut lanes = vec![shifted.clone()];
            let mut depth = 0usize;
            while lanes.iter().any(|lane| lane.len() > 1) {
                depth += 1;
                lanes = lanes
                    .iter()
                    .flat_map(|lane| {
                        let (p, q) = separate(lane);
                        [p, q]
                    })
                    .collect();
                if interlace(&lanes, source_bits, depth) != shifted {
                    return Err(format!("depth-{depth} reassembly failed at cut {cut}"));
                }
                let direct = product(&lanes);
                let inverse: Vec<Tape> = lanes
                    .iter()
                    .map(|lane| rotat(lane, lane.len().wrapping_sub(cut % lane.len().max(1))))
                    .collect();
                let inverse_product = product(&inverse);
                if cut == 0 {
                    let (p, q) = separate(&source);
                    first = Some((value(&p), value(&q)));
                }
                if level_stats.len() < depth {
                    level_stats.push((depth, 0, 0, lanes.len()));
                }
                let stats = &mut level_stats[depth - 1];
                stats.1 += usize::from(value(&direct) == source_value);
                stats.2 += usize::from(value(&inverse_product) == source_value);
                if value(&direct) == source_value {
                    closures.push(format!("cut={cut} depth={depth} direct {} lanes", lanes.len()));
                }
                if value(&inverse_product) == source_value {
                    closures.push(format!("cut={cut} depth={depth} inverse-ROTAT {} lanes", lanes.len()));
                }
            }
        }

        let (p, q) = first.unwrap_or_else(|| ("0".to_string(), "0".to_string()));
        Ok(format!(
            "source {source_value}\nsource-bits {source_bits}\nframeshift-cuts {source_bits}\nfirst-p {p}\nfirst-q {q}\nproduct-closure-count {}\nproduct-closures {}\nseparation-levels\n{}\n",
            closures.len(),
            if closures.is_empty() { "none".to_string() } else { closures.join("\n") },
            level_stats.iter().map(|(depth, direct, inverse, leaves)|
                format!("depth={depth} leaves={leaves} direct={direct} inverse-ROTAT={inverse}"))
                .collect::<Vec<_>>().join("\n"),
        ))
    })();

    match result {
        Ok(report) => print!("{report}elapsed-seconds {:.6}\n", started.elapsed().as_secs_f64()),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
