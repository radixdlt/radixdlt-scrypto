use scrypto::prelude::*;
use sha256::digest;

#[blueprint]
mod s {
    struct Test {}

    impl Test {
        pub fn f() {
            loop {
                // Avoid loop being optimised away!
                std::hint::black_box(digest("hello"));
            }
        }
    }
}
