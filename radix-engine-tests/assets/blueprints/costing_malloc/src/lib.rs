use scrypto::prelude::*;

#[blueprint]
mod s {
    struct Test {}

    impl Test {
        pub fn f(i: usize, j: usize) {
            for _ in 0..i {
                // Avoid loop being optimized away!
                std::hint::black_box(core::mem::forget(Vec::<u8>::with_capacity(j)));
            }
        }
    }
}
