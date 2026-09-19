use vox::dialectic_certificate::{
    certify_dialectic, decode_dialectic_certificate, encode_dialectic_certificate,
    verify_dialectic_certificate,
};
use vox::dialectic_reentry::DialecticObject;
use vox::morphism_factor::{mul, tape_u64};

#[test]
fn imscription_relation_is_the_single_owner_of_boundary_span_and_word() {
    let runtime_source = include_str!("../src/dialectic_reentry.rs");
    assert!(!runtime_source.contains("pub boundary: Tape"));
    assert!(!runtime_source.contains("pub span: Tape"));
    assert!(!runtime_source.contains("pub word: Vec<Mark>"));
    assert!(runtime_source.contains("pub rwx: ImscriptionRwx"));

    let certificate_source = include_str!("../src/dialectic_certificate.rs");
    assert!(!certificate_source.contains("pub terminal_word:"));
    assert!(!certificate_source.contains("pub terminal_boundary:"));
    assert!(!certificate_source.contains("pub terminal_span:"));
    assert!(!certificate_source.contains("pub terminal_rwx:"));
    assert!(certificate_source.contains("pub terminal_imscription: Imscription"));

    let n = mul(&tape_u64(1_000_003), &tape_u64(1_032_007));
    let object = DialecticObject::new(n.clone()).unwrap();
    let wire = object.encode();
    let restored = DialecticObject::decode(&wire).unwrap();
    assert_eq!(restored.boundary(), &restored.imscription.rwx.write_boundary);
    assert_eq!(restored.span(), &restored.imscription.rwx.execute_span);
    assert_eq!(restored.word(), restored.imscription.rwx.execute_word.as_slice());

    // A second raw copy of the current word is not accepted as hidden state.
    let mut duplicated_object = wire.clone();
    assert_eq!(duplicated_object.pop(), Some('⊣'));
    duplicated_object.extend_from_slice(object.word());
    duplicated_object.push('⊣');
    assert!(DialecticObject::decode(&duplicated_object).is_err());

    let certificate = certify_dialectic(&object).unwrap();
    let certificate_wire = encode_dialectic_certificate(&certificate);
    let restored_certificate = decode_dialectic_certificate(&certificate_wire).unwrap();
    assert_eq!(restored_certificate, certificate);
    verify_dialectic_certificate(&restored_certificate).unwrap();

    // The terminal closed word likewise occurs only inside terminal_imscription.
    let mut duplicated_certificate = certificate_wire;
    assert_eq!(duplicated_certificate.pop(), Some('⊣'));
    duplicated_certificate
        .extend_from_slice(&certificate.terminal_imscription.rwx.execute_word);
    duplicated_certificate.push('⊣');
    assert!(decode_dialectic_certificate(&duplicated_certificate).is_err());

    println!(
        "imscription ownership: runtime + certificate store boundary/span/word only in r/w/x; duplicate word payloads rejected"
    );
}
