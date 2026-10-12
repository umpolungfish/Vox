//! Structural multiplication reads over cell-binary Gödel numerals.
//!
//! This layer keeps the arithmetic and the glyph geometry coupled: a claimed
//! product is checked both by exact `Nat` multiplication and by multiplying the
//! two binary-support polynomials, then normalizing the resulting coefficients
//! by base-2 carry propagation. The bundled RBD corpus contains 23 exact
//! product/factor triples mined from the repository's analyzer output.

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::godel_analyzer::{bitlength, codec_assertions, residue_pow2, v2_plus_one, V2};
use crate::godel_calculus::{
    check, decode, encode_cell_binary, Family, Nat, Operator,
};
use crate::morphism_factor;

pub const RBD_PRODUCT_CORPUS: &str = include_str!("rbd_products.txt");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProductAnalysis {
    pub product: Nat,
    pub left: Nat,
    pub right: Nat,
    pub exact_product: bool,
    pub codec_assertions: bool,
    pub convolution_normalizes: bool,
    pub carry_positions: Nat,
    pub carry_units: Nat,
    pub product_bitlength: Nat,
    pub left_bitlength: Nat,
    pub right_bitlength: Nat,
    pub low_width: Nat,
    pub low_product_residue: Nat,
    pub low_left_residue: Nat,
    pub low_right_residue: Nat,
    pub low_factor_product_residue: Nat,
    pub low_residue_relation: bool,
}

fn bits_msb(value: &Nat) -> String {
    value
        .bits_le()
        .iter()
        .rev()
        .map(|bit| if *bit { '1' } else { '0' })
        .collect()
}

fn bits_le(value: &Nat) -> String {
    value
        .bits_le()
        .iter()
        .map(|bit| if *bit { '1' } else { '0' })
        .collect()
}

fn nat_from_index(mut value: usize) -> Nat {
    let mut bits = Vec::new();
    while value != 0 {
        bits.push(value & 1 == 1);
        value >>= 1;
    }
    Nat::from_bits_le(bits)
}

fn half(value: &Nat) -> Nat {
    if value.bits_le().len() <= 1 {
        Nat::zero()
    } else {
        Nat::from_bits_le(value.bits_le()[1..].to_vec())
    }
}

fn support_convolution(left: &Nat, right: &Nat) -> Vec<Nat> {
    if left.is_zero() || right.is_zero() {
        return Vec::new();
    }
    let len = left.bits_le().len() + right.bits_le().len() - 1;
    let mut coeffs = alloc::vec![Nat::zero(); len];
    let one = Nat::one();
    for (i, l) in left.bits_le().iter().copied().enumerate() {
        if !l {
            continue;
        }
        for (j, r) in right.bits_le().iter().copied().enumerate() {
            if r {
                coeffs[i + j] = coeffs[i + j].add(&one);
            }
        }
    }
    coeffs
}

/// Multiply the two support polynomials and normalize their coefficients in
/// base 2. This is deliberately independent of `Nat::mul`, so it checks the
/// same product through the glyph-support geometry rather than through the
/// arithmetic implementation itself.
pub fn normalize_support_product(left: &Nat, right: &Nat) -> (Nat, Nat, Nat) {
    if left.is_zero() || right.is_zero() {
        return (Nat::zero(), Nat::zero(), Nat::zero());
    }

    let mut coeffs = support_convolution(left, right);
    // One extra cell is sufficient: a product of b_l and b_r bit values has at
    // most b_l + b_r bits, while raw convolution ends at b_l + b_r - 2.
    coeffs.push(Nat::zero());

    let mut bits = Vec::with_capacity(coeffs.len());
    let mut carry_positions = Nat::zero();
    let mut carry_units = Nat::zero();
    let one = Nat::one();

    for i in 0..coeffs.len() - 1 {
        bits.push(coeffs[i].bits_le().first().copied().unwrap_or(false));
        let carry = half(&coeffs[i]);
        if !carry.is_zero() {
            carry_positions = carry_positions.add(&one);
            carry_units = carry_units.add(&carry);
            coeffs[i + 1] = coeffs[i + 1].add(&carry);
        }
    }
    bits.push(
        coeffs
            .last()
            .and_then(|n| n.bits_le().first())
            .copied()
            .unwrap_or(false),
    );

    (Nat::from_bits_le(bits), carry_positions, carry_units)
}

