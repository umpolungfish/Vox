#include <stdint.h>
volatile uint64_t VOX_SIEVE_COUNTERS[12] = {32,8,9,0,0,0,0,0,0,123,456,789};
int main(void) {
    for (uint64_t i = 0; i < 50000000; ++i) {
        VOX_SIEVE_COUNTERS[3] = i;
        VOX_SIEVE_COUNTERS[4] = 2*i;
        VOX_SIEVE_COUNTERS[5] = i/2;
    }
    return 0;
}
