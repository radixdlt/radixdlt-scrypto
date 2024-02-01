use scrypto::prelude::*;

#[blueprint]
mod caller {
    struct Caller;

    impl Caller {
        pub fn recursive(n: u32) {
            if n > 1 {
                Blueprint::<Caller>::recursive(n - 1);
            }
        }
    }
}
