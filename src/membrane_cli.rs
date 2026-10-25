use alloc::{format, string::String, vec::Vec};

use crate::membrane_interface::{Membrane, MembraneClosure, MembraneProgress};

pub struct MembraneGenerator {
    pub name: &'static str,
    pub description: &'static str,
    pub runner: MembraneGeneratorRunner,
}

pub type MembraneGeneratorRunner = fn(Vec<char>, usize) -> Result<String, String>;

pub const GENERATORS: &[MembraneGenerator] = &[
    MembraneGenerator {
        name: "factor-synthesis",
        description: "source-bound dialectic and resident GLUT factor synthesis",
        runner: run_factor_synthesis,
    },
    MembraneGenerator {
        name: "prepared-glut",
        description: "prepared resident GLUT carrier and banked closure",
        runner: run_prepared_glut,
    },
];

pub fn execute(args: &[&str]) -> Result<String, String> {
    execute_with(args, &[])
}

pub fn execute_with(
    args: &[&str],
    extensions: &[MembraneGenerator],
) -> Result<String, String> {
    match args.first().copied() {
        Some("list" | "generators") if args.len() == 1 => {
            let mut output = String::from("membrane generators\n");
            for generator in GENERATORS.iter().chain(extensions.iter()) {
                output.push_str(&format!("  {:<20} {}\n", generator.name, generator.description));
            }
            Ok(output)
        }
        Some("run") => run_generator(&args[1..], extensions),
        _ => Err(String::from(
            "usage: vox membrane list | vox membrane run <generator> <N> [--steps COUNT]",
        )),
    }
}

fn run_generator(args: &[&str], extensions: &[MembraneGenerator]) -> Result<String, String> {
    if args.len() < 2 {
        return Err(String::from(
            "usage: vox membrane run <factor-synthesis|prepared-glut> <N> [--steps COUNT]",
        ));
    }
    let steps = parse_steps(&args[2..])?;
    let source = crate::morphism_factor::decimal_to_tape(args[1])
        .ok_or_else(|| String::from("N must be a positive decimal integer"))?;
    let generator = GENERATORS
        .iter()
        .chain(extensions.iter())
        .find(|generator| generator.name == args[0])
        .ok_or_else(|| format!("unknown membrane generator `{}`; use `membrane list`", args[0]))?;
    (generator.runner)(source, steps)
}

fn parse_steps(args: &[&str]) -> Result<usize, String> {
    if args.is_empty() {
        return Ok(1);
    }
    if args.len() != 2 || args[0] != "--steps" {
        return Err(String::from("expected optional `--steps COUNT`"));
    }
    let steps = args[1]
        .parse::<usize>()
        .map_err(|_| String::from("step count must be a positive integer"))?;
    if steps == 0 || steps > 1_000_000 {
        return Err(String::from("step count must be between 1 and 1000000"));
    }
    Ok(steps)
}

fn run_factor_synthesis(source: Vec<char>, steps: usize) -> Result<String, String> {
    let source_decimal = decimal(&source);
    let mut membrane = <crate::factor_synthesis::FactorSynthesis as Membrane>::prepare(source)?;
    let mut output = format!("prepared generator=factor-synthesis source={source_decimal}\n");
    let mut ready = false;
    let mut completed = 0;
    for step in 1..=steps {
        completed = step;
        match <crate::factor_synthesis::FactorSynthesis as Membrane>::advance(&mut membrane)? {
            MembraneProgress::Open => output.push_str(&format!("advance {step}: open\n")),
            MembraneProgress::ReadyToClose => {
                output.push_str(&format!("advance {step}: ready-to-close\n"));
                ready = true;
                break;
            }
        }
    }
    match <crate::factor_synthesis::FactorSynthesis as Membrane>::close(membrane)? {
        MembraneClosure::Closed { readout, .. } => {
            output.push_str(&format!(
                "closed producer={} turns={} p={} q={} product_verified=true\n",
                readout.producer,
                readout.turns,
                decimal(&readout.p),
                decimal(&readout.q),
            ));
        }
        MembraneClosure::Open(_continuation) => {
            output.push_str(&format!(
                "closure=open continuation=returned-to-caller steps={} ready={ready}; not persisted after this process exits\n",
                completed,
            ));
        }
    }
    Ok(output)
}

fn run_prepared_glut(source: Vec<char>, steps: usize) -> Result<String, String> {
    let source_decimal = decimal(&source);
    let mut membrane = <crate::glut_system::PreparedGlutMembrane as Membrane>::prepare(source)?;
    let mut output = format!("prepared generator=prepared-glut source={source_decimal}\n");
    let mut ready = false;
    for step in 1..=steps {
        match <crate::glut_system::PreparedGlutMembrane as Membrane>::advance(&mut membrane)? {
            MembraneProgress::Open => output.push_str(&format!("advance {step}: open\n")),
            MembraneProgress::ReadyToClose => {
                output.push_str(&format!("advance {step}: ready-to-close\n"));
                ready = true;
                break;
            }
        }
    }
    if !ready {
        output.push_str(&format!(
            "closure=open continuation=returned-to-caller steps={} ready=false; not persisted after this process exits\n",
            steps,
        ));
        return Ok(output);
    }
    match <crate::glut_system::PreparedGlutMembrane as Membrane>::close(membrane)? {
        MembraneClosure::Closed { readout, .. } => match readout {
            Some(execution) => output.push_str(&format!(
                "closed generator=prepared-glut p={} q={} product_verified=true\n",
                decimal(&execution.p),
                decimal(&execution.q),
            )),
            None => output.push_str("closed generator=prepared-glut readout=none\n"),
        },
        MembraneClosure::Open(_continuation) => {
            output.push_str("closure=open continuation=retained\n")
        }
    }
    Ok(output)
}

fn decimal(tape: &[char]) -> String {
    let limbs = crate::morphism_factor::tape_to_limbs(tape, (tape.len() + 31) / 32);
    let mut digits = alloc::vec![0u8];
    for limb in limbs.iter().rev() {
        let mut carry = *limb as u64;
        for digit in &mut digits {
            let value = *digit as u64 * (1u64 << 32) + carry;
            *digit = (value % 10) as u8;
            carry = value / 10;
        }
        while carry != 0 {
            digits.push((carry % 10) as u8);
            carry /= 10;
        }
    }
    digits.iter().rev().map(|digit| (b'0' + *digit) as char).collect()
}
