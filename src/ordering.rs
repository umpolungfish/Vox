//! Explicit, reversible order selection and lazy families of every positional order.
use alloc::{format, string::{String, ToString}, vec::Vec};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Permutation { pub indices: Vec<usize> }

impl Permutation {
    pub fn checked(indices: Vec<usize>, width: usize) -> Result<Self, String> {
        let mut seen = alloc::vec![false; width];
        if indices.len() != width { return Err(format!("order has {} positions, expected {width}", indices.len())); }
        for &i in &indices {
            if i >= width || seen[i] { return Err("order must contain every input position exactly once".into()); }
            seen[i] = true;
        }
        Ok(Self { indices })
    }
    pub fn parse(spec: &str, width: usize) -> Result<Self, String> {
        if let Some(path) = spec.strip_prefix("file:") {
            #[cfg(feature = "hosted")]
            {
                let contents = std::fs::read_to_string(path).map_err(|error| format!("read order file: {error}"))?;
                return Self::parse(contents.trim(), width);
            }
            #[cfg(not(feature = "hosted"))]
            return Err("file-backed orders require the hosted feature".into());
        }
        match spec {
            "canonical" | "forward" => Ok(Self { indices: (0..width).collect() }),
            "reverse" | "inverse" => Ok(Self { indices: (0..width).rev().collect() }),
            "all" => Err("all orders form a family; use the ordering CLI or AllOrders iterator".into()),
            _ if spec.starts_with("swap:") => {
                let pair = spec.strip_prefix("swap:").unwrap().split(',').map(|s| s.parse::<usize>().map_err(|_| format!("invalid swap position {s}"))).collect::<Result<Vec<_>, _>>()?;
                if pair.len() != 2 || pair[0] == pair[1] || pair.iter().any(|&position| position >= width) {
                    return Err("swap order requires two distinct in-range positions".into());
                }
                let mut indices: Vec<_> = (0..width).collect();
                indices.swap(pair[0], pair[1]);
                Self::checked(indices, width)
            }
            _ => Self::checked(spec.strip_prefix("perm:").unwrap_or(spec).split(',')
                .filter(|s| !s.is_empty()).map(|s| s.parse::<usize>()
                    .map_err(|_| format!("invalid order position {s}"))).collect::<Result<_, _>>()?, width),
        }
    }
    pub fn apply<T: Clone>(&self, input: &[T]) -> Result<Vec<T>, String> {
        Self::checked(self.indices.clone(), input.len())?;
        Ok(self.indices.iter().map(|&i| input[i].clone()).collect())
    }
    pub fn inverse(&self) -> Self {
        let mut indices = alloc::vec![0; self.indices.len()];
        for (position, &source) in self.indices.iter().enumerate() { indices[source] = position; }
        Self { indices }
    }
    pub fn spec(&self) -> String {
        if self.is_canonical() { return "canonical".into(); }
        if self.is_reverse() { return "reverse".into(); }
        let moved: Vec<_> = self.indices.iter().enumerate().filter(|(i, value)| *i != **value).map(|(i, value)| (i, *value)).collect();
        if moved.len() == 2 && moved[0].0 == moved[1].1 && moved[1].0 == moved[0].1 {
            return format!("swap:{},{}", moved[0].0, moved[1].0);
        }
        self.indices.iter().map(ToString::to_string).collect::<Vec<_>>().join(",")
    }
    pub fn label(&self) -> String {
        if self.is_canonical() { return "canonical".into(); }
        if self.is_reverse() { return "reverse".into(); }
        let moved = self.indices.iter().enumerate().filter(|(i, value)| *i != **value).count();
        if moved == 2 { return self.spec(); }
        let mut identity = 0xcbf29ce484222325u64;
        for index in &self.indices {
            for byte in (*index as u64).to_le_bytes() {
                identity = (identity ^ u64::from(byte)).wrapping_mul(0x100000001b3);
            }
        }
        format!("permutation(width={},moved={moved},id={identity:016x})", self.indices.len())
    }
    fn is_canonical(&self) -> bool { self.indices.iter().enumerate().all(|(i, value)| i == *value) }
    fn is_reverse(&self) -> bool {
        self.indices.iter().enumerate().all(|(i, value)| *value == self.indices.len() - 1 - i)
    }
}

/// No factorial allocation or width ceiling. Equal payloads retain their distinct positions.
pub struct AllOrders { next: Option<Permutation> }
impl AllOrders {
    pub fn new(width: usize) -> Self { Self { next: Some(Permutation { indices: (0..width).collect() }) } }
    pub fn from_order(order: Permutation) -> Self { Self { next: Some(order) } }
}
impl Iterator for AllOrders {
    type Item = Permutation;
    fn next(&mut self) -> Option<Self::Item> {
        let current = self.next.take()?;
        let mut next = current.indices.clone();
        if let Some(pivot) = (1..next.len()).rev().find(|&i| next[i - 1] < next[i]).map(|i| i - 1) {
            let last = (pivot + 1..next.len()).rev().find(|&i| next[pivot] < next[i]).unwrap();
            next.swap(pivot, last);
            next[pivot + 1..].reverse();
            self.next = Some(Permutation { indices: next });
        }
        Some(current)
    }
}

