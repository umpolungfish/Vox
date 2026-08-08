//! x86-64 operand decoder — hand-written, zero external crates.
//!
//! Enough of the ISA to recompile and *run* the twelve-glyph module: full
//! ModRM/SIB/REX/prefix decoding into structured operands (register, immediate,
//! memory base:index:scale:disp:size), covering the integer and SSE opcodes gcc
//! emits. Operands are in Intel order (dest first) and print in the module's own
//! normal form so the machine never parses assembly:
//!     r:rax   i:0x10   m:base:index:scale:disp:size

use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::format;

#[derive(Clone, Debug)]
pub enum Op {
    Reg(String),
    Imm(i64),
    Mem { base: String, index: String, scale: u8, disp: i64, size: u8 },
}

impl Op {
    pub fn field(&self) -> String {
        match self {
            Op::Reg(r) => format!("r:{}", r),
            Op::Imm(v) => if *v >= 0 { format!("i:{:#x}", v) } else { format!("i:-{:#x}", -v) },
            Op::Mem { base, index, scale, disp, size } => {
                let d = if *disp >= 0 { format!("{:#x}", disp) } else { format!("-{:#x}", -disp) };
                format!("m:{}:{}:{}:{}:{}", base, index, scale, d, size)
            }
        }
    }
    fn is_mem(&self) -> bool { matches!(self, Op::Mem { .. }) }
}

#[derive(Clone, Debug)]
pub struct Insn {
    pub addr: u64,
    pub len: usize,
    pub mnemonic: String,
    pub ops: Vec<Op>,
    pub target: Option<u64>,
    pub writes_mem: bool,
}

const R64: [&str; 16] = ["rax","rcx","rdx","rbx","rsp","rbp","rsi","rdi",
                         "r8","r9","r10","r11","r12","r13","r14","r15"];
const R32: [&str; 16] = ["eax","ecx","edx","ebx","esp","ebp","esi","edi",
                         "r8d","r9d","r10d","r11d","r12d","r13d","r14d","r15d"];
const R16: [&str; 16] = ["ax","cx","dx","bx","sp","bp","si","di",
                         "r8w","r9w","r10w","r11w","r12w","r13w","r14w","r15w"];
const R8:  [&str; 16] = ["al","cl","dl","bl","spl","bpl","sil","dil",
                         "r8b","r9b","r10b","r11b","r12b","r13b","r14b","r15b"];
const R8L: [&str; 8]  = ["al","cl","dl","bl","ah","ch","dh","bh"];
const CC:  [&str; 16] = ["o","no","b","ae","e","ne","be","a","s","ns","p","np","l","ge","le","g"];

fn reg(n: u8, size: u8, rex: bool) -> String {
    let i = (n & 15) as usize;
    match size {
        8 => R64[i].to_string(), 4 => R32[i].to_string(), 2 => R16[i].to_string(),
        1 => if rex { R8[i].to_string() } else { R8L[(n & 7) as usize].to_string() },
        16 => format!("xmm{}", n & 15),
        _ => R64[i].to_string(),
    }
}
fn rop(n: u8, size: u8, rex: bool) -> Op { Op::Reg(reg(n, size, rex)) }

struct Cur<'a> { b: &'a [u8], i: usize }
impl<'a> Cur<'a> {
    fn u8(&mut self) -> Option<u8> { let v = *self.b.get(self.i)?; self.i += 1; Some(v) }
    fn imm(&mut self, n: usize, signed: bool) -> Option<i64> {
        if self.i + n > self.b.len() { return None; }
        let mut v: u64 = 0;
        for k in 0..n { v |= (self.b[self.i + k] as u64) << (8 * k); }
        self.i += n;
        if signed {
            let bits = 8 * n as u32; let s = 1u64 << (bits - 1);
            Some(if v & s != 0 { (v as i128 - (1i128 << bits)) as i64 } else { v as i64 })
        } else { Some(v as i64) }
    }
}

struct Rex { w: bool, r: bool, x: bool, b: bool, p: bool }

