int __attribute__ ((noinline)) internal_add(int x, int y) {
    return x + y;
}

int __attribute__ ((noinline)) internal_sub(int x, int y) {
    return x - y;
}

// This function that takes a function pointer and calls it.
// When compiled to WASM it should result with using `call_indirect` instruction.
// Binary encoding of the `call_indirect` instruction changes depending on the `reference-types`.
//
// - reference-types disabled
//    0x11                      - call_indirect binary opcode
//    0x80 0x80 0x80 0x80 0x00  - 32-bit zero LEB encoded (function table index)
//    0x00                      - fixed zero (reserved value)
// - reference-types enabled
//    0x11                      - call_indirect binary opcode
//    0x80 0x80 0x80 0x80 0x00  - 32-bit zero LEB encoded (function table index)
//    0x80 0x80 0x80 0x80 0x00  - 32-bit zero LEB encoded (filled later by linker with the final table index)
//
// More details:
//  https://blog.rust-lang.org/2024/09/24/webassembly-targets-change-in-default-target-features.html#enabling-reference-types-by-default
int __attribute__ ((noinline)) compute(int (*operation)(int, int), int a, int b) {
    return operation(a, b);
}

int add(int x, int y) {
    return compute(internal_add, x, y);
}

int sub(int x, int y) {
    return compute(internal_sub, x, y);
}

