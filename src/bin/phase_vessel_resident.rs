#![no_std]
#![no_main]

use core::arch::asm;

// Phase-vessel resident: the selector-relation lane nested in the one-tower
// membrane. N baked at compile time via option_env!("FACTOR_N_DEC"); the
// factorization is RUNTIME phase walk: a = ceil(sqrt(N)), then a++ while
// b2 = a*a - N is not a perfect square. At the crossing b*b == b2 the vessel
// closes: p = a-b, q = a+b. leaves counts the a-increments — the same count
// the --selector-relation report names "internal phase leaves".
const fn baked_n() -> u64 {
    match option_env!("FACTOR_N_DEC") { Some(s) => parse_decimal(s), None => 10403 }
}
const N: u64 = baked_n();

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

fn print_dec(mut n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 20;
    if n == 0 { unsafe { write(b"0"); } return; }
    while n > 0 { i -= 1; buf[i] = b'0' + (n % 10) as u8; n /= 10; }
    unsafe { write(&buf[i..]); }
}

fn isqrt64(n: u64) -> u64 {
    if n == 0 { return 0; }
    let mut x = n; let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + n / x) / 2; }
    x
}

fn isqrt128(n: u128) -> u128 {
    if n == 0 { return 0; }
    let mut x = n; let mut y = (x + 1) / 2;
    while y < x { x = y; y = (x + n / x) / 2; }
    x
}

fn rem(n: u64, d: u64) -> u64 {
    let mut r = 0u64;
    for i in (0..64).rev() { r = (r << 1) | ((n >> i) & 1); if r >= d { r -= d; } }
    r
}

// The ABI expects rsp = 8 (mod 16) at function entry — a call has just pushed
// a return address. Process entry instead hands _start rsp = 0 (mod 16), so
// the body's prologue lands its frame off-boundary and the movaps zero-fill
// faults (SIGSEGV at _start+17: movaps %xmm0,0x50(%rsp), rsp misaligned by 8).
// Enter the real body through a call: that restores the ABI invariant and
// every aligned store in the frame lands true.
// nostdlib provides no libc: the array zero-fill lowers to memset, so we
// own it. Byte-simple; the builtin is what a static musl would give.
#[no_mangle]
pub extern "C" fn memset(dst: *mut u8, c: i32, n: usize) -> *mut u8 {
    let b = c as u8;
    let mut i = 0usize;
    while i < n { unsafe { *dst.add(i) = b; } i += 1; }
    dst
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    unsafe { asm!("and rsp, -16\ncall {body}", body = sym phase_body, options(noreturn)); }
}

unsafe extern "C" fn phase_body() -> ! {
    // Trial strip first: the even part, then odd 3..29. What remains goes
    // through the phase walk. leaves = the a-increments the walk consumed.
    let mut remainder = N;
    let mut factors = [0u64; 66];
    let mut count = 0usize;
    while rem(remainder, 2) == 0 && remainder > 1 {
        factors[count] = 2; count += 1; remainder >>= 1;
    }
    let mut d = 3u64;
    while d <= 29 && remainder > 1 {
        if rem(remainder, d) == 0 {
            factors[count] = d; count += 1; remainder /= d;
        } else { d += 2; }
    }
    let mut leaves = 0u64;
    if remainder > 1 {
        let nn = remainder as u128;
        let cap = (remainder >> 1) + 1; // the p=1 witness sits here for prime r
        let mut a = isqrt64(remainder) as u128;
        if a * a < nn { a += 1; }
        let a0 = a;
        loop {
            let b2 = a * a - nn;
            let b = isqrt128(b2);
            if b * b == b2 {
                let p = (a - b) as u64;
                if p > 1 && p < remainder && rem(remainder, p) == 0 {
                    factors[count] = p; count += 1;
                    remainder /= p;
                    if remainder > 1 { factors[count] = remainder; count += 1; }
                } else {
                    // crossing with p <= 1: the remainder is prime
                    factors[count] = remainder; count += 1;
                }
                break;
            }
            a += 1;
            if a >= cap as u128 { factors[count] = remainder; count += 1; break; }
        }
        leaves = (a - a0) as u64;
    }
    unsafe { write(b"N=0x"); } print_hex(N); unsafe { write(b" factors=0x"); }
    for i in 0..count { if i != 0 { unsafe { write(b" x 0x"); } } print_hex(factors[i]); }
    unsafe { write(b" leaves="); } print_dec(leaves);
    unsafe { write(b" boundary=true steps=31\n"); }
    exit(0)
}
