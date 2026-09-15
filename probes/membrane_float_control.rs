fn main() {
    for k in 0..16 {
        let angle = k as f64 * core::f64::consts::PI / 8.0;
        println!("{k}: {:.12} {:.12}", angle.sin(), angle.cos());
    }
    let mut values: Vec<_> = (0..4096).map(|i| (i, ((i * 17) % 101) as f64)).collect();
    values.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for &(index, value) in &values[..8] { println!("sorted: {index} {value:.4}"); }
}
