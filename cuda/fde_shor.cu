// FDE Shor membrane primitive.
//
// One CUDA thread owns one resident transition.  The forward and reverse
// sidearms each perform the same arbitrary-width modular multiplication from
// the opened state.  The boundary flag is true only for equal sidearms whose
// fused value is strictly below N.

#include <stdint.h>

__device__ bool less_limbs(const uint32_t *a, const uint32_t *b, uint32_t L) {
    for (int i = (int)L - 1; i >= 0; --i) {
        if (a[i] != b[i]) return a[i] < b[i];
    }
    return false;
}

__device__ bool equal_limbs(const uint32_t *a, const uint32_t *b, uint32_t L) {
    for (uint32_t i = 0; i < L; ++i) if (a[i] != b[i]) return false;
    return true;
}

__device__ void copy_limbs(uint32_t *dst, const uint32_t *src, uint32_t L) {
    for (uint32_t i = 0; i < L; ++i) dst[i] = src[i];
}

// out = (a + b) mod n.  All inputs are already reduced and the carry is kept
// separately, so no machine-width widening assumption is made about N.
__device__ void add_mod(uint32_t *out, const uint32_t *a, const uint32_t *b,
                        const uint32_t *n, uint32_t L) {
    uint64_t carry = 0;
    for (uint32_t i = 0; i < L; ++i) {
        uint64_t s = (uint64_t)a[i] + b[i] + carry;
        out[i] = (uint32_t)s;
        carry = s >> 32;
    }
    if (carry || !less_limbs(out, n, L)) {
        uint64_t borrow = 0;
        for (uint32_t i = 0; i < L; ++i) {
            uint64_t sub = (uint64_t)n[i] + borrow;
            uint64_t value = out[i];
            out[i] = (uint32_t)(value - sub);
            borrow = value < sub;
        }
    }
}

// Deliberately structure the product as a bit-fold over the resident limb
// tape.  This is the same fold shape as the IMASM arithmetic membrane.
__device__ void mul_mod(uint32_t *out, const uint32_t *x, const uint32_t *y,
                        const uint32_t *n, uint32_t L, uint32_t *scratch) {
    uint32_t *acc = scratch;
    uint32_t *cur = scratch + L;
    uint32_t *tmp = scratch + 2 * L;
    for (uint32_t i = 0; i < L; ++i) { acc[i] = 0; cur[i] = x[i]; }
    for (uint32_t bit = 0; bit < L * 32; ++bit) {
        if ((y[bit / 32] >> (bit % 32)) & 1u) {
            add_mod(tmp, acc, cur, n, L);
            copy_limbs(acc, tmp, L);
        }
        add_mod(tmp, cur, cur, n, L);
        copy_limbs(cur, tmp, L);
    }
    copy_limbs(out, acc, L);
}

extern "C" __global__ void fde_modular_step(
    const uint32_t *opened, const uint32_t *base, const uint32_t *modulus,
    uint32_t *forward, uint32_t *reverse, uint8_t *closed, uint32_t L) {
    uint32_t lane = blockIdx.x * blockDim.x + threadIdx.x;
    const uint32_t *state = opened + lane * L;
    uint32_t *f = forward + lane * L;
    uint32_t *r = reverse + lane * L;
    uint32_t scratch[3 * 1024];
    if (L > 1024) { closed[lane] = 0; return; }
    mul_mod(f, state, base, modulus, L, scratch);
    mul_mod(r, state, base, modulus, L, scratch);
    closed[lane] = equal_limbs(f, r, L) && less_limbs(f, modulus, L);
}
