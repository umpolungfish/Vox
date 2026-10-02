//! Exact structural H/T/S/X preparation, with no amplitude expansion.
use crate::vox::{EVALF, EVALT};
use alloc::vec::Vec;

/// T has order eight. These three numeral cells describe that physical phase
/// ring; they do not bound circuit length, register width or nesting depth.
pub type Phase = [char; 3];
const ZERO: Phase = [EVALT; 3];
fn add(a: Phase, b: Phase) -> Phase {
    let mut out = ZERO;
    let mut carry = false;
    for i in 0..3 {
        let x = a[i] == EVALF;
        let y = b[i] == EVALF;
        out[i] = if x ^ y ^ carry { EVALF } else { EVALT };
        carry = (x && y) || (x && carry) || (y && carry);
    }
    out
}
fn neg(a: Phase) -> Phase {
    add(
        a.map(|bit| if bit == EVALF { EVALT } else { EVALF }),
        [EVALF, EVALT, EVALT],
    )
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gate {
    Hadamard,
    BitFlip,
    Phase(Phase),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Circuit {
    pub gates: Vec<Gate>,
    pub global_phase: Phase,
}
impl Circuit {
    pub fn prepare(spec: &str) -> Result<Self, &'static str> {
        let mut out = Self {
            gates: Vec::new(),
            global_phase: ZERO,
        };
        for mark in spec.chars() {
            if mark.is_whitespace() {
                continue;
            }
            out.gates.push(match mark.to_ascii_uppercase() {
                'H' => Gate::Hadamard,
                'X' => Gate::BitFlip,
                'T' => Gate::Phase([EVALF, EVALT, EVALT]),
                'S' => Gate::Phase([EVALT, EVALF, EVALT]),
                _ => return Err("unknown structural quantum gate"),
            });
            loop {
                let n = out.gates.len();
                if matches!(out.gates.last(),Some(Gate::Phase(p)) if *p==ZERO) {
                    out.gates.pop();
                    continue;
                }
                if n >= 2 {
                    match (out.gates[n - 2], out.gates[n - 1]) {
                        (Gate::Phase(a), Gate::Phase(b)) => {
                            out.gates.truncate(n - 2);
                            out.gates.push(Gate::Phase(add(a, b)));
                            continue;
                        }
                        (Gate::Hadamard, Gate::Hadamard) | (Gate::BitFlip, Gate::BitFlip) => {
                            out.gates.truncate(n - 2);
                            continue;
                        }
                        _ => {}
                    }
                }
                if n >= 3 {
                    match (out.gates[n - 3], out.gates[n - 2], out.gates[n - 1]) {
                        (Gate::Hadamard, Gate::Phase([EVALT, EVALT, EVALF]), Gate::Hadamard) => {
                            out.gates.truncate(n - 3);
                            out.gates.push(Gate::BitFlip);
                            continue;
                        }
                        (Gate::BitFlip, Gate::Phase(p), Gate::BitFlip) => {
                            out.global_phase = add(out.global_phase, p);
                            out.gates.truncate(n - 3);
                            out.gates.push(Gate::Phase(neg(p)));
                            continue;
                        }
                        _ => {}
                    }
                }
                break;
            }
        }
        Ok(out)
    }
}
#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    type Complex = (f64, f64);
    type Matrix = [[Complex; 2]; 2];
    fn mul(a: Matrix, b: Matrix) -> Matrix {
        let mut out = [[(0., 0.); 2]; 2];
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    out[i][j].0 += a[i][k].0 * b[k][j].0 - a[i][k].1 * b[k][j].1;
                    out[i][j].1 += a[i][k].0 * b[k][j].1 + a[i][k].1 * b[k][j].0;
                }
            }
        }
        out
    }
    fn identity() -> Matrix {
        [[(1., 0.), (0., 0.)], [(0., 0.), (1., 0.)]]
    }
    fn phase(p: Phase) -> Complex {
        let tick = p
            .iter()
            .enumerate()
            .map(|(i, &v)| usize::from(v == EVALF) << i)
            .sum::<usize>();
        let angle = tick as f64 * std::f64::consts::FRAC_PI_4;
        (angle.cos(), angle.sin())
    }
    fn matrix(g: Gate) -> Matrix {
        match g {
            Gate::Hadamard => {
                let h = std::f64::consts::FRAC_1_SQRT_2;
                [[(h, 0.), (h, 0.)], [(h, 0.), (-h, 0.)]]
            }
            Gate::BitFlip => [[(0., 0.), (1., 0.)], [(1., 0.), (0., 0.)]],
            Gate::Phase(p) => [[(1., 0.), (0., 0.)], [(0., 0.), phase(p)]],
        }
    }
    #[test]
    fn qc_example_folds_to_one_octant_between_hadamards() {
        let c = Circuit::prepare("hssstttssh").unwrap();
        assert_eq!(
            c.gates,
            vec![
                Gate::Hadamard,
                Gate::Phase([EVALF, EVALT, EVALF]),
                Gate::Hadamard
            ]
        );
        assert_eq!(c.global_phase, ZERO);
    }
    #[test]
    fn all_short_circuits_preserve_the_complete_unitary() {
        let alphabet = ['H', 'S', 'T', 'X'];
        for length in 0..=6 {
            for pattern in 0..4usize.pow(length) {
                let mut code = pattern;
                let mut spec = alloc::string::String::new();
                let mut expected = identity();
                for _ in 0..length {
                    let mark = alphabet[code % 4];
                    code /= 4;
                    spec.push(mark);
                    let gate = match mark {
                        'H' => Gate::Hadamard,
                        'X' => Gate::BitFlip,
                        'S' => Gate::Phase([EVALT, EVALF, EVALT]),
                        _ => Gate::Phase([EVALF, EVALT, EVALT]),
                    };
                    expected = mul(matrix(gate), expected);
                }
                let folded = Circuit::prepare(&spec).unwrap();
                let mut actual = identity();
                for gate in folded.gates {
                    actual = mul(matrix(gate), actual);
                }
                let scalar = phase(folded.global_phase);
                actual = mul(
                    [[(scalar.0, scalar.1), (0., 0.)], [(0., 0.), scalar]],
                    actual,
                );
                for i in 0..2 {
                    for j in 0..2 {
                        assert!((actual[i][j].0 - expected[i][j].0).abs() < 1e-12, "{spec}");
                        assert!((actual[i][j].1 - expected[i][j].1).abs() < 1e-12, "{spec}");
                    }
                }
            }
        }
    }
}
