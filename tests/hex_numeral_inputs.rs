use vox::{combined_factor as codec, fixed_point_imasm::ImasmNumeralWord,
    godel_calculus, membrane_factor, morphism_factor as tape};
use num_bigint::BigUint;

#[test]
fn every_numeral_reader_and_word_operation_accepts_hex() {
    let wide: BigUint = "1218745108538121657062000951884568623049596243192565916939841".parse().unwrap();
    for n in (0u32..16).map(BigUint::from).chain([wide.clone(), (&wide << 700usize)+BigUint::from(1u8)]) {
        let native = codec::native_word(&n);
        let hex = codec::hex_word(&n);
        assert_eq!(codec::decode_numeral_word(&native).unwrap(), n);
        assert_eq!(codec::decode_numeral_word(&hex).unwrap(), n);
        assert_eq!(tape::parse_numeral(&native).unwrap(), tape::parse_numeral(&hex).unwrap());
        assert_eq!(ImasmNumeralWord::checked(&native).unwrap(), ImasmNumeralWord::checked(&hex).unwrap());
        assert_eq!(codec::rotat_word(&native, 3).unwrap(), codec::rotat_word(&hex, 3).unwrap());
        assert_eq!(codec::interlace_words(&native, &native).unwrap(), codec::interlace_words(&hex, &hex).unwrap());
        assert_eq!(codec::deinterlace_word(&native).unwrap(), codec::deinterlace_word(&hex).unwrap());
        let normalized = godel_calculus::normalize_numeral_word(&hex).unwrap();
        assert_eq!(godel_calculus::parse_natural_word(&normalized).unwrap().to_string(), n.to_string());
        assert!(codec::decode_hex_bits(&(codec::HEX_WORDS[0].to_owned()+&hex)).is_err());
    }
    let native = codec::native_word(&wide);
    let hex = codec::hex_word(&wide);
    for args in [vec![native], vec![hex.clone()], vec!["--native-word".into(), hex.clone()], vec!["--hex-word".into(), hex]] {
        assert_eq!(membrane_factor::source_for_gpu_producer(&args).unwrap(), Some(wide.to_string()));
    }
}
