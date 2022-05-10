use scrypto::prelude::*;

blueprint! {
    struct Metering {}

    impl Metering {
        pub fn iterations(n: u32) -> u32 {
            let mut x = n;
            for _ in 0..n {
                x *= x;
            }
            x
        }
    }
}
