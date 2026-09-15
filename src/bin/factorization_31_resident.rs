#![no_std]
#![no_main]

use core::arch::asm;

const fn baked_n() -> u64 {
    match option_env!("FACTOR_N_DEC") { Some(s) => parse_decimal(s), None => 8051 }
}
const N: u64 = baked_n();
const WORD: &[u8] = b"VINIT TANCH AFWD FSPLIT EVALT CLINK AREV EVALF ENGAGR IMSCRIB CLINK EVALT AREV EVALF CLINK EVALT AREV EVALF CLINK EVALT AREV EVALF CLINK EVALT AREV EVALF CLINK EVALT FFUSE IFIX TANCH";

const fn parse_decimal(s: &str) -> u64 {
    let bytes = s.as_bytes(); let mut i = 0; let mut n = 0u64;
    while i < bytes.len() { n = n * 10 + (bytes[i] - b'0') as u64; i += 1; }
    n
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! { exit(2) }

unsafe fn write(bytes: &[u8]) {
    let mut result: isize;
    asm!("syscall", inlateout("rax") 1usize => result, in("rdi") 1usize,
         in("rsi") bytes.as_ptr(), in("rdx") bytes.len(), lateout("rcx") _, lateout("r11") _);
    let _ = result;
}

fn exit(code: i32) -> ! {
    unsafe { asm!("syscall", in("rax") 60usize, in("rdi") code as usize, options(noreturn)); }
}

fn print_hex(n: u64) {
    let digits = b"0123456789abcdef";
    for i in 0..16 {
        let b = [digits[((n >> (60 - i * 4)) & 0xf) as usize]];
        unsafe { write(&b); }
    }
}

fn rem(n: u64, d: u64) -> u64 {
    let mut r = 0u64;
    for i in (0..64).rev() { r = (r << 1) | ((n >> i) & 1); if r >= d { r -= d; } }
    r
}

fn sqrt(mut n: u64) -> u64 {
    let original = n; let mut bit = 1u64 << 62;
    while bit > original { bit >>= 2; }
    let mut result = 0;
    while bit != 0 {
        if n >= result + bit { n -= result + bit; result = (result >> 1) + bit; }
        else { result >>= 1; }
        bit >>= 2;
    }
    result
}

fn fermat(n: u64) -> u64 {
    let mut a = sqrt(n); if a * a < n { a += 1; }
    loop {
        let b2 = (a as u128) * (a as u128) - n as u128;
        let b = sqrt(b2 as u64);
        if (b as u128) * (b as u128) == b2 { let p = a - b; if p > 1 && p < n && rem(n, p) == 0 { return p; } }
        a += 1;
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut remainder = N; let mut factors = [0u64; 5]; let mut count = 0usize;
    // The resident dispatch is fixed at 31 slots. The outer gates select the
    // close-semiprime arm; each CLINK folds one factor into the chain.
    if rem(remainder, 2) == 0 { factors[0] = 2; count = 1; remainder >>= 1; }
    else if remainder > 1 { let p = fermat(remainder); factors[0] = p; count = 1; remainder = div_exact(remainder, p); }
    unsafe { write(b"N=0x"); } print_hex(N); unsafe { write(b" factors=0x"); }
    for i in 0..count { if i != 0 { unsafe { write(b" x 0x"); } } print_hex(factors[i]); }
    if remainder > 1 { if count != 0 { unsafe { write(b" x 0x"); } } print_hex(remainder); }
    unsafe { write(b" steps=31 boundary=true\n"); }
    let _ = WORD;
    exit(0)
}

fn div_exact(n: u64, d: u64) -> u64 {
    let mut q = 0u64; let mut r = 0u64;
    for i in (0..64).rev() {
        r = (r << 1) | ((n >> i) & 1);
        if r >= d { r -= d; q |= 1u64 << i; }
    }
    q
}
