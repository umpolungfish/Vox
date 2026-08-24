//! A machine that runs an IMASM module. Dispatch is on the glyph and nothing
//! else; what an instruction *was* in x86 survives only as payload the glyph
//! reads. Ported from imasm_vm.py. ⊞ engages the ALU, ⋈ links slots, ◻ commits
//! to memory, ⊤ makes a truth and ⊥ consumes one, ∈ splits and ∋ fuses, > calls
//! and ⊣ terminates, < transfers, ⊙ transfers through data.

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;

fn mask(size: u8) -> u128 {
    match size { 1 => 0xFF, 2 => 0xFFFF, 4 => 0xFFFF_FFFF, 8 => u64::MAX as u128, 16 => u128::MAX, _ => u64::MAX as u128 }
}
fn sign(v: u128, size: u8) -> i128 {
    let m = mask(size); let v = v & m;
    if v > m >> 1 { v as i128 - (m as i128 + 1) } else { v as i128 }
}

/// (base register name, size in bytes, byte offset within base)
fn reg_info(name: &str) -> (&'static str, u8, u8) {
    const R64: [&str;16]=["rax","rcx","rdx","rbx","rsp","rbp","rsi","rdi","r8","r9","r10","r11","r12","r13","r14","r15"];
    // 64-bit
    for &r in R64.iter() { if name == r { return (r, 8, 0); } }
    // xmm
    if let Some(rest) = name.strip_prefix("xmm") { if rest.parse::<u8>().is_ok() {
        // return the canonical static string
        const X: [&str;16]=["xmm0","xmm1","xmm2","xmm3","xmm4","xmm5","xmm6","xmm7","xmm8","xmm9","xmm10","xmm11","xmm12","xmm13","xmm14","xmm15"];
        return (X[rest.parse::<usize>().unwrap()], 16, 0);
    }}
    if name == "rip" { return ("rip", 8, 0); }
    // e** (32-bit) and r**d
    let map32: [(&str,&str);16]=[("eax","rax"),("ecx","rcx"),("edx","rdx"),("ebx","rbx"),("esp","rsp"),("ebp","rbp"),("esi","rsi"),("edi","rdi"),
        ("r8d","r8"),("r9d","r9"),("r10d","r10"),("r11d","r11"),("r12d","r12"),("r13d","r13"),("r14d","r14"),("r15d","r15")];
    for (n,b) in map32 { if name==n { return (b,4,0); } }
    let map16: [(&str,&str);16]=[("ax","rax"),("cx","rcx"),("dx","rdx"),("bx","rbx"),("sp","rsp"),("bp","rbp"),("si","rsi"),("di","rdi"),
        ("r8w","r8"),("r9w","r9"),("r10w","r10"),("r11w","r11"),("r12w","r12"),("r13w","r13"),("r14w","r14"),("r15w","r15")];
    for (n,b) in map16 { if name==n { return (b,2,0); } }
    // 8-bit low
    let map8: [(&str,&str);16]=[("al","rax"),("cl","rcx"),("dl","rdx"),("bl","rbx"),("spl","rsp"),("bpl","rbp"),("sil","rsi"),("dil","rdi"),
        ("r8b","r8"),("r9b","r9"),("r10b","r10"),("r11b","r11"),("r12b","r12"),("r13b","r13"),("r14b","r14"),("r15b","r15")];
    for (n,b) in map8 { if name==n { return (b,1,0); } }
    // 8-bit high
    for (n,b) in [("ah","rax"),("ch","rcx"),("dh","rdx"),("bh","rbx")] { if name==n { return (b,1,1); } }
    ("rax", 8, 0)
}

pub enum Stop { Halt(String), SysExit(i32) }

pub struct Machine {
    code: BTreeMap<u64, Vec<(char, Vec<String>)>>,
    addrs: Vec<u64>,
    next_of: BTreeMap<u64, u64>,
    pub entry: u64,
    reg: BTreeMap<String, u128>,
    mem: BTreeMap<u64, u8>,
    flags: (u128, u128, u8),
    kind: String,
    pub steps: u64,
    pub bits: u8,
}