fn modrm(c: &mut Cur, rex: &Rex, osz: u8, msz: u8) -> Option<(Op, u8)> {
    let m = c.u8()?;
    let md = m >> 6;
    let regf = ((m >> 3) & 7) | if rex.r { 8 } else { 0 };
    let rm3 = m & 7;
    if md == 3 {
        let rm = rm3 | if rex.b { 8 } else { 0 };
        return Some((rop(rm, osz, rex.p), regf));
    }
    let (mut base, mut index, mut scale, mut disp) = (String::new(), String::new(), 1u8, 0i64);
    if rm3 == 4 {
        let sib = c.u8()?;
        let ss = sib >> 6;
        let idx = ((sib >> 3) & 7) | if rex.x { 8 } else { 0 };
        let bas = (sib & 7) | if rex.b { 8 } else { 0 };
        scale = 1 << ss;
        if (sib >> 3) & 7 != 4 { index = R64[(idx & 15) as usize].to_string(); }
        if (sib & 7) == 5 && md == 0 { disp = c.imm(4, true)?; }
        else { base = R64[(bas & 15) as usize].to_string(); }
    } else if rm3 == 5 && md == 0 {
        base = "rip".to_string(); disp = c.imm(4, true)?;
    } else {
        let rm = rm3 | if rex.b { 8 } else { 0 };
        base = R64[(rm & 15) as usize].to_string();
    }
    match md { 1 => disp = c.imm(1, true)?, 2 => disp = c.imm(4, true)?, _ => {} }
    Some((Op::Mem { base, index, scale, disp, size: msz }, regf))
}

fn opsize(rex: &Rex, o66: bool) -> u8 { if rex.w { 8 } else if o66 { 2 } else { 4 } }

macro_rules! ins { ($addr:expr,$c:expr,$mn:expr,$ops:expr,$wm:expr,$t:expr) =>
    { Some(Insn{addr:$addr,len:$c.i,mnemonic:$mn.to_string(),ops:$ops,target:$t,writes_mem:$wm}) }; }

