//! Baked full-value factorization through a Gödel evaluation frame.
//!
//! The decimal input is imscribed before compilation. Its LSB-first IMASM
//! numeral is divided into width-sized joint states, then the whole numeral is
//! reconstructed in that frame before the Vox factor membrane runs. Each
//! returned factor is framed in the same way and transported back to the
//! source numeral. The product check closes in the source frame.

use vox::morphism_factor::{self, dec_of, divmod, factor, mul, parse_numeral};

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
    symbols: Vec<(String, usize)>,
}

fn shift_to_frame(source: &[char], width: usize) -> EvaluationFrame {
    EvaluationFrame {
        width,
        symbols: source
            .chunks(width)
            .map(|group| (morphism_factor::emit_numeral(group), group.len()))
            .collect(),
    }
}

fn return_from_frame(frame: &EvaluationFrame) -> Result<Tape, String> {
    frame
        .symbols
        .iter()
        .try_fold(Vec::new(), |mut returned, (symbol, width)| {
            let mut group = parse_numeral(symbol)?;
            group.resize(*width, vox::vox::EVALT);
            returned.extend(group);
            Ok(returned)
        })
}

fn factor_in_frame(frame: &EvaluationFrame) -> Result<Vec<EvaluationFrame>, String> {
    let shifted_source = return_from_frame(frame)?;
    let source_word = morphism_factor::emit_numeral(&shifted_source);
    let divisor_word = factor(&source_word)?;
    let divisor = parse_numeral(&divisor_word)?;
    let (quotient, remainder) = divmod(&shifted_source, &divisor);
    if remainder.iter().any(|bit| *bit == vox::vox::EVALF)
        || divisor == shifted_source
        || quotient == vec![vox::vox::EVALT]
    {
        return Err("the factor membrane did not return a nontrivial exact split".into());
    }
    Ok([divisor, quotient]
        .iter()
        .map(|factor| shift_to_frame(factor, frame.width))
        .collect())
}

fn factor_baked_value() -> Result<String, String> {
    if BAKED_WIDTH < 2 {
        return Err("baked frame width must be at least 2".into());
    }
    let source = parse_numeral(BAKED_N_WORD)?;
    let source_frame = shift_to_frame(&source, BAKED_WIDTH);
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

    fn numeral(value: u64) -> Tape {
        let mut bits = Vec::new();
        let mut remaining = value;
        while remaining != 0 {
            bits.push(if remaining & 1 == 1 {
                vox::vox::EVALF
            } else {
                vox::vox::EVALT
            });
            remaining >>= 1;
        }
        if bits.is_empty() {
            bits.push(vox::vox::EVALT);
        }
        bits
    }

    #[test]
    fn frame_shift_returns_whole_value_at_every_resolution() {
        let source = numeral(152_415_787_501_905_21);
        for width in (2..=8).chain([17, 65, 257]) {
            assert_eq!(
                return_from_frame(&shift_to_frame(&source, width)).unwrap(),
                source
            );
        }
    }

    #[test]
    fn returned_factors_transport_back_and_close_in_source_frame() {
        let source = numeral(100_160_063);
        for width in (2..=8).chain([65, 257]) {
            let source_frame = shift_to_frame(&source, width);
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
}
