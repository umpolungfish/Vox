#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Winding {
    num: u64,
    den: u64,
}

impl Winding {
    fn new(num: u64, den: u64) -> Self {
        assert!(den != 0);
        let num = num % den;
        if num == 0 {
            return Self { num: 0, den: 1 };
        }
        let g = gcd_u64(num, den);
        Self {
            num: num / g,
            den: den / g,
        }
    }

    fn is_self_inverse(self) -> bool {
        self.num == 0 || (self.den == 2 && self.num == 1)
    }
}

fn gcd_u64(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }
    a
}

fn is_quadratic_residue_mod_11(r: u64) -> bool {
    (1..11).any(|x| (x * x) % 11 == r % 11)
}

fn discrete_log_base_2_mod_11(r: u64) -> Option<u64> {
    let mut value = 1u64;
    for exponent in 0..10 {
        if value == r {
            return Some(exponent);
        }
        value = (value * 2) % 11;
    }
    None
}

fn quadratic_character_mod_11(x: i64) -> i8 {
    let r = x.rem_euclid(11) as u64;
    if r == 0 {
        0
    } else if is_quadratic_residue_mod_11(r) {
        1
    } else {
        -1
    }
}

fn paley_12() -> [[i8; 12]; 12] {
    let mut h = [[0i8; 12]; 12];
    for entry in &mut h[0] {
        *entry = 1;
    }
    for i in 0..11 {
        h[i + 1][0] = 1;
        for j in 0..11 {
            h[i + 1][j + 1] = if i == j {
                -1
            } else {
                quadratic_character_mod_11(j as i64 - i as i64)
            };
        }
    }
    h
}

fn sylvester_16() -> [[i8; 16]; 16] {
    let mut h = [[0i8; 16]; 16];
    for i in 0u32..16 {
        for j in 0u32..16 {
            h[i as usize][j as usize] = if (i & j).count_ones() & 1 == 0 {
                1
            } else {
                -1
            };
        }
    }
    h
}

fn assert_hadamard<const N: usize>(h: &[[i8; N]; N]) {
    for row in h {
        for &entry in row {
            assert!(entry == 1 || entry == -1);
        }
    }
    for i in 0..N {
        for j in 0..N {
            let dot: i64 = (0..N)
                .map(|k| i64::from(h[i][k]) * i64::from(h[j][k]))
                .sum();
            assert_eq!(dot, if i == j { N as i64 } else { 0 }, "rows {i},{j}");
        }
    }
}

#[test]
fn generic_cyclic_winding_is_real_only_at_orders_one_and_two() {
    for d in 1..=64 {
        let witness = Winding::new(1, d);
        assert_eq!(
            witness.is_self_inverse(),
            d <= 2,
            "W(1,1)=1/{d} gave an unexpected self-inverse reading",
        );
    }

    for d in [1u64, 2] {
        for j in 0..d {
            for k in 0..d {
                assert!(
                    Winding::new(j * k, d).is_self_inverse(),
                    "order {d} entry ({j},{k}) was not real",
                );
            }
        }
    }
}

#[test]
fn paley_12_quadratic_character_is_the_order_two_reading() {
    for r in 1..11 {
        let qr = is_quadratic_residue_mod_11(r);
        let log = discrete_log_base_2_mod_11(r).expect("2 should generate F_11^*");
        assert_eq!(qr, log % 2 == 0, "residue {r}");

        let winding = if qr {
            Winding::new(0, 1)
        } else {
            Winding::new(1, 2)
        };
        assert!(winding.is_self_inverse(), "residue {r}");
    }

    let h = paley_12();
    assert_hadamard(&h);
}

#[test]
fn sylvester_16_carrier_and_its_negation_are_hadamard() {
    let h = sylvester_16();
    assert_hadamard(&h);

    let mut negated = h;
    for row in &mut negated {
        for entry in row {
            *entry = -*entry;
        }
    }
    assert_hadamard(&negated);
}