impl Machine {
    pub fn new(module: &str) -> Machine {
        let mut m = Machine {
            code: BTreeMap::new(), addrs: Vec::new(), next_of: BTreeMap::new(), entry: 0,
            reg: BTreeMap::new(), mem: BTreeMap::new(), flags: (0,0,1), kind: "cmp".into(), steps: 0, bits: 64,
        };
        for r in ["rax","rcx","rdx","rbx","rsp","rbp","rsi","rdi","r8","r9","r10","r11","r12","r13","r14","r15","rip"] {
            m.reg.insert(r.into(), 0);
        }
        for k in 0..16 { m.reg.insert(format!("xmm{}", k), 0); }
        m.reg.insert("rsp".into(), 0x7FFF_0000);
        m.parse(module);
        m
    }

    fn parse(&mut self, text: &str) {
        let mut addr: Option<u64> = None;
        for line in text.lines() {
            if let Some(rest) = line.strip_prefix(';') {
                let t = rest.trim();
                if let Some(e) = t.strip_prefix("entry ") {
                    if let Ok(v) = u64::from_str_radix(e.trim().trim_start_matches("0x"), 16) { self.entry = v; }
                } else if let Some(b) = t.strip_prefix("bits ") {
                    if let Ok(v) = b.trim().parse::<u8>() { self.bits = v; }
                }
            } else if let Some(rest) = line.strip_prefix('=') {
                let mut it = rest.splitn(2, '\t');
                if let (Some(at), Some(blob)) = (it.next(), it.next()) {
                    if let Ok(a) = u64::from_str_radix(at.trim().trim_start_matches("0x"), 16) {
                        let bytes = hexbytes(blob);
                        for (k, b) in bytes.iter().enumerate() { if *b != 0 { self.mem.insert(a + k as u64, *b); } }
                    }
                }
            } else if let Some(rest) = line.strip_prefix('@') {
                if let Ok(a) = u64::from_str_radix(rest.trim().trim_start_matches("0x"), 16) {
                    addr = Some(a); self.code.entry(a).or_default();
                }
            } else if !line.is_empty() {
                if let Some(a) = addr {
                    let mut parts = line.split('\t');
                    if let Some(g) = parts.next() {
                        let glyph = g.chars().next().unwrap_or('?');
                        let fields: Vec<String> = parts.map(|s| s.to_string()).collect();
                        self.code.get_mut(&a).unwrap().push((glyph, fields));
                    }
                }
            }
        }
        self.addrs = self.code.keys().copied().collect();
        for w in self.addrs.windows(2) { self.next_of.insert(w[0], w[1]); }
    }

