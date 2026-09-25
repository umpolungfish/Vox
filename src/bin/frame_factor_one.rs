//! Baked support-domain unbraiding through a Gödel evaluation frame.
//!
//! The decimal input is imscribed before compilation. Its LSB-first IMASM
//! support is divided into width-sized joint states. The bit-register lift
//! performs inverse convolution and carry closure while traversing those
//! groups. Returned factor supports are then transported back through the same
//! frames, where their product closes on the baked source.

use vox::morphism_factor::{self, dec_of, mul, parse_numeral};

const BAKED_N_WORD: &str = match option_env!("FRAME_FACTOR_N_WORD") {
    Some(word) => word,
    None => "⊢⊙⊡⊣",
};
const BAKED_WIDTH: usize = match option_env!("FRAME_FACTOR_WIDTH") {
    Some(width) => match parse_width(width) {
        Some(value) => value,
        None => 2,
    },
    None => 2,
};

const fn parse_width(raw: &str) -> Option<usize> {
    let bytes = raw.as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let mut index = 0;
    let mut value = 0usize;
    while index < bytes.len() {
        let digit = bytes[index].wrapping_sub(b'0');
        if digit > 9 {
            return None;
        }
        let Some(next) = value.checked_mul(10) else {
            return None;
        };
        let Some(next) = next.checked_add(digit as usize) else {
            return None;
        };
        value = next;
        index += 1;
    }
    if value >= 2 {
        Some(value)
    } else {
        None
    }
}

type Tape = Vec<char>;

struct EvaluationFrame {
    width: usize,
    groups: Vec<Tape>,
    bit_count: usize,
}

fn shift_evaluation_frame(source: &[char], width: usize) -> EvaluationFrame {
    EvaluationFrame {
        width,
        groups: source.chunks(width).map(<[char]>::to_vec).collect(),
        bit_count: source.len(),
    }
}

fn shifted_support(frame: &EvaluationFrame) -> Result<Tape, String> {
    let support = frame.groups.iter().flatten().copied().collect::<Tape>();
    if support.len() != frame.bit_count {
        return Err("evaluation frame changed its encoded support length".into());
    }
    Ok(support)
}

fn frame_support(frame: &EvaluationFrame) -> Result<Vec<Tape>, String> {
    shifted_support(frame)?;
    Ok(frame.groups.clone())
}

fn return_from_frame(frame: &EvaluationFrame) -> Result<Tape, String> {
    frame_support(frame).map(|groups| groups.into_iter().flatten().collect())
}

fn factor_in_frame(
    frame: &EvaluationFrame,
) -> Result<(Vec<EvaluationFrame>, Option<Tape>), String> {
    let support_frames = frame_support(frame)?;
    let phase_pair = vox::factor_2adic::factor_2adic_phase_support_frames(&support_frames);
    let (mut left, mut right, phase_index) = if let Some((left, right, index)) = phase_pair {
        (left, right, Some(index))
    } else {
        let (left, right) =
            vox::factor_2adic::factor_2adic_semiprime_frames(&support_frames, Some(1))
                .into_iter()
                .next()
                .ok_or_else(|| {
                    "support/frame phase closure and inverse convolution did not close".to_string()
                })?;
        (left, right, None)
    };
    if morphism_factor::cmp(&left, &right) == core::cmp::Ordering::Greater {
        core::mem::swap(&mut left, &mut right);
    }
    if !vox::factor_2adic::factor_pair_closes_in_frames(&support_frames, &left, &right) {
        return Err(format!(
            "candidate pair failed the inverse-convolution frame closure (phase {phase_index:?})"
        ));
    }
    let shifted_source = return_from_frame(frame)?;
    let left_frame = shift_evaluation_frame(&left, frame.width);
    let right_frame = shift_evaluation_frame(&right, frame.width);
    let shifted_product = vox::factor_2adic::multiply_shifted_frames(
        &frame_support(&left_frame)?,
        left_frame.width,
        &frame_support(&right_frame)?,
        right_frame.width,
    )
    .ok_or_else(|| "factor frame product could not be transported".to_string())?;
    if shifted_product != shifted_source {
        return Err("inverse-convolution registers do not close on the source frame".into());
    }
    Ok((vec![left_frame, right_frame], phase_index))
}