pub fn setting(name: &str) -> Option<String> {
    #[cfg(feature = "hosted")]
    if let Ok(value) = std::env::var(name) { return Some(value); }
    match name {
        "IG_ORDER_SITE" => option_env!("IG_ORDER_SITE"),
        "IG_ORDER_SPEC" => option_env!("IG_ORDER_SPEC"),
        _ => None,
    }.map(ToString::to_string)
}

pub fn selected(site: &str, width: usize) -> Result<Permutation, String> {
    if setting("IG_ORDER_SITE").as_deref() != Some(site) {
        return Permutation::parse("canonical", width);
    }
    #[cfg(feature = "hosted")]
    if std::env::var_os("IG_ORDER_DISCOVER").is_some() {
        std::println!("ORDER_SHAPE={site} {width}");
        std::process::exit(0);
    }
    let spec = setting("IG_ORDER_SPEC").unwrap_or_else(|| "canonical".into());
    let order = Permutation::parse(&spec, width)?;
    #[cfg(feature = "hosted")]
    {
        use std::sync::{Mutex, OnceLock};
        static REPORTED: OnceLock<Mutex<alloc::collections::BTreeSet<String>>> = OnceLock::new();
        let key = format!("{site}:{}", order.spec());
        if REPORTED.get_or_init(|| Mutex::new(alloc::collections::BTreeSet::new())).lock().unwrap().insert(key) {
            std::eprintln!("ORDER_APPLIED site={site} selected={} inverse={}", order.label(), order.inverse().label());
        }
    }
    Ok(order)
}

pub fn selected_braid(word: &[i32]) -> Result<Vec<i32>, String> {
    let order = selected("braid", word.len())?;
    let mut result = order.apply(word)?;
    if setting("IG_ORDER_SITE").as_deref() == Some("braid")
        && setting("IG_ORDER_SPEC").as_deref() == Some("inverse") {
        for crossing in &mut result { *crossing = crossing.checked_neg().ok_or("crossing inverse overflows")?; }
    }
    Ok(result)
}

#[cfg(feature = "hosted")]
pub mod cli {
    use super::*;
    use std::{path::PathBuf, process::{Command, Stdio}};
    use core::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

    static SPEC_FILE_COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn environment_spec(spec: &str) -> Result<(String, Option<PathBuf>), String> {
        if spec.len() <= 4096 { return Ok((spec.to_string(), None)); }
        let serial = SPEC_FILE_COUNTER.fetch_add(1, AtomicOrdering::Relaxed);
        let path = std::env::current_dir().map_err(|error| format!("read current directory: {error}"))?
            .join(format!(".imsgct-order-{}-{serial}.txt", std::process::id()));
        std::fs::write(&path, spec).map_err(|error| format!("write order file: {error}"))?;
        Ok((format!("file:{}", path.display()), Some(path)))
    }