    // ── slots ──
    fn get_reg(&self, name: &str) -> u128 {
        let (base, size, off) = reg_info(name);
        (self.reg[base] >> (off as u32 * 8)) & mask(size)
    }
    fn set_reg(&mut self, name: &str, val: u128) {
        let (base, size, off) = reg_info(name);
        let cur = self.reg.get(base).copied().unwrap_or(0);
        let nv = match size {
            8 => val & mask(8),
            4 => val & mask(4),           // 32-bit writes zero the top
            16 => val & mask(16),
            _ => { let m = mask(size) << (off as u32 * 8); (cur & !m) | ((val << (off as u32 * 8)) & m) }
        };
        self.reg.insert(base.to_string(), nv);
    }
    fn load(&self, addr: u64, size: u8) -> u128 {
        let mut v = 0u128;
        for k in 0..size as u64 { v |= (*self.mem.get(&(addr + k)).unwrap_or(&0) as u128) << (8 * k); }
        v
    }
    fn store(&mut self, addr: u64, val: u128, size: u8) {
        let v = val & mask(size);
        for k in 0..size as u64 { self.mem.insert(addr + k, ((v >> (8*k)) & 0xFF) as u8); }
    }
    fn ea(&self, field: &str) -> (u64, u8) {
        // m:base:index:scale:disp:size
        let parts: Vec<&str> = field.split(':').collect();
        let base = parts[1]; let index = parts[2]; let scale = parts[3];
        let disp = parts[4]; let size: u8 = parts[5].parse().unwrap_or(8);
        let mut a: i128 = if !base.is_empty() { self.get_reg(base) as i128 } else { 0 };
        if !index.is_empty() { a += self.get_reg(index) as i128 * scale.parse::<i128>().unwrap_or(1); }
        a += parse_imm(disp);
        ((a as u128 & mask(8)) as u64, size)
    }
    fn read(&self, field: &str, size_hint: u8) -> (u128, u8) {
        match field.as_bytes()[0] {
            b'r' => { let n = &field[2..]; (self.get_reg(n), reg_info(n).1) }
            b'i' => ((parse_imm(&field[2..]) as u128) & mask(8), size_hint),
            _ => { let (a, s) = self.ea(field); (self.load(a, s), s) }
        }
    }
    fn write(&mut self, field: &str, val: u128) {
        if field.as_bytes()[0] == b'r' { self.set_reg(&field[2..], val); }
        else { let (a, s) = self.ea(field); self.store(a, val, s); }
    }
    fn width(&self, field: &str) -> u8 {
        match field.as_bytes()[0] {
            b'r' => reg_info(&field[2..]).1,
            b'm' => field.rsplit(':').next().and_then(|s| s.parse().ok()).unwrap_or(8),
            _ => 8,
        }
    }

    // ── truth ──
    fn cc(&self, name: &str) -> bool {
        let (a, b, size) = self.flags;
        let (zf, sf, cf, of);
        if self.kind == "test" {
            let r = (a & b) & mask(size);
            zf = r == 0; sf = sign(r, size) < 0; cf = false; of = false;
        } else {
            let r = a.wrapping_sub(b) & mask(size);
            zf = r == 0; sf = sign(r, size) < 0;
            cf = (a & mask(size)) < (b & mask(size));
            of = (sign(a, size) - sign(b, size)) != sign(r, size);
        }
        match name {
            "e"|"z" => zf, "ne"|"nz" => !zf,
            "s" => sf, "ns" => !sf,
            "b"|"nae"|"c" => cf, "ae"|"nb"|"nc" => !cf,
            "be"|"na" => cf||zf, "a"|"nbe" => !(cf||zf),
            "l"|"nge" => sf!=of, "ge"|"nl" => sf==of,
            "le"|"ng" => zf||(sf!=of), "g"|"nle" => !zf&&sf==of,
            "o" => of, "no" => !of, "p" => false, "np" => true,
            _ => false,
        }
    }
    fn set_flags(&mut self, a: u128, b: u128, size: u8, kind: &str) { self.flags = (a, b, size); self.kind = kind.to_string(); }

    fn slot(&self) -> u8 { self.bits / 8 }              // stack slot width, 8 or 4
    fn push_val(&mut self, v: u128) { let s = self.slot(); self.set_reg("rsp", self.get_reg("rsp").wrapping_sub(s as u128)); let sp = self.get_reg("rsp") as u64; self.store(sp, v, s); }
    fn pop_val(&mut self) -> u128 { let s = self.slot(); let sp = self.get_reg("rsp") as u64; let v = self.load(sp, s); self.set_reg("rsp", self.get_reg("rsp").wrapping_add(s as u128)); v }

    fn do_syscall(&mut self) -> Result<(), Stop> {
        let num = sign(self.get_reg("rax"), 8);
        let a0 = self.get_reg("rdi"); let _a1 = self.get_reg("rsi"); let _a2 = self.get_reg("rdx");
        if num == 60 || num == 231 { return Err(Stop::SysExit((sign(a0,8) & 0xFF) as i32)); }
        // write and others: this is a function tester; anything but exit returns -ENOSYS
        self.set_reg("rax", (-38i128 as u128) & mask(8));
        Ok(())
    }

