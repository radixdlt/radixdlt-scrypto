// Helper macro to reduce the boilerplate code needed to implement
// fuzz tests for our fuzzers: afl, libfuzzer, simple-fuzzer.
// For more complicated tests this macro is noth enough and the tests
// must be coded manually (see src/bin/transaction.rs).
//
// NOTE!
// Following piece of code has to be put anyway in implemented fuzz target
// in the first line:
// #![cfg_attr(feature = "libfuzzer-sys", no_main)]
#[macro_export]
macro_rules! fuzz_template {
    (|$buf:ident: $dty: ty| $body:block) => {
        #[cfg(feature = "libfuzzer-sys")]
        use libfuzzer_sys::fuzz_target;

        #[cfg(feature = "afl")]
        use afl::fuzz;

        #[cfg(feature = "simple-fuzzer")]
        use fuzz_tests::fuzz;

        // Fuzzer entry points
        #[cfg(feature = "libfuzzer-sys")]
        libfuzzer_sys::fuzz_target!(|$buf: $dty| { $body });

        #[cfg(feature = "afl")]
        fn main() {
            afl::fuzz!(|$buf: $dty| { $body });
        }

        #[cfg(feature = "simple-fuzzer")]
        fn main() {
            fuzz_tests::fuzz!(|$buf: $dty| { $body });
        }
    };
}
