use vox::glut_factor;

fn dec(tape: &[char]) -> u64 {
    tape.iter().enumerate().map(|(i,c)| if *c=='⊥' {1u64<<i} else {0}).sum()
}

fn main() {
    // the tape as written in the test (claims to be 8051)
    let wrong = vec!['⊥','⊥','⊤','⊤','⊤','⊥','⊥','⊤','⊥','⊥','⊤','⊤','⊤'];
    // the TRUE 8051 tape, LSB-first
    let right: Vec<char> = (0..13u32).map(|i| if (8051>>i)&1==1 {'⊥'} else {'⊤'}).collect();
    println!("wrong-tape value = {}", dec(&wrong));
    println!("right-tape value = {}", dec(&right));
    let r1 = glut_factor(&wrong);
    println!("glut_factor(wrong) = {:?}", r1);
    let r2 = glut_factor(&right);
    println!("glut_factor(right) = {:?}", r2);
    if let Some((p,q)) = r2 {
        let prod = vox::morphism_factor::mul(&p,&q);
        let c = vox::morphism_factor::cmp(&prod,&right);
        println!("verify p*q==8051: {:?}  p={} q={}", c, p.iter().collect::<String>(), q.iter().collect::<String>());
    }
}