    // ── ALU ──
    fn alu(&mut self, op: &str, f: &[String]) {
        match op {
            "nop"|"endbr64"|"endbr32" => return,
            "lea" => { let (a,_) = self.ea(&f[1]); self.write(&f[0], a as u128); return; }
            "cdq"|"cltd" => { let v = if sign(self.get_reg("eax"),4) < 0 { mask(4) } else { 0 }; self.set_reg("edx", v); return; }
            "cqo" => { let v = if sign(self.get_reg("rax"),8) < 0 { mask(8) } else { 0 }; self.set_reg("rdx", v); return; }
            "cdqe"|"cltq" => { let v = (sign(self.get_reg("eax"),4) as u128) & mask(8); self.set_reg("rax", v); return; }
            "idiv"|"div" => {
                let size = self.width(&f[0]);
                let d = self.read(&f[0], size).0;
                let (lo_n, hi_n) = ([8u8,4,2,1].contains(&size), true); let _ = (lo_n, hi_n);
                let lo = self.get_reg(match size {8=>"rax",4=>"eax",2=>"ax",_=>"al"});
                let hi = self.get_reg(match size {8=>"rdx",4=>"edx",2=>"dx",_=>"ah"});
                let (q, r);
                if op == "idiv" {
                    let n = sign((hi << (size as u32 *8)) | lo, if size < 8 { size*2 } else { 8 });
                    let dd = sign(d, size);
                    let qq = (n.abs() / dd.abs()) * if (n<0)==(dd<0) {1} else {-1};
                    q = (qq as u128) & mask(size); r = ((n - qq*dd) as u128) & mask(size);
                } else {
                    let n = (hi << (size as u32 *8)) | lo;
                    q = (n / d) & mask(size); r = (n % d) & mask(size);
                }
                self.set_reg(match size {8=>"rax",4=>"eax",2=>"ax",_=>"al"}, q);
                self.set_reg(match size {8=>"rdx",4=>"edx",2=>"dx",_=>"ah"}, r);
                return;
            }
            _ => {}
        }
        if is_simd(op) { self.simd(op, f); return; }

        let size = self.width(&f[0]);
        let a = self.read(&f[0], size).0;
        if matches!(op, "not"|"neg"|"inc"|"dec") {
            let r = match op { "not" => !a, "neg" => (a as i128).wrapping_neg() as u128, "inc" => a.wrapping_add(1), _ => a.wrapping_sub(1) };
            self.write(&f[0], r & mask(size));
            if op != "not" { self.set_flags(r & mask(size), 0, size, "cmp"); }
            return;
        }
        // One-operand imul/mul: the full 2*size product lands in edx:eax
        // (rdx:rax at 64-bit). Dropping the high half silently breaks every
        // magic-number division the compiler emits.
        if (op == "imul" || op == "mul") && f.len() == 1 {
            let (lo_r, hi_r) = match size { 8 => ("rax","rdx"), 4 => ("eax","edx"), 2 => ("ax","dx"), _ => ("al","ah") };
            let prod: u128 = if op == "imul" {
                ((sign(self.get_reg(lo_r), size)).wrapping_mul(sign(a, size))) as u128
            } else {
                (self.get_reg(lo_r) & mask(size)).wrapping_mul(a & mask(size))
            };
            self.set_reg(lo_r, prod & mask(size));
            let hi = (prod >> (size as u32 * 8)) & mask(size);
            self.set_reg(hi_r, hi);
            self.set_flags(prod & mask(size), 0, size, "cmp");
            return;
        }
        if op == "imul" && f.len() == 3 {
            let x = self.read(&f[1], size).0; let y = self.read(&f[2], size).0;
            let r = (sign(x, size) * sign(y, size)) as u128;
            self.write(&f[0], r & mask(size));
            self.set_flags(r & mask(size), 0, size, "cmp");
            return;
        }
        let b = self.read(&f[f.len()-1], size).0;
        let r: u128 = match op {
            "add"|"adc" => a.wrapping_add(b),
            "sub"|"sbb" => a.wrapping_sub(b),
            "and" => a & b, "or" => a | b, "xor" => a ^ b,
            "imul" => (sign(a,size) * sign(b,size)) as u128,
            "shl"|"sal" => a.wrapping_shl((b & 63) as u32),
            "shr" => (a & mask(size)) >> (b & 63),
            "sar" => (sign(a,size) >> (b & 63)) as u128,
            _ => a,
        };
        self.write(&f[0], r & mask(size));
        self.set_flags(r & mask(size), 0, size, "cmp");
    }

