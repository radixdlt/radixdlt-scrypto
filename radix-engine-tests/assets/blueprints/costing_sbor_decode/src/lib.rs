use scrypto::prelude::*;

#[blueprint]
mod s {
    struct Test {}

    impl Test {
        pub fn f() {
            loop {
                // Avoid loop being optimised away!
                std::hint::black_box(
                    manifest_decode::<ManifestValue>(include_bytes!("../../../large_package.rpd"))
                        .unwrap(),
                );
            }
        }
    }
}
