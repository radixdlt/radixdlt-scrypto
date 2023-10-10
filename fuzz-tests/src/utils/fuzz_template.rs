// Helper macro to reduce the boilerplate code needed to implement
// fuzz tests for our fuzzers: afl, simple-fuzzer.
// For more complicated tests this macro is noth enough and the tests
// must be coded manually (see src/bin/transaction.rs).
#[macro_export]
macro_rules! fuzz_template {
    (|$buf:ident: $dty: ty| $body:block) => {
        #[cfg(feature = "afl")]
        use afl::fuzz;

        #[cfg(feature = "simple-fuzzer")]
        use fuzz_tests::fuzz;

        // Fuzzer entry points
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