    fn simd(&mut self, op: &str, f: &[String]) {
        let dst = f[0].clone();
        match op {
            "movdqa"|"movdqu"|"movaps"|"movups" => { let v = self.read(&f[1], 16).0; self.write(&dst, v & mask(16)); }
            "movd"|"movq" => {
                let w = if op == "movd" { 4 } else { 8 };
                let src_is_x = f[1].starts_with("r:xmm");
                let v = self.read(&f[1], w).0;
                if dst.starts_with("r:xmm") { self.write(&dst, v & mask(w)); }
                else { self.write(&dst, if src_is_x { v & mask(w) } else { v }); }
            }
            "psrldq" => { let a = self.read(&dst,16).0; let n = parse_imm(&f[1][2..]) as u32; self.write(&dst, (a >> (n*8)) & mask(16)); }
            "psrlq"|"psllq" => {
                let a = self.read(&dst,16).0;
                let n = if f[1].as_bytes()[0]==b'i' { parse_imm(&f[1][2..]) as u32 } else { self.read(&f[1],8).0 as u32 };
                let lanes = [ (a & mask(8)), (a >> 64) & mask(8) ];
                let r: [u128;2] = if op=="psrlq" { [lanes[0]>>n, lanes[1]>>n] } else { [(lanes[0]<<n)&mask(8), (lanes[1]<<n)&mask(8)] };
                self.write(&dst, r[0] | (r[1] << 64));
            }
            "pshufd" => {
                let a = self.read(&f[1],16).0; let sel = parse_imm(&f[2][2..]) as u128;
                let l = [a&mask(4),(a>>32)&mask(4),(a>>64)&mask(4),(a>>96)&mask(4)];
                let mut o=0u128; for k in 0..4 { let s=((sel>>(2*k))&3) as usize; o |= l[s] << (32*k); }
                self.write(&dst, o);
            }
            "punpcklqdq" => { let a=self.read(&dst,16).0; let b=self.read(&f[1],16).0; self.write(&dst, (a&mask(8))|((b&mask(8))<<64)); }
            "punpckldq" => {
                let a=self.read(&dst,16).0; let b=self.read(&f[1],16).0;
                let x=[a&mask(4),(a>>32)&mask(4)]; let y=[b&mask(4),(b>>32)&mask(4)];
                self.write(&dst, x[0] | (y[0]<<32) | (x[1]<<64) | (y[1]<<96));
            }
            "pmuludq" => {
                let a=self.read(&dst,16).0; let b=self.read(&f[1],16).0;
                let x0=a&mask(4); let x2=(a>>64)&mask(4); let y0=b&mask(4); let y2=(b>>64)&mask(4);
                self.write(&dst, (x0*y0) | ((x2*y2)<<64));
            }
            "pxor"|"pand"|"por" => { let a=self.read(&dst,16).0; let b=self.read(&f[1],16).0;
                self.write(&dst, match op {"pxor"=>a^b,"pand"=>a&b,_=>a|b}); }
            _ => { // lane-wise padd/psub/pmull
                let a=self.read(&dst,16).0; let b=self.read(&f[1],16).0;
                let w: u8 = match op.chars().last().unwrap() {'b'=>1,'w'=>2,'d'=>4,'q'=>8,_=>4};
                let n = 16 / w; let mut o=0u128;
                for k in 0..n {
                    let sh = (k*w) as u32 * 8;
                    let p = (a >> sh) & mask(w); let q = (b >> sh) & mask(w);
                    let v = if op.starts_with("padd") { p.wrapping_add(q) } else if op.starts_with("psub") { p.wrapping_sub(q) } else { p.wrapping_mul(q) };
                    o |= (v & mask(w)) << sh;
                }
                self.write(&dst, o);
            }
        }
    }