fn parse_input(raw: &str) -> Result<Nat, String> {
    if raw.starts_with('⊢') {
        let reading = decode(raw).map_err(|e| e.to_string())?;
        if reading.family != Family::CellBinary {
            return Err("product accepts cell-binary words or decimal naturals".to_string());
        }
        Ok(reading.value)
    } else {
        Nat::from_decimal(raw)
            .ok_or_else(|| format!("not a natural number or cell-binary word: {raw}"))
    }
}

fn codec_ok(value: &Nat) -> Result<bool, String> {
    let word = encode_cell_binary(value);
    Ok(codec_assertions(value, &word)?.all())
}

pub fn analyze_product(product: &Nat, left: &Nat, right: &Nat) -> Result<ProductAnalysis, String> {
    let exact = left.mul(right);
    let (normalized, carry_positions, carry_units) = normalize_support_product(left, right);

    let low_width = match v2_plus_one(product) {
        V2::Finite(k) => k,
        V2::Infinity => return Err("unexpected infinite v2(product+1)".to_string()),
    };
    let low_product_residue = residue_pow2(product, &low_width);
    let low_left_residue = residue_pow2(left, &low_width);
    let low_right_residue = residue_pow2(right, &low_width);
    let low_factor_product_residue =
        residue_pow2(&low_left_residue.mul(&low_right_residue), &low_width);

    Ok(ProductAnalysis {
        product: product.clone(),
        left: left.clone(),
        right: right.clone(),
        exact_product: exact == *product,
        codec_assertions: codec_ok(product)? && codec_ok(left)? && codec_ok(right)?,
        convolution_normalizes: normalized == *product,
        carry_positions,
        carry_units,
        product_bitlength: bitlength(product),
        left_bitlength: bitlength(left),
        right_bitlength: bitlength(right),
        low_width,
        low_product_residue: low_product_residue.clone(),
        low_left_residue,
        low_right_residue,
        low_factor_product_residue: low_factor_product_residue.clone(),
        low_residue_relation: low_factor_product_residue == low_product_residue,
    })
}

pub fn render(analysis: &ProductAnalysis) -> String {
    let pass = |x: bool| if x { "PASS" } else { "FAIL" };
    format!(
        "product                    {}\n\
         binary.product             {}\n\
         bits-le.product            {}\n\
         left                       {}\n\
         binary.left                {}\n\
         bits-le.left               {}\n\
         right                      {}\n\
         binary.right               {}\n\
         bits-le.right              {}\n\
         relation.exact-product     {}\n\
         relation.codec-assertions  {}\n\
         relation.support-carry     {}\n\
         carry.positions            {}\n\
         carry.units                {}\n\
         bitlength.product          {}\n\
         bitlength.left             {}\n\
         bitlength.right            {}\n\
         2adic.width=v2(product+1)  {}\n\
         2adic.product-residue      {}\n\
         2adic.left-residue         {}\n\
         2adic.right-residue        {}\n\
         2adic.factor-product       {}\n\
         2adic.residue-relation     {}\n",
        analysis.product,
        bits_msb(&analysis.product),
        bits_le(&analysis.product),
        analysis.left,
        bits_msb(&analysis.left),
        bits_le(&analysis.left),
        analysis.right,
        bits_msb(&analysis.right),
        bits_le(&analysis.right),
        pass(analysis.exact_product),
        pass(analysis.codec_assertions),
        pass(analysis.convolution_normalizes),
        analysis.carry_positions,
        analysis.carry_units,
        analysis.product_bitlength,
        analysis.left_bitlength,
        analysis.right_bitlength,
        analysis.low_width,
        analysis.low_product_residue,
        analysis.low_left_residue,
        analysis.low_right_residue,
        analysis.low_factor_product_residue,
        pass(analysis.low_residue_relation),
    )
}