pub fn decode(b: &[u8], addr: u64) -> Option<Insn> {
    let mut c = Cur { b, i: 0 };
    let (mut o66, mut f3, mut f2) = (false, false, false);
    let mut rex = Rex { w:false, r:false, x:false, b:false, p:false };
    loop {
        let p = *b.get(c.i)?;
        match p {
            0x66 => { o66 = true; c.i += 1; }
            0xF3 => { f3 = true; c.i += 1; }
            0xF2 => { f2 = true; c.i += 1; }
            0x67 | 0xF0 | 0x2E | 0x36 | 0x3E | 0x26 | 0x64 | 0x65 => { c.i += 1; }
            0x40..=0x4F => { rex = Rex { w:p&8!=0, r:p&4!=0, x:p&2!=0, b:p&1!=0, p:true }; c.i += 1; break; }
            _ => break,
        }
    }
    let osz = opsize(&rex, o66);
    let op = c.u8()?;

    const ALU: [&str; 8] = ["add","or","adc","sbb","and","sub","xor","cmp"];
    if op < 0x40 && (op & 7) <= 5 && (op & 0xC0) == 0 {
        let mn = ALU[(op >> 3) as usize];
        match op & 7 {
            0 => { let (rm,r)=modrm(&mut c,&rex,1,1)?; let wm=rm.is_mem(); return ins!(addr,c,mn,vec![rm,rop(r,1,rex.p)],wm,None); }
            1 => { let (rm,r)=modrm(&mut c,&rex,osz,osz)?; let wm=rm.is_mem(); return ins!(addr,c,mn,vec![rm,rop(r,osz,rex.p)],wm,None); }
            2 => { let (rm,r)=modrm(&mut c,&rex,1,1)?; return ins!(addr,c,mn,vec![rop(r,1,rex.p),rm],false,None); }
            3 => { let (rm,r)=modrm(&mut c,&rex,osz,osz)?; return ins!(addr,c,mn,vec![rop(r,osz,rex.p),rm],false,None); }
            4 => { let im=c.imm(1,true)?; return ins!(addr,c,mn,vec![Op::Reg("al".into()),Op::Imm(im)],false,None); }
            5 => { let im=c.imm(if osz==2{2}else{4},true)?; return ins!(addr,c,mn,vec![rop(0,osz,rex.p),Op::Imm(im)],false,None); }
            _ => {}
        }
    }

    match op {
        0xA8 => { let im=c.imm(1,true)?; ins!(addr,c,"test",vec![Op::Reg("al".into()),Op::Imm(im)],false,None) }
        0xA9 => { let im=c.imm(if osz==2{2}else{4},true)?; ins!(addr,c,"test",vec![rop(0,osz,rex.p),Op::Imm(im)],false,None) }
        0xB0..=0xB7 => { let r=(op-0xB0)|if rex.b{8}else{0}; let im=c.imm(1,false)?; ins!(addr,c,"mov",vec![rop(r,1,rex.p),Op::Imm(im)],false,None) }
        0xB8..=0xBF => { let r=(op-0xB8)|if rex.b{8}else{0};
            if rex.w { let im=c.imm(8,false)?; ins!(addr,c,"movabs",vec![rop(r,8,rex.p),Op::Imm(im)],false,None) }
            else { let im=c.imm(if osz==2{2}else{4},false)?; ins!(addr,c,"mov",vec![rop(r,osz,rex.p),Op::Imm(im)],false,None) } }
        0x50..=0x57 => { let r=(op-0x50)|if rex.b{8}else{0}; ins!(addr,c,"push",vec![rop(r,8,rex.p)],false,None) }
        0x58..=0x5F => { let r=(op-0x58)|if rex.b{8}else{0}; ins!(addr,c,"pop",vec![rop(r,8,rex.p)],false,None) }
        0x63 => { let (rm,r)=modrm(&mut c,&rex,4,4)?; ins!(addr,c,"movsxd",vec![rop(r,8,rex.p),rm],false,None) }
        0x68 => { let im=c.imm(4,true)?; ins!(addr,c,"push",vec![Op::Imm(im)],false,None) }
        0x6A => { let im=c.imm(1,true)?; ins!(addr,c,"push",vec![Op::Imm(im)],false,None) }
        0x69 => { let (rm,r)=modrm(&mut c,&rex,osz,osz)?; let im=c.imm(if osz==2{2}else{4},true)?; ins!(addr,c,"imul",vec![rop(r,osz,rex.p),rm,Op::Imm(im)],false,None) }
        0x6B => { let (rm,r)=modrm(&mut c,&rex,osz,osz)?; let im=c.imm(1,true)?; ins!(addr,c,"imul",vec![rop(r,osz,rex.p),rm,Op::Imm(im)],false,None) }
        0x70..=0x7F => { let d=c.imm(1,true)?; let t=(addr as i64 + c.i as i64 + d) as u64; ins!(addr,c,format!("j{}",CC[(op-0x70) as usize]),vec![Op::Imm(t as i64)],false,Some(t)) }
        0x80 => { let (rm,g)=modrm(&mut c,&rex,1,1)?; let im=c.imm(1,true)?; let wm=rm.is_mem(); ins!(addr,c,ALU[(g&7) as usize],vec![rm,Op::Imm(im)],wm,None) }
        0x81 => { let (rm,g)=modrm(&mut c,&rex,osz,osz)?; let im=c.imm(if osz==2{2}else{4},true)?; let wm=rm.is_mem(); ins!(addr,c,ALU[(g&7) as usize],vec![rm,Op::Imm(im)],wm,None) }
        0x83 => { let (rm,g)=modrm(&mut c,&rex,osz,osz)?; let im=c.imm(1,true)?; let wm=rm.is_mem(); ins!(addr,c,ALU[(g&7) as usize],vec![rm,Op::Imm(im)],wm,None) }
        0x84 => { let (rm,r)=modrm(&mut c,&rex,1,1)?; ins!(addr,c,"test",vec![rm,rop(r,1,rex.p)],false,None) }
        0x85 => { let (rm,r)=modrm(&mut c,&rex,osz,osz)?; ins!(addr,c,"test",vec![rm,rop(r,osz,rex.p)],false,None) }
        0x86 => { let (rm,r)=modrm(&mut c,&rex,1,1)?; let wm=rm.is_mem(); ins!(addr,c,"xchg",vec![rm,rop(r,1,rex.p)],wm,None) }
        0x87 => { let (rm,r)=modrm(&mut c,&rex,osz,osz)?; let wm=rm.is_mem(); ins!(addr,c,"xchg",vec![rm,rop(r,osz,rex.p)],wm,None) }
        0x88 => { let (rm,r)=modrm(&mut c,&rex,1,1)?; let wm=rm.is_mem(); ins!(addr,c,"mov",vec![rm,rop(r,1,rex.p)],wm,None) }
        0x89 => { let (rm,r)=modrm(&mut c,&rex,osz,osz)?; let wm=rm.is_mem(); ins!(addr,c,"mov",vec![rm,rop(r,osz,rex.p)],wm,None) }
        0x8A => { let (rm,r)=modrm(&mut c,&rex,1,1)?; ins!(addr,c,"mov",vec![rop(r,1,rex.p),rm],false,None) }
        0x8B => { let (rm,r)=modrm(&mut c,&rex,osz,osz)?; ins!(addr,c,"mov",vec![rop(r,osz,rex.p),rm],false,None) }
        0x8D => { let (rm,r)=modrm(&mut c,&rex,osz,osz)?; ins!(addr,c,"lea",vec![rop(r,osz,rex.p),rm],false,None) }
        0x8F => { let (rm,_)=modrm(&mut c,&rex,8,8)?; let wm=rm.is_mem(); ins!(addr,c,"pop",vec![rm],wm,None) }
        0x90 => ins!(addr,c,"nop",vec![],false,None),
        0x98 => ins!(addr,c,if rex.w{"cdqe"}else{"cwde"},vec![],false,None),
        0x99 => ins!(addr,c,if rex.w{"cqo"}else{"cdq"},vec![],false,None),
        0xC0|0xC1 => { let sz=if op==0xC0{1}else{osz}; let (rm,g)=modrm(&mut c,&rex,sz,sz)?; let im=c.imm(1,false)?; let wm=rm.is_mem();
            ins!(addr,c,["rol","ror","rcl","rcr","shl","shr","sal","sar"][(g&7) as usize],vec![rm,Op::Imm(im)],wm,None) }
        0xD0|0xD1|0xD2|0xD3 => { let sz=if op==0xD0||op==0xD2{1}else{osz}; let (rm,g)=modrm(&mut c,&rex,sz,sz)?; let wm=rm.is_mem();
            let amt=if op==0xD0||op==0xD1{Op::Imm(1)}else{Op::Reg("cl".into())};
            ins!(addr,c,["rol","ror","rcl","rcr","shl","shr","sal","sar"][(g&7) as usize],vec![rm,amt],wm,None) }
        0xC6 => { let (rm,_)=modrm(&mut c,&rex,1,1)?; let im=c.imm(1,true)?; let wm=rm.is_mem(); ins!(addr,c,"mov",vec![rm,Op::Imm(im)],wm,None) }
        0xC7 => { let (rm,_)=modrm(&mut c,&rex,osz,osz)?; let im=c.imm(if osz==2{2}else{4},true)?; let wm=rm.is_mem(); ins!(addr,c,"mov",vec![rm,Op::Imm(im)],wm,None) }
        0xC3 => ins!(addr,c,"ret",vec![],false,None),
        0xC9 => ins!(addr,c,"leave",vec![],false,None),
        0xCC => ins!(addr,c,"int3",vec![],false,None),
        0xE8 => { let d=c.imm(4,true)?; let t=(addr as i64 + c.i as i64 + d) as u64; ins!(addr,c,"call",vec![Op::Imm(t as i64)],false,Some(t)) }
        0xE9 => { let d=c.imm(4,true)?; let t=(addr as i64 + c.i as i64 + d) as u64; ins!(addr,c,"jmp",vec![Op::Imm(t as i64)],false,Some(t)) }
        0xEB => { let d=c.imm(1,true)?; let t=(addr as i64 + c.i as i64 + d) as u64; ins!(addr,c,"jmp",vec![Op::Imm(t as i64)],false,Some(t)) }
        0xF6 => { let (rm,g)=modrm(&mut c,&rex,1,1)?; let wm=rm.is_mem();
            match g&7 { 0|1 => { let im=c.imm(1,true)?; ins!(addr,c,"test",vec![rm,Op::Imm(im)],false,None) }
                        2 => ins!(addr,c,"not",vec![rm],wm,None), 3 => ins!(addr,c,"neg",vec![rm],wm,None),
                        4 => ins!(addr,c,"mul",vec![rm],false,None), 5 => ins!(addr,c,"imul",vec![rm],false,None),
                        6 => ins!(addr,c,"div",vec![rm],false,None), _ => ins!(addr,c,"idiv",vec![rm],false,None) } }
        0xF7 => { let (rm,g)=modrm(&mut c,&rex,osz,osz)?; let wm=rm.is_mem();
            match g&7 { 0|1 => { let im=c.imm(if osz==2{2}else{4},true)?; ins!(addr,c,"test",vec![rm,Op::Imm(im)],false,None) }
                        2 => ins!(addr,c,"not",vec![rm],wm,None), 3 => ins!(addr,c,"neg",vec![rm],wm,None),
                        4 => ins!(addr,c,"mul",vec![rm],false,None), 5 => ins!(addr,c,"imul",vec![rm],false,None),
                        6 => ins!(addr,c,"div",vec![rm],false,None), _ => ins!(addr,c,"idiv",vec![rm],false,None) } }
        0xFE => { let (rm,g)=modrm(&mut c,&rex,1,1)?; let wm=rm.is_mem(); ins!(addr,c,if g&7==0{"inc"}else{"dec"},vec![rm],wm,None) }
        0xFF => { let (rm,g)=modrm(&mut c,&rex,osz,osz)?; let wm=rm.is_mem();
            match g&7 { 0 => ins!(addr,c,"inc",vec![rm],wm,None), 1 => ins!(addr,c,"dec",vec![rm],wm,None),
                        2 => ins!(addr,c,"call",vec![rm],false,None), 4 => ins!(addr,c,"jmp",vec![rm],false,None),
                        6 => ins!(addr,c,"push",vec![rm],false,None), _ => None } }
        0x0F => decode_0f(&mut c, addr, &rex, osz, f3),
        _ => None,
    }
}

