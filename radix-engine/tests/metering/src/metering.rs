use scrypto::prelude::*;

blueprint! {
    struct Metering {}

    impl Metering {
        pub fn loooop(n: u32) -> u32 {
            let mut sum: u32 = 0;
            for i in 0..n {
                sum = sum + i;
            }
            sum
        }

        pub fn fib(n: u32) -> u32 {
            match n {
                0 => 0,
                1 => 1,
                _ => Self::fib(n - 1) + Self::fib(n - 2),
            }
        }
    }
}
