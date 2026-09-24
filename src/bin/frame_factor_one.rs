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
    word: String,
    bit_count: usize,
}

fn shift_evaluation_frame(source: &[char], width: usize) -> EvaluationFrame {
    EvaluationFrame {
        width,
        word: morphism_factor::emit_numeral(source),
        bit_count: source.len(),
    }
}

fn shifted_support(frame: &EvaluationFrame) -> Result<Tape, String> {
    let support = parse_numeral(&frame.word)?;
    if support.len() != frame.bit_count {
        return Err("evaluation frame changed its encoded support length".into());
    }
    Ok(support)
}

fn frame_support(frame: &EvaluationFrame) -> Result<Vec<Tape>, String> {
    let support = shifted_support(frame)?;
    Ok(support.chunks(frame.width).map(<[char]>::to_vec).collect())
}

fn return_from_frame(frame: &EvaluationFrame) -> Result<Tape, String> {
    frame_support(frame).map(|groups| groups.into_iter().flatten().collect())
}

fn factor_in_frame(frame: &EvaluationFrame) -> Result<Vec<EvaluationFrame>, String> {
    let support_frames = frame_support(frame)?;
    let (left, right) = vox::factor_2adic::factor_2adic_semiprime_frames(&support_frames, Some(1))
        .into_iter()
        .next()
        .ok_or_else(|| "support-frame inverse convolution did not close".to_string())?;
    let shifted_source = return_from_frame(frame)?;
    if mul(&left, &right) != shifted_source {
        return Err("inverse-convolution registers do not close on the source frame".into());
    }
    Ok([left, right]
        .iter()
        .map(|factor| shift_evaluation_frame(factor, frame.width))
        .collect())
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

    let factor_frames = factor_in_frame(&source_frame)?;
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

    let factor_words = returned_factors
        .iter()
        .map(|factor| morphism_factor::emit_numeral(factor))
        .collect::<Vec<_>>();
    let factor_values = returned_factors
        .iter()
        .map(|factor| dec_of(factor))
        .collect::<Vec<_>>();
    Ok(format!(
        "frame-width  {}\nsource-word  {}\nsource       {}\nfactor-words {}\nfactors      {}\nproduct      {}\nclosure      closed\n",
        BAKED_WIDTH,
        BAKED_N_WORD,
        dec_of(&source),
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
        for width in (2..=8).chain([65, 257]) {
            let source_frame = shift_evaluation_frame(&source, width);
            let factor_frames = factor_in_frame(&source_frame).unwrap();
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