    fn step(&mut self, addr: u64) -> Result<Option<u64>, Stop> {
        let next = self.next_of.get(&addr).copied().unwrap_or(0);
        self.reg.insert("rip".into(), next as u128);
        let insns = self.code.get(&addr).cloned().unwrap_or_default();
        for (glyph, f) in insns {
            match glyph {
                '∋' => continue,
                '⊣' => {
                    if f.get(0).map(|s| s.as_str()) == Some("leave") {
                        let s = self.slot();
                        let rbp = self.get_reg("rbp"); self.set_reg("rsp", rbp);
                        let v = self.load(self.get_reg("rsp") as u64, s); self.set_reg("rbp", v);
                        self.set_reg("rsp", self.get_reg("rsp").wrapping_add(s as u128)); continue;
                    }
                    let ret = self.pop_val();
                    // ret imm16: stdcall callee-cleanup of stack args.
                    if let Some(im) = f.get(1) { if im.as_bytes().get(0) == Some(&b'i') {
                        let n = parse_imm(&im[2..]) as u128;
                        self.set_reg("rsp", self.get_reg("rsp").wrapping_add(n));
                    }}
                    return Ok(Some(ret as u64));
                }
                '⊤' => { let size=self.width(&f[1]); let a=self.read(&f[1],size).0; let b=self.read(&f[2],size).0; self.set_flags(a,b,size,&f[0]); }
                '∈' => { if self.cc(&f[0]) { return Ok(Some(parse_imm(&f[1][2..]) as u64)); } }
                '≺' => { return Ok(Some(parse_imm(&f[1][2..]) as u64)); }
                '⊙' => {
                    if f.get(0).map(|s|s.as_str()) == Some("syscall") { self.do_syscall()?; continue; }
                    if f.get(0).map(|s|s.as_str()) == Some("external") { return Err(Stop::Halt(format!("external {}", f.get(1).cloned().unwrap_or_default()))); }
                    let tgt = self.read(&f[1], 8).0;
                    if f.get(0).map(|s|s.as_str()) == Some("call") { self.push_val(next as u128); }
                    return Ok(Some(tgt as u64));
                }
                '≻' => {
                    self.push_val(next as u128);
                    return Ok(Some(parse_imm(&f[1][2..]) as u64));
                }
                '⋈' | '◻' => {
                    let op = f[0].as_str();
                    match op {
                        "push" => { let v=self.read(&f[1],self.slot()).0; self.push_val(v); }
                        "pop" => { let v=self.pop_val(); self.write(&f[1], v); }
                        "leave" => { let s=self.slot(); let rbp=self.get_reg("rbp"); self.set_reg("rsp",rbp); let v=self.load(self.get_reg("rsp") as u64,s); self.set_reg("rbp",v); self.set_reg("rsp", self.get_reg("rsp").wrapping_add(s as u128)); }
                        "xchg" => { let x=self.read(&f[1],8).0; let y=self.read(&f[2],8).0; self.write(&f[1],y); self.write(&f[2],x); }
                        "mov"|"movabs" => { let w=self.width(&f[1]); let v=self.read(&f[2],w).0; self.write(&f[1], v & mask(w)); }
                        "movzx" => { let v=self.read(&f[2],8).0; self.write(&f[1], v & mask(self.width(&f[2]))); }
                        "movsx"|"movsxd" => { let sw=self.width(&f[2]); let v=self.read(&f[2],8).0; self.write(&f[1], (sign(v,sw) as u128) & mask(self.width(&f[1]))); }
                        _ => self.alu(op, &f[1..]),
                    }
                }
                '⊥' => {
                    if f.get(1).map(|s|s.as_str()) == Some("set") { let v = if self.cc(&f[0]) {1} else {0}; self.write(&f[2], v); }
                    else if self.cc(&f[0]) { let w=self.width(&f[2]); let v=self.read(&f[3],w).0; self.write(&f[2], v); }
                }
                '⊞' => self.alu(&f[0], &f[1..]),
                _ => {}
            }
        }
        Ok(self.next_of.get(&addr).copied())
    }

