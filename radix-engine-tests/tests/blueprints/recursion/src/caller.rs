use scrypto::prelude::*;

#[blueprint]
mod caller {
    struct Caller;

    impl Caller {
        pub fn recursive(n: u32) {
            if n > 1 {
                let _: () = Runtime::call_function(
                    Runtime::package_address(),
                    "Caller",
                    "recursive",
                    scrypto_args!(n - 1),
                );
            }
        }

        pub fn recursive_with_memory(n: u32, m: usize) {
            if n > 1 {
                let _v: Vec<u8> = Vec::with_capacity(m);
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
