use vox::dialectic_reentry::{Descent, DialecticObject};
use vox::morphism_factor::{mul, tape_u64};

#[test]
fn neutral_identity_survives_repeated_restart_at_every_cut() {
    let mut current = DialecticObject::new(mul(&tape_u64(1_000_003), &tape_u64(10_000_019))).unwrap();
    let mut cuts = 0;
    loop {
        let neutral = current.clone().route_around().unwrap();
        let wire = neutral.encode();
        let mut restarted = neutral;
        for _ in 0..3 {
            restarted = DialecticObject::decode(&restarted.encode()).unwrap();
            restarted = match restarted.descend() {
                Descent::N(identity) => identity,
                other => panic!("neutral descent changed FOUR: {other:?}"),
            };
            assert_eq!(restarted.encode(), wire);
        }
        cuts += 1;
        match current.descend() {
            Descent::B(next) => current = next,
            Descent::T(_) => break,
            other => panic!("productive control failed: {other:?}"),
        }
    }
    assert_eq!(cuts, 12);
}