    /// Run one function to its ⊣. 64-bit takes integer args in registers (System
    /// V); 32-bit takes them on the stack (cdecl). Returns eax.
    pub fn call(&mut self, addr: u64, args: &[i64], limit: u64) -> Result<i64, Stop> {
        let sentinel: u64 = 0xDEAD_0000;
        if self.bits == 32 {
            // cdecl: args pushed right-to-left, then the return address on top.
            for v in args.iter().rev() { self.push_val((*v as u32) as u128); }
            self.push_val(sentinel as u128);
        } else {
            for (name, v) in ["rdi","rsi","rdx","rcx","r8","r9"].iter().zip(args) {
                self.set_reg(name, (*v as u128) & mask(8));
            }
            self.push_val(sentinel as u128);
        }
        let mut pc = addr; self.steps = 0;
        let mut trace: Vec<String> = Vec::new();
        while pc != sentinel {
            if !self.code.contains_key(&pc) {
                return Err(Stop::Halt(format!("no instruction at 0x{:x} after {} steps\n  previous 12:\n{}", pc, self.steps, trace.join("\n"))));
            }
            if trace.len() >= 12 { trace.remove(0); }
            let insn_txt = self.code.get(&pc).map(|v| v.iter().map(|(g,f)| format!("{} {}", g, f.join(" "))).collect::<Vec<_>>().join(" ; ")).unwrap_or_default();
            trace.push(format!("  {:04x}: {}", pc, insn_txt));
            match self.step(pc)? {
                Some(n) => pc = n,
                None => return Err(Stop::Halt(format!("ran off the end after {} steps\n  previous 12:\n{}", self.steps, trace.join("\n")))),
            }
            self.steps += 1;
            if self.steps > limit { return Err(Stop::Halt(format!("ran off the end after {} steps", self.steps))); }
        }
        Ok(sign(self.get_reg("eax"), 4) as i64)
    }
}

fn is_simd(op: &str) -> bool {
    matches!(op, "movdqa"|"movdqu"|"movaps"|"movups"|"movd"|"movq"|"pxor"|"pand"|"por"
        |"paddd"|"paddq"|"paddw"|"paddb"|"psubd"|"psubq"|"psubw"|"psubb"|"pmulld"|"pmuludq"
        |"psrlq"|"psllq"|"psrldq"|"pshufd"|"punpckldq"|"punpcklqdq")
}

fn hexbytes(s: &str) -> Vec<u8> {
    let mut out = Vec::new(); let mut hi: Option<u8> = None;
    for ch in s.chars() {
        let v = match ch { '0'..='9'=>ch as u8-b'0','a'..='f'=>ch as u8-b'a'+10,'A'..='F'=>ch as u8-b'A'+10,_=>continue };
        match hi { None=>hi=Some(v), Some(h)=>{ out.push((h<<4)|v); hi=None; } }
    }
    out
}

/// Parse an immediate spelled "0x1a" or "-0x1a".
fn parse_imm(s: &str) -> i128 {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix('-') {
        -(i128::from_str_radix(rest.trim_start_matches("0x"), 16).unwrap_or(0))
    } else {
        i128::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or(0)
    }
}