pub fn help_addendum() -> &'static str {
    "godel product <product> <left-factor> <right-factor>\n\
     godel frame <value>\n\
     godel shiab <value> [left-factor right-factor]\n\
     godel separate <semiprime>\n\
     godel factor <value>\n"
}

fn shiab_delta(value: &[char]) -> (Vec<char>, Vec<char>, Vec<char>, usize) {
    let mut even = Vec::with_capacity((value.len() + 1) / 2);
    let mut odd = Vec::with_capacity(value.len() / 2);
    for (index, bit) in value.iter().copied().enumerate() {
        if index % 2 == 0 {
            even.push(bit);
        } else {
            odd.push(bit);
        }
    }
    let width = even.len();
    let even_lane = morphism_factor::trim(even);
    let odd_lane = morphism_factor::trim(odd);
    let mut shifted_odd = alloc::vec!['⊤'; width];
    shifted_odd.extend_from_slice(&odd_lane);
    let boundary = morphism_factor::add(&even_lane, &shifted_odd);
    (boundary, even_lane, odd_lane, value.len())
}

fn shiab_mu(boundary: &[char], source_width: usize) -> Vec<char> {
    let even_width = (source_width + 1) / 2;
    let mut restored = alloc::vec!['⊤'; source_width];
    for index in 0..source_width {
        restored[index] = if index % 2 == 0 {
            boundary.get(index / 2).copied().unwrap_or('⊤')
        } else {
            boundary
                .get(even_width + index / 2)
                .copied()
                .unwrap_or('⊤')
        };
    }
    morphism_factor::trim(restored)
}

fn shiab_report(value: &Nat, factors: Option<(&Nat, &Nat)>) -> Result<String, String> {
    let bulk = imasm_tape(value);
    let (boundary, even, odd, width) = shiab_delta(&bulk);
    let recovered = shiab_mu(&boundary, width);
    let boundary_value = Nat::from_bits_le(boundary.iter().map(|bit| *bit == '⊥').collect());
    let mut out = format!(
        "bulk                    {}\n\
         bulk.binary             {}\n\
         bulk.bits-le            {}\n\
         bulk.word               {}\n\
         boundary                {}\n\
         boundary.binary         {}\n\
         boundary.bits-le        {}\n\
         boundary.word           {}\n\
         δ.even-lane              {}\n\
         δ.odd-lane               {}\n\
         frame.width             {}\n\
         μ(δ(bulk))               {}\n\
         closure                  {}\n",
        value,
        bits_msb(value),
        bits_le(value),
        encode_cell_binary(value),
        morphism_factor::dec_of(&boundary),
        bits_msb(&boundary_value),
        bits_le(&boundary_value),
        encode_cell_binary(&boundary_value),
        morphism_factor::dec_of(&even),
        morphism_factor::dec_of(&odd),
        width,
        morphism_factor::dec_of(&recovered),
        if recovered == bulk { "PASS" } else { "FAIL" },
    );
    if let Some((left, right)) = factors {
        let left_tape = imasm_tape(left);
        let right_tape = imasm_tape(right);
        if morphism_factor::mul(&left_tape, &right_tape) != bulk {
            return Err("supplied factor values do not multiply to the SHIAB bulk".to_string());
        }
        let (left_boundary, left_even, left_odd, left_width) = shiab_delta(&left_tape);
        let (right_boundary, right_even, right_odd, right_width) = shiab_delta(&right_tape);
        let factor_boundary_product = morphism_factor::mul(&left_boundary, &right_boundary);
        out.push_str(&format!(
            "factor.left              {}\n\
             factor.left.boundary     {}\n\
             factor.left.lanes        {} | {}\n\
             factor.left.width        {}\n\
             factor.right             {}\n\
             factor.right.boundary    {}\n\
             factor.right.lanes       {} | {}\n\
             factor.right.width       {}\n\
             boundary.factor-product {}\n\
             boundary.product-equal  {}\n\
             factor.return.product   {}\n\
             factor.closure           PASS\n",
            left,
            morphism_factor::dec_of(&left_boundary),
            morphism_factor::dec_of(&left_even),
            morphism_factor::dec_of(&left_odd),
            left_width,
            right,
            morphism_factor::dec_of(&right_boundary),
            morphism_factor::dec_of(&right_even),
            morphism_factor::dec_of(&right_odd),
            right_width,
            morphism_factor::dec_of(&factor_boundary_product),
            if factor_boundary_product == boundary { "true" } else { "false" },
            morphism_factor::dec_of(&morphism_factor::mul(&left_tape, &right_tape)),
        ));
    }
    Ok(out)
}

