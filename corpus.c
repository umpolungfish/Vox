/* The thirteen functions verify.py decides against.
 *
 * Each one is here because it forces a particular glyph to carry real weight:
 * plain arithmetic and loops for the linear glyphs, recursion for > and ⊣,
 * a dense switch and an array of function pointers for ⊙, a stack array for
 * ◻, and a vectorisable reduction so that -O3 has something to turn into SSE.
 * gcc is free to transform all of it however it likes at every level; the
 * claim is that the twelve hold whatever comes out.
 */

int gcd(int a, int b) {
    while (b) {
        int t = a % b;
        a = b;
        b = t;
    }
    return a;
}

int fib(int n) {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

int sumsq(int n) {
    int s = 0;
    for (int i = 1; i <= n; i++) s += i * i;
    return s;
}

int clamp(int x, int lo, int hi) {
    if (x < lo) return lo;
    if (x > hi) return hi;
    return x;
}

int popcnt(int x) {
    unsigned u = (unsigned)x;
    int c = 0;
    while (u) {
        c += (int)(u & 1u);
        u >>= 1;
    }
    return c;
}

int collatz(int n) {
    int steps = 0;
    long v = n;
    while (v != 1) {
        v = (v & 1) ? 3 * v + 1 : v / 2;
        steps++;
    }
    return steps;
}

int fact(int n) {
    if (n <= 1) return 1;
    return n * fact(n - 1);
}

int ack(int m, int n) {
    if (m == 0) return n + 1;
    if (n == 0) return ack(m - 1, 1);
    return ack(m - 1, ack(m, n - 1));
}

/* vectorisable: a fixed-length reduction over two built arrays */
int viadd(int a, int b) {
    int xs[64], ys[64];
    for (int i = 0; i < 64; i++) {
        xs[i] = a + i;
        ys[i] = b - i;
    }
    int s = 0;
    for (int i = 0; i < 64; i++) s += xs[i] * ys[i];
    return s;
}

/* stack memory: a local array written then read back */
int arr(int k) {
    int buf[64];
    for (int i = 0; i < 64; i++) buf[i] = (i * i + 7 * i + 1) % 251;
    int idx = k & 63;
    return buf[idx] + buf[(idx * 3 + 5) & 63];
}

/* dense switch: gcc builds a jump table at -O1 and above */
int sw(int k, int v) {
    switch (k & 15) {
    case 0:  return v + 1;
    case 1:  return v - 1;
    case 2:  return v * 2;
    case 3:  return v / 3;
    case 4:  return v % 7;
    case 5:  return v ^ 0x5a;
    case 6:  return v & 0xff;
    case 7:  return v | 0x100;
    case 8:  return -v;
    case 9:  return v << 2;
    case 10: return v >> 3;
    case 11: return v + k;
    case 12: return v - k;
    case 13: return v * k;
    case 14: return (v > 0) ? v : -v;
    default: return 0;
    }
}

static int op_add(int a, int b) { return a + b; }
static int op_sub(int a, int b) { return a - b; }
static int op_mul(int a, int b) { return a * b; }
static int op_xor(int a, int b) { return a ^ b; }

/* indirect call through a function-pointer array: the ⊙ case */
int fptr(int k, int a, int b) {
    static int (*const ops[4])(int, int) = {op_add, op_sub, op_mul, op_xor};
    return ops[k & 3](a, b);
}

int sumdigits(int n) {
    int s = 0;
    while (n) {
        s += n % 10;
        n /= 10;
    }
    return s;
}
