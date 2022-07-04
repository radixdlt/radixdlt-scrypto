use scrypto::prelude::*;

blueprint! {
    struct Caller;

    impl Caller {
        pub fn call(n: u32) {
            if n > 1 {
                let _: () = Runtime::call_function(
                    Runtime::package_address(),
                    "Caller",
                    "call",
                    args!(n - 1),
                );
            }
        }
    }
}