fn imasm_tape(value: &Nat) -> Vec<char> {
    value
        .bits_le()
        .iter()
        .map(|bit| if *bit { '⊥' } else { '⊤' })
        .collect()
}

#[derive(Clone)]
struct ResidualFactorFrame {
    shift: usize,
    shifts_tested: usize,
    a: Nat,
    b: Nat,
    product: Nat,
    residual_magnitude: Nat,
    common: Nat,
}

fn residual_factor_frame(product: &Nat) -> Option<ResidualFactorFrame> {
    let bits = product.bits_le();
    if bits.is_empty() {
        return None;
    }
    let source = imasm_tape(product);
    let mut first = None;
    let mut first_proper = None;
    for shift in 0..bits.len() {
        let mut even = Vec::with_capacity((bits.len() + 1) / 2);
        let mut odd = Vec::with_capacity(bits.len() / 2);
        for index in 0..bits.len() {
            let bit = bits[(index + shift) % bits.len()];
            let lane = if bit { '⊥' } else { '⊤' };
            if index % 2 == 0 {
                even.push(lane);
            } else {
                odd.push(lane);
            }
        }
        let even = morphism_factor::trim(even);
        let odd = morphism_factor::trim(odd);
        let lane_product = morphism_factor::mul(&even, &odd);
        let residual = if morphism_factor::cmp(&source, &lane_product) == core::cmp::Ordering::Greater {
            morphism_factor::sub(&source, &lane_product)
        } else {
            morphism_factor::sub(&lane_product, &source)
        };
        let common = morphism_factor::gcd(residual.clone(), source.clone());
        let frame = ResidualFactorFrame {
            shift,
            shifts_tested: shift + 1,
            a: Nat::from_decimal(&morphism_factor::dec_of(&even))?,
            b: Nat::from_decimal(&morphism_factor::dec_of(&odd))?,
            product: Nat::from_decimal(&morphism_factor::dec_of(&lane_product))?,
            residual_magnitude: Nat::from_decimal(&morphism_factor::dec_of(&residual))?,
            common: Nat::from_decimal(&morphism_factor::dec_of(&common))?,
        };
        if first.is_none() {
            first = Some(frame.clone());
        }
        let proper = common.len() > 1
            && morphism_factor::cmp(&common, &source) == core::cmp::Ordering::Less;
        if proper && first_proper.is_none() {
            first_proper = Some(frame.clone());
        }
        if proper
            && morphism_factor::cmp(&common, &source) == core::cmp::Ordering::Less
            && morphism_factor::miller_rabin(&common)
            && morphism_factor::modulo(&source, &common) == alloc::vec!['⊤']
        {
            return Some(frame);
        }
    }
    let mut frame = first_proper.or(first)?;
    frame.shifts_tested = bits.len();
    Some(frame)
}