fn decode_0f(c: &mut Cur, addr: u64, rex: &Rex, osz: u8, f3: bool) -> Option<Insn> {
    let op2 = c.u8()?;
    match op2 {
        0x05 => ins!(addr,c,"syscall",vec![],false,None),
        0x0B => ins!(addr,c,"ud2",vec![],false,None),
        0x1E => { let m=c.u8()?; ins!(addr,c,if m==0xFA{"endbr64"}else{"nop"},vec![],false,None) }
        0x1F => { let (_rm,_)=modrm(c,rex,osz,osz)?; ins!(addr,c,"nop",vec![],false,None) }
        0x40..=0x4F => { let (rm,r)=modrm(c,rex,osz,osz)?; ins!(addr,c,format!("cmov{}",CC[(op2-0x40) as usize]),vec![rop(r,osz,rex.p),rm],false,None) }
        0x80..=0x8F => { let d=c.imm(4,true)?; let t=(addr as i64 + c.i as i64 + d) as u64; ins!(addr,c,format!("j{}",CC[(op2-0x80) as usize]),vec![Op::Imm(t as i64)],false,Some(t)) }
        0x90..=0x9F => { let (rm,_)=modrm(c,rex,1,1)?; let wm=rm.is_mem(); ins!(addr,c,format!("set{}",CC[(op2-0x90) as usize]),vec![rm],wm,None) }
        0xAF => { let (rm,r)=modrm(c,rex,osz,osz)?; ins!(addr,c,"imul",vec![rop(r,osz,rex.p),rm],false,None) }
        0xB6 => { let (rm,r)=modrm(c,rex,1,1)?; ins!(addr,c,"movzx",vec![rop(r,osz,rex.p),rm],false,None) }
        0xB7 => { let (rm,r)=modrm(c,rex,2,2)?; ins!(addr,c,"movzx",vec![rop(r,osz,rex.p),rm],false,None) }
        0xBE => { let (rm,r)=modrm(c,rex,1,1)?; ins!(addr,c,"movsx",vec![rop(r,osz,rex.p),rm],false,None) }
        0xBF => { let (rm,r)=modrm(c,rex,2,2)?; ins!(addr,c,"movsx",vec![rop(r,osz,rex.p),rm],false,None) }
        0x28 => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"movaps",vec![rop(r,16,rex.p),rm],false,None) }
        0x29 => { let (rm,r)=modrm(c,rex,16,16)?; let wm=rm.is_mem(); ins!(addr,c,"movaps",vec![rm,rop(r,16,rex.p)],wm,None) }
        0x6E => { let sz=if rex.w{8}else{4}; let (rm,r)=modrm(c,rex,sz,sz)?; ins!(addr,c,if rex.w{"movq"}else{"movd"},vec![rop(r,16,rex.p),rm],false,None) }
        0x6F => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,if f3{"movdqu"}else{"movdqa"},vec![rop(r,16,rex.p),rm],false,None) }
        0x7E => { if f3 { let (rm,r)=modrm(c,rex,16,8)?; ins!(addr,c,"movq",vec![rop(r,16,rex.p),rm],false,None) }
                  else { let sz=if rex.w{8}else{4}; let (rm,r)=modrm(c,rex,sz,sz)?; let wm=rm.is_mem(); ins!(addr,c,if rex.w{"movq"}else{"movd"},vec![rm,rop(r,16,rex.p)],wm,None) } }
        0x7F => { let (rm,r)=modrm(c,rex,16,16)?; let wm=rm.is_mem(); ins!(addr,c,if f3{"movdqu"}else{"movdqa"},vec![rm,rop(r,16,rex.p)],wm,None) }
        0xD4 => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"paddq",vec![rop(r,16,rex.p),rm],false,None) }
        0xFE => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"paddd",vec![rop(r,16,rex.p),rm],false,None) }
        0xFD => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"paddw",vec![rop(r,16,rex.p),rm],false,None) }
        0xFC => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"paddb",vec![rop(r,16,rex.p),rm],false,None) }
        0xFA => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"psubd",vec![rop(r,16,rex.p),rm],false,None) }
        0xFB => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"psubq",vec![rop(r,16,rex.p),rm],false,None) }
        0xEF => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"pxor",vec![rop(r,16,rex.p),rm],false,None) }
        0xDB => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"pand",vec![rop(r,16,rex.p),rm],false,None) }
        0xEB => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"por",vec![rop(r,16,rex.p),rm],false,None) }
        0xF4 => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"pmuludq",vec![rop(r,16,rex.p),rm],false,None) }
        0x38 => { let op3=c.u8()?; let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,if op3==0x40{"pmulld"}else{"pshufb"},vec![rop(r,16,rex.p),rm],false,None) }
        0x70 => { let (rm,r)=modrm(c,rex,16,16)?; let im=c.imm(1,false)?; ins!(addr,c,"pshufd",vec![rop(r,16,rex.p),rm,Op::Imm(im)],false,None) }
        0x73 => { let (rm,g)=modrm(c,rex,16,16)?; let im=c.imm(1,false)?;
                  ins!(addr,c,match g&7 {2=>"psrlq",3=>"psrldq",6=>"psllq",7=>"pslldq",_=>"psrlq"},vec![rm,Op::Imm(im)],false,None) }
        0x72 => { let (rm,g)=modrm(c,rex,16,16)?; let im=c.imm(1,false)?;
                  ins!(addr,c,match g&7 {2=>"psrld",6=>"pslld",4=>"psrad",_=>"psrld"},vec![rm,Op::Imm(im)],false,None) }
        0xD3 => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"psrlq",vec![rop(r,16,rex.p),rm],false,None) }
        0xF3 => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"psllq",vec![rop(r,16,rex.p),rm],false,None) }
        0x62 => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"punpckldq",vec![rop(r,16,rex.p),rm],false,None) }
        0x6C => { let (rm,r)=modrm(c,rex,16,16)?; ins!(addr,c,"punpcklqdq",vec![rop(r,16,rex.p),rm],false,None) }
        0xA2 => ins!(addr,c,"cpuid",vec![],false,None),
        0xC8..=0xCF => ins!(addr,c,"bswap",vec![],false,None),
        _ => None,
    }
}
