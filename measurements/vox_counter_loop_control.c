#include <stdint.h>
int main(void) {
    volatile uint64_t state = 1;
    for (uint64_t i = 0; i < 5000000; ++i)
        state = (state << 1) ^ (state >> 3) ^ i;
    return 0;
}