fn residual_frame_closes(product: &Nat, frame: &ResidualFactorFrame) -> bool {
    let source = imasm_tape(product);
    let common = imasm_tape(&frame.common);
    morphism_factor::cmp(&common, &alloc::vec!['⊥']) == core::cmp::Ordering::Greater
        && morphism_factor::cmp(&common, &source) == core::cmp::Ordering::Less
        && morphism_factor::miller_rabin(&common)
        && morphism_factor::modulo(&source, &common) == alloc::vec!['⊤']
}

fn residual_frame_report(frame: &ResidualFactorFrame, closed: bool) -> String {
    format!(
        "frame.shift             {}\n\
         frame.shifts-tested     {}\n\
         frame.a                 {}\n\
         frame.b                 {}\n\
         frame.ab                {}\n\
         frame.r.magnitude       {}\n\
         frame.gcd               {}\n\
         frame.prime-factor      {}\n",
        frame.shift,
        frame.shifts_tested,
        frame.a,
        frame.b,
        frame.product,
        frame.residual_magnitude,
        frame.common,
        if closed { "PASS" } else { "open" },
    )
}

fn separate_lanes(product: &Nat) -> Result<(Nat, Nat), String> {
    let source = imasm_tape(product);
    if source.len() < 2 {
        return Err("Gödel separation needs a value wider than one cell".to_string());
    }
    let source_word = morphism_factor::emit_numeral(&source);
    let (left_word, right_word) = if source.first() == Some(&'⊤') {
        let shifted = morphism_factor::trim(source[1..].to_vec());
        (morphism_factor::emit_numeral(&alloc::vec!['⊤', '⊥']), morphism_factor::emit_numeral(&shifted))
    } else {
        // Separate the product through the source tape's LSB-first carry
        // relation. The result must return through the common IMASM product
        // check below before it can leave this boundary.
        let execution = crate::glut_system::glut_correlation_execution(&source)
            .ok_or_else(|| "nested IMASM bitregister correlation did not close".to_string())?;
        (
            morphism_factor::emit_numeral(&execution.p),
            morphism_factor::emit_numeral(&execution.q),
        )
    };
    let left_tape = morphism_factor::parse_numeral(&left_word)?;
    let right_tape = morphism_factor::parse_numeral(&right_word)?;
    let return_check = morphism_factor::verify(&left_word, &right_word, &source_word)?;
    if !return_check.starts_with("p*q == N: true\n") {
        return Err("separated prime lanes failed IMASM return closure".to_string());
    }
    let left_nat = Nat::from_decimal(&morphism_factor::dec_of(&left_tape))
        .ok_or_else(|| "left IMASM lane did not return a natural numeral".to_string())?;
    let right_nat = Nat::from_decimal(&morphism_factor::dec_of(&right_tape))
        .ok_or_else(|| "right IMASM lane did not return a natural numeral".to_string())?;
    Ok((left_nat, right_nat))
}

