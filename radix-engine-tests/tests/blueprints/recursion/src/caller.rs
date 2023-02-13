use scrypto::prelude::*;

blueprint! {
    struct Caller;

    impl Caller {
        pub fn recursive(n: u32) {
            if n > 1 {
                let _: () = Runtime::call_function(
                    Runtime::package_address(),
                    "Caller",
                    "recursive",
                    args!(n - 1),
                );
            }
        }

        pub fn recursive_with_memory(n: u32, m: usize) {
            let _v: Vec<u8> = Vec::with_capacity(m);
            if n > 1 {
                let _: () = Runtime::call_function(
                    Runtime::package_address(),
                    "Caller",
                    "recursive_with_memory",
                    args!(n - 1, m),
                );
            }
        }
    }
}