    /// Shared by every executable entry: flags select a real operation boundary.
    pub fn intercept() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        if !args.iter().any(|s| s == "--ordering" || s == "--orderings") { return; }
        match run(args) { Ok(code) => std::process::exit(code), Err(error) => { std::eprintln!("ordering: {error}"); std::process::exit(2); } }
    }
    fn run(args: Vec<String>) -> Result<i32, String> {
        let mut forwarded = Vec::new();
        let (mut spec, mut site, mut limit, mut start) = (String::new(), None, None, None);
        let mut i = 0;
        while i < args.len() {
            let value = |i: usize| args.get(i + 1).cloned().ok_or_else(|| format!("{} needs a value", args[i]));
            match args[i].as_str() {
                "--ordering" | "--orderings" => { spec = value(i)?; i += 1; }
                "--order-site" => { site = Some(value(i)?); i += 1; }
                "--order-limit" => { limit = Some(value(i)?.parse::<usize>().map_err(|_| "invalid order limit")?); i += 1; }
                "--order-start" => { start = Some(value(i)?); i += 1; }
                _ => forwarded.push(args[i].clone()),
            }
            i += 1;
        }
        let joined = forwarded.join(" ");
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        let executable_name = executable.file_stem().and_then(|name| name.to_str()).unwrap_or_default();
        let site = site.unwrap_or_else(|| {
            let tokens: Vec<_> = joined.split_whitespace().collect();
            if let Some(position) = tokens.windows(2).position(|pair| pair == ["gpu_kernel", "interleave_deep"]) {
                let lanes = tokens.iter().skip(position + 3).take_while(|token| token.contains(':')).count().clamp(1, 4);
                return format!("gpu-lanes-{lanes}");
            }
            if executable_name == "braid_order_probe"
                || executable_name == "anyon_stack_audit" { "braid-protocol" }
            else if executable_name == "godel_frame_separator" { "separating" }
            else if joined.contains("native_numeral") && joined.contains("unbraid") { "separating" }
            else if joined.contains("gpu_kernel interleave") { "gpu-lanes-3" }
            else if joined.contains("interleave") || joined.contains("interlace") { "lanes" }
            else if joined.contains("phase_unbraid") || joined.contains("phase-unbraid") { "phase-bits" }
            else if forwarded.first().is_some_and(|arg| arg == "braid")
                || joined.split_whitespace().collect::<Vec<_>>().windows(2)
                    .any(|pair| pair == ["godel", "braid"]) { "lanes" }
            else if joined.contains("proof-braider")
                || (joined.contains("compiler") && joined.contains("braid")
                    && (joined.contains("imasm") || joined.contains("lean"))) { "braid-protocol" }
            else if joined.contains("unbraid") { "lanes" }
            else if joined.contains("separat") || joined.contains("split") || joined.contains("fuse") { "separating" }
            else if forwarded.iter().any(|arg| arg == "qc" || arg.starts_with("qc ")) { "circuit" } else { "braid" }.into()
        });
        if limit == Some(0) { return Err("order limit must be positive".into()); }
        // A concrete choice is validated by the selected operation at its real
        // width. Discovering that width in a child first would compile some
        // braid programs twice, and could double the expensive synthesis path.
        if spec != "all" {
            let (spec, path) = environment_spec(&spec)?;
            let status = Command::new(&executable).args(&forwarded)
                .env("IG_ORDER_SITE", &site).env("IG_ORDER_SPEC", spec)
                .status().map_err(|error| error.to_string());
            if let Some(path) = path { let _ = std::fs::remove_file(path); }
            return status.map(|status| status.code().unwrap_or(1));
        }
        let discovered = Command::new(&executable).args(&forwarded)
            .env("IG_ORDER_SITE", &site).env("IG_ORDER_SPEC", "canonical").env("IG_ORDER_DISCOVER", "1")
            .output().map_err(|e| e.to_string())?;
        let stdout = String::from_utf8_lossy(&discovered.stdout);
        let prefix = format!("ORDER_SHAPE={site} ");
        let width = stdout.lines().find_map(|line| line.strip_prefix(&prefix).and_then(|s| s.trim().parse::<usize>().ok()))
            .ok_or_else(|| format!("site {site} was not reached: {}{}", stdout, String::from_utf8_lossy(&discovered.stderr)))?;
        let command = || {
            let mut c = Command::new(&executable);
            c.args(&forwarded).env("IG_ORDER_SITE", &site).env("IG_ORDER_WIDTH", width.to_string())
                .env_remove("IG_ORDER_DISCOVER");
            c
        };
        let first = start.map(|s| Permutation::parse(&s, width)).transpose()?
            .unwrap_or_else(|| Permutation { indices: (0..width).collect() });
        let mut orders = AllOrders::from_order(first).peekable();
        let mut count = 0usize;
        let mut failed = false;
        while let Some(order) = orders.next() {
            std::println!("ORDER_VARIANT site={site} selected={} inverse={}", order.label(), order.inverse().label());
            let (spec, path) = environment_spec(&order.spec())?;
            let status = command().env("IG_ORDER_SPEC", spec).stdin(Stdio::inherit())
                .stdout(Stdio::inherit()).stderr(Stdio::inherit()).status().map_err(|e| e.to_string());
            if let Some(path) = path { let _ = std::fs::remove_file(path); }
            let status = status?;
            std::println!("ORDER_RETURN selected={} exit={}", order.label(), status.code().unwrap_or(1));
            failed |= !status.success();
            count += 1;
            if limit == Some(count) {
                if let Some(next) = orders.peek() {
                    std::println!("ORDER_FAMILY status=held completed={count} next={}", next.label());
                    return Ok(3);
                }
            }
        }
        std::println!("ORDER_FAMILY status=complete completed={count}");
        Ok(if failed { 2 } else { 0 })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn all_orders_are_complete_and_inverse_retains_each_position() {
        let input = ["source", "phase", "return", "bank"];
        let orders: Vec<_> = AllOrders::new(input.len()).collect();
        assert_eq!(orders.len(), 24);
        let mut unique = alloc::collections::BTreeSet::new();
        for order in orders {
            let actual = order.apply(&input).unwrap();
            assert!(unique.insert(actual.clone()));
            assert_eq!(order.inverse().apply(&actual).unwrap(), input);
        }
        assert!(Permutation::parse("0,0", 2).is_err());
        assert!(Permutation::parse("0,2", 2).is_err());
        assert_eq!(Permutation::parse("reverse", 42_141).unwrap().spec(), "reverse");
        let adjacent = AllOrders::new(4).nth(1).unwrap();
        assert_eq!(adjacent.spec(), "swap:2,3");
        assert_eq!(Permutation::parse(&adjacent.spec(), 4).unwrap(), adjacent);
        assert_eq!(AllOrders::new(200).take(2).count(), 2);
        assert_eq!(AllOrders::new(0).count(), 1);
    }

    #[test]
    fn nonadjacent_order_labels_retain_permutation_identity() {
        let left = Permutation::parse("1,2,0,3", 4).unwrap();
        let right = Permutation::parse("2,0,1,3", 4).unwrap();
        assert_ne!(left.label(), right.label());
        assert_ne!(left.label(), left.inverse().label());
    }
}