fn separate_product(product: &Nat) -> Result<String, String> {
    let source_tape = imasm_tape(product);
    let source_word = encode_cell_binary(product);
    let frame_state = residual_factor_frame(product);
    let residual_report = frame_state
        .as_ref()
        .map(|frame| residual_frame_report(frame, residual_frame_closes(product, frame)))
        .unwrap_or_default();
    let (factor, method, membrane_diagnostic) = if let Some(frame) = frame_state
        .as_ref()
        .filter(|frame| residual_frame_closes(product, frame))
    {
        (frame.common.clone(), "residual-gcd", String::new())
    } else {
        let (execution, stats) = crate::glut_system::glut_factor_execution_with_stats(&source_tape);
        let execution = execution
            .ok_or_else(|| "nested IMASM membrane did not return a prime divisor".to_string())?;
        execution.verify(&source_tape)?;
        let factor_tape = [execution.p, execution.q]
            .into_iter()
            .find(|candidate| {
                morphism_factor::miller_rabin(candidate)
                    && morphism_factor::modulo(&source_tape, candidate) == alloc::vec!['⊤']
            })
            .ok_or_else(|| "nested IMASM return had no prime-divisor lane".to_string())?;
        let factor = Nat::from_decimal(&morphism_factor::dec_of(&factor_tape))
            .ok_or_else(|| "prime-divisor tape did not decode as a natural".to_string())?;
        let diagnostic = format!(
            "glut.decisions           {}\n\
             glut.peak-frontier       {}\n\
             glut.peak-cells          {}\n\
             glut.rejected            {}\n\
             glut.splits              {}\n\
             glut.correlation-decisions {}\n\
             glut.correlation-conflicts {}\n\
             glut.square-advances     {}\n\
             glut.square-cells        {}\n\
             glut.root-marks          {}\n\
             glut.root-link-visits    {}\n\
             glut.root-links          {}\n\
             glut.root-candidates     {}\n\
             glut.mask-work           {}\n\
             glut.correlation-work    {}\n",
            stats.decisions,
            stats.peak_frontier,
            stats.peak_cells,
            stats.rejected,
            stats.splits,
            stats.correlation_decisions,
            stats.correlation_conflicts,
            stats.square_advances,
            stats.square_cells,
            stats.square_root_marks,
            stats.square_root_link_visits,
            stats.square_root_links,
            stats.square_root_candidates,
            stats.mask_work,
            stats.correlation_work,
        );
        (factor, "nested-return", diagnostic)
    };
    let factor_tape = imasm_tape(&factor);
    if !morphism_factor::miller_rabin(&factor_tape)
        || morphism_factor::modulo(&source_tape, &factor_tape) != alloc::vec!['⊤']
    {
        return Err("single prime-factor IMASM return failed".to_string());
    }
    let factor_word = encode_cell_binary(&factor);
    Ok(format!(
        "source                  {}\n\
         source.binary           {}\n\
         source.word             {}\n\
         extraction              IMASM one-factor frame separation\n\
         factor.frame            {}\n\
         {}\
         {}\
         prime.factor            {}\n\
         factor.word             {}\n\
         factor.prime            PASS\n\
         factor.remainder        0\n\
         factor.return-closure   PASS\n\
         source.frames           {}\n\
         factor.frames           {}\n",
        product,
        bits_msb(product),
        source_word,
        method,
        residual_report,
        membrane_diagnostic,
        factor,
        factor_word,
        product.bits_le().len(),
        factor.bits_le().len(),
    ))
}

fn collect_prime_factors(value: &Nat, out: &mut Vec<Nat>) -> Result<(), String> {
    if value.is_zero() || value == &Nat::one() {
        return Ok(());
    }
    let tape = imasm_tape(value);
    if morphism_factor::miller_rabin(&tape) {
        out.push(value.clone());
        return Ok(());
    }
    let (left, right) = separate_lanes(value)?;
    collect_prime_factors(&left, out)?;
    collect_prime_factors(&right, out)
}

fn factor_product(product: &Nat) -> Result<String, String> {
    let mut factors = Vec::new();
    collect_prime_factors(product, &mut factors)?;
    if factors.is_empty() {
        return Err("prime factorization needs a value greater than one".to_string());
    }
    let product_of_factors = factors.iter().fold(Nat::one(), |acc, factor| acc.mul(factor));
    if product_of_factors != *product {
        return Err("recursive prime factors failed exact product closure".to_string());
    }
    let mut report = format!("source                  {product}\nprime.factors.count     {}\n", factors.len());
    let mut accumulated = Nat::one();
    let mut accumulated_word = encode_cell_binary(&accumulated);
    for (index, factor) in factors.iter().enumerate() {
        let word = encode_cell_binary(factor);
        let decoded = decode(&word).map_err(|error| error.to_string())?;
        if decoded.value != *factor || !morphism_factor::miller_rabin(&imasm_tape(factor)) {
            return Err("encoded factor failed the prime lane check".to_string());
        }
        let next = accumulated.mul(factor);
        let next_word = encode_cell_binary(&next);
        let encoded_closure = check(&accumulated_word, Operator::Mul, &word, &next_word)
            .map_err(|error| error.to_string())?;
        let convolution = analyze_product(&next, &accumulated, factor)?;
        if !encoded_closure.valid || !convolution.convolution_normalizes {
            return Err("recursive factor encoding failed multiplication closure".to_string());
        }
        report.push_str(&format!(
            "prime.factor.{}          {}\nprime.factor.{}.word     {}\n",
            index + 1,
            factor,
            index + 1,
            word
        ));
        accumulated = next;
        accumulated_word = next_word;
    }
    if accumulated != *product {
        return Err("recursive encoded product differs from source".to_string());
    }
    report.push_str("godel.check.mul        PASS\nconvolution.normalize  PASS\nrecursive.product       PASS\n");
    Ok(report)
}