fn factor_baked_value() -> Result<String, String> {
    if BAKED_WIDTH < 2 {
        return Err("baked frame width must be at least 2".into());
    }
    let source = parse_numeral(BAKED_N_WORD)?;
    let source_frame = shift_evaluation_frame(&source, BAKED_WIDTH);
    let framed_source = return_from_frame(&source_frame)?;
    if framed_source != source {
        return Err("source frame failed its exact return check".into());
    }

    let (factor_frames, phase_index) = factor_in_frame(&source_frame)?;
    let returned_factors = factor_frames
        .iter()
        .map(|frame| return_from_frame(frame))
        .collect::<Result<Vec<_>, _>>()?;
    let product = returned_factors
        .iter()
        .fold(vec![vox::vox::EVALF], |acc, factor| mul(&acc, factor));
    if product != source {
        return Err("returned factors do not close on the baked source numeral".into());
    }
    for width in 2..=8 {
        let left = shift_evaluation_frame(&returned_factors[0], width);
        let right = shift_evaluation_frame(&returned_factors[1], width);
        let transported = vox::factor_2adic::multiply_shifted_frames(
            &frame_support(&left)?,
            left.width,
            &frame_support(&right)?,
            right.width,
        )
        .ok_or_else(|| format!("factor product did not transport through width-{width}"))?;
        let source_frame = shift_evaluation_frame(&source, width);
        if transported != return_from_frame(&source_frame)? {
            return Err(format!(
                "factor product did not close in width-{width} frame"
            ));
        }
    }

    let factor_words = returned_factors
        .iter()
        .map(|factor| morphism_factor::emit_numeral(factor))
        .collect::<Vec<_>>();
    let factor_values = returned_factors
        .iter()
        .map(|factor| dec_of(factor))
        .collect::<Vec<_>>();
    let route = if phase_index.is_some() {
        "support-frame-phase"
    } else {
        "prefix-fallback"
    };
    Ok(format!(
        "frame-width  {}\nsource-word  {}\nsource       {}\nfactor-route {}\nphase-index  {}\nfactor-words {}\nfactors      {}\nproduct      {}\nclosure      closed\n",
        BAKED_WIDTH,
        BAKED_N_WORD,
        dec_of(&source),
        route,
        phase_index
            .as_ref()
            .map_or_else(|| "none".to_string(), |index| dec_of(index)),
        factor_words.join(" | "),
        factor_values.join(" x "),
        dec_of(&product),
    ))
}

fn main() {
    match factor_baked_value() {
        Ok(report) => print!("{report}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn numeral(value: &str) -> Tape {
        morphism_factor::decimal_to_tape(value).expect("decimal test numeral")
    }

    #[test]
    fn frame_shift_returns_whole_value_at_every_resolution() {
        let source = numeral("15241578750190521");
        for width in (2..=8).chain([17, 65, 257]) {
            assert_eq!(
                return_from_frame(&shift_evaluation_frame(&source, width)).unwrap(),
                source
            );
        }
    }

    #[test]
    fn returned_factors_transport_back_and_close_in_source_frame() {
        let source = numeral("100160063");
        let source_frame = shift_evaluation_frame(&source, 8);
        let (factor_frames, _) = factor_in_frame(&source_frame).unwrap();
        let returned = factor_frames
            .iter()
            .map(|frame| return_from_frame(frame).unwrap())
            .collect::<Vec<_>>();
        let product = returned
            .iter()
            .fold(vec![vox::vox::EVALF], |acc, factor| mul(&acc, factor));
        assert_eq!(product, source);
        assert_eq!(
            returned
                .iter()
                .map(|factor| dec_of(factor))
                .collect::<Vec<_>>(),
            ["10007", "10009"]
        );
    }

    #[test]
    fn frame_solver_consumes_support_groups_and_closes_the_encoded_product() {
        let source = numeral("100160063");
        let frame = shift_evaluation_frame(&source, 8);
        let support_frames = frame_support(&frame).unwrap();
        assert_eq!(
            vox::factor_2adic::factor_2adic_frames(&support_frames, Some(1)),
            vec![(numeral("10007"), numeral("10009"))]
        );
        assert!(vox::factor_2adic::factor_pair_closes_in_frames(
            &support_frames,
            &numeral("10007"),
            &numeral("10009"),
        ));
        assert!(!vox::factor_2adic::factor_pair_closes_in_frames(
            &support_frames,
            &numeral("10008"),
            &numeral("10009"),
        ));
    }

    #[test]
    fn semiprime_sieve_preserves_factors_equal_to_its_prime_registers() {
        for (n, p, q) in [("15", "3", "5"), ("49", "7", "7")] {
            let source = numeral(n);
            for width in 2..=8 {
                let frames = frame_support(&shift_evaluation_frame(&source, width)).unwrap();
                assert_eq!(
                    vox::factor_2adic::factor_2adic_semiprime_frames(&frames, Some(1)),
                    vec![(numeral(p), numeral(q))],
                    "N={n}, width={width}"
                );
            }
        }
    }
}