pub fn command(args: &[&str]) -> Result<String, String> {
    if args.first().copied() == Some("frame") {
        if args.len() != 2 {
            return Err("godel frame <value>".to_string());
        }
        let value = parse_input(args[1])?;
        let frame = residual_factor_frame(&value)
            .ok_or_else(|| "value has no Gödel bit frame".to_string())?;
        return Ok(format!(
            "source                  {}\n{}",
            value,
            residual_frame_report(&frame, residual_frame_closes(&value, &frame))
        ));
    }
    if args.first().copied() == Some("factor") {
        if args.len() != 2 {
            return Err("godel factor <value>".to_string());
        }
        return factor_product(&parse_input(args[1])?);
    }
    if args.first().copied() == Some("shiab") {
        if !(2..=4).contains(&args.len()) || args.len() == 3 {
            return Err("godel shiab <value> [left-factor right-factor]".to_string());
        }
        let value = parse_input(args[1])?;
        let factors = if args.len() == 4 {
            Some((parse_input(args[2])?, parse_input(args[3])?))
        } else {
            None
        };
        return shiab_report(&value, factors.as_ref().map(|(left, right)| (left, right)));
    }
    if args.first().copied() == Some("separate") {
        if args.len() != 2 {
            return Err("godel separate <product>".to_string());
        }
        let product = parse_input(args[1])?;
        return separate_product(&product);
    }
    if args.first().copied() != Some("product") || args.len() != 4 {
        return Err("godel product <product> <left-factor> <right-factor> | godel separate <product>".to_string());
    }
    let product = parse_input(args[1])?;
    let left = parse_input(args[2])?;
    let right = parse_input(args[3])?;
    Ok(render(&analyze_product(&product, &left, &right)?))
}

fn corpus_triples() -> Result<Vec<(Nat, Nat, Nat)>, String> {
    let mut out = Vec::new();
    for (line_no, line) in RBD_PRODUCT_CORPUS.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let mut parts = line.split('|');
        let n = parts
            .next()
            .and_then(Nat::from_decimal)
            .ok_or_else(|| format!("invalid RBD product at line {}", line_no + 1))?;
        let p = parts
            .next()
            .and_then(Nat::from_decimal)
            .ok_or_else(|| format!("invalid RBD left factor at line {}", line_no + 1))?;
        let q = parts
            .next()
            .and_then(Nat::from_decimal)
            .ok_or_else(|| format!("invalid RBD right factor at line {}", line_no + 1))?;
        if parts.next().is_some() {
            return Err(format!("too many RBD fields at line {}", line_no + 1));
        }
        out.push((n, p, q));
    }
    Ok(out)
}

pub fn selftest_report() -> Result<String, String> {
    let triples = corpus_triples()?;
    let total = nat_from_index(triples.len());
    let mut passed = Nat::zero();
    let one = Nat::one();
    let mut all_ok = true;

    for (n, p, q) in triples {
        let analysis = analyze_product(&n, &p, &q)?;
        let ok = analysis.exact_product
            && analysis.codec_assertions
            && analysis.convolution_normalizes
            && analysis.low_residue_relation;
        if ok {
            passed = passed.add(&one);
        } else {
            all_ok = false;
        }
    }

    let report = format!(
        "rbd-product {passed}/{total} exact product+support-carry triples  {}\n",
        if all_ok { "PASS" } else { "FAIL" }
    );
    if all_ok {
        Ok(report)
    } else {
        Err(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn small_product_normalizes_support() {
        let n = Nat::from_u64(21);
        let p = Nat::from_u64(3);
        let q = Nat::from_u64(7);
        let analysis = analyze_product(&n, &p, &q).unwrap();
        assert!(analysis.exact_product);
        assert!(analysis.codec_assertions);
        assert!(analysis.convolution_normalizes);
        assert!(analysis.low_residue_relation);
    }

    #[test]
    fn supplied_binary_examples_report_product_and_reversed_tapes() {
        for (n, p, q, binary_n, reversed_n) in [
            (143, 11, 13, "10001111", "11110001"),
            (851, 23, 37, "1101010011", "1100101011"),
        ] {
            let analysis = analyze_product(
                &Nat::from_u64(n),
                &Nat::from_u64(p),
                &Nat::from_u64(q),
            )
            .unwrap();
            assert!(analysis.exact_product);
            assert!(analysis.convolution_normalizes);
            let report = render(&analysis);
            assert!(report.contains(&format!("binary.product             {binary_n}")));
            assert!(report.contains(&format!("bits-le.product            {reversed_n}")));
        }
    }

    #[test]
    fn unbraid_accepts_an_unequal_width_semiprime() {
        let report = separate_product(&Nat::from_u64(10873)).unwrap();
        assert!(report.contains("prime.factor            83")
            || report.contains("prime.factor            131"));
        assert!(report.contains("factor.return-closure   PASS"));
    }

    #[test]
    fn separation_closes_even_semiprimes_and_recursive_factor_trees() {
        let semiprime = separate_product(&Nat::from_u64(1994)).unwrap();
        assert!(semiprime.contains("prime.factor            2"));
        assert!(semiprime.contains("factor.return-closure   PASS"));
        let composite = factor_product(&Nat::from_u64(12)).unwrap();
        assert!(composite.contains("prime.factor.1          2"));
        assert!(composite.contains("prime.factor.2          2"));
        assert!(composite.contains("prime.factor.3          3"));
        assert!(composite.contains("recursive.product       PASS"));
    }

    #[test]
    fn shiab_frames_match_the_supplied_number_collapse_and_return_exactly() {
        for (n, boundary) in [(143, 179), (851, 573)] {
            let value = Nat::from_u64(n);
            let tape = imasm_tape(&value);
            let (collapsed, _, _, width) = shiab_delta(&tape);
            assert_eq!(morphism_factor::dec_of(&collapsed), boundary.to_string());
            assert_eq!(shiab_mu(&collapsed, width), tape);
        }
    }

    #[test]
    fn shiab_factor_readings_keep_binary_product_closure_explicit() {
        for (n, p, q) in [(143, 11, 13), (851, 23, 37)] {
            let report = shiab_report(
                &Nat::from_u64(n),
                Some((&Nat::from_u64(p), &Nat::from_u64(q))),
            )
            .unwrap();
            assert!(report.contains(&format!("factor.return.product   {n}")));
            assert!(report.contains("factor.closure           PASS"));
        }
    }

    #[test]
    fn rbd_corpus_is_23_exact_product_triples() {
        let triples = corpus_triples().unwrap();
        assert_eq!(triples.len(), 23);
        for (n, p, q) in triples {
            let analysis = analyze_product(&n, &p, &q).unwrap();
            assert!(analysis.exact_product);
            assert!(analysis.codec_assertions);
            assert!(analysis.convolution_normalizes);
            assert!(analysis.low_residue_relation);
        }
    }
}
