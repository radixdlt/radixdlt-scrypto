#[cfg(feature = "simple-fuzzer")]
// Import fuzzing macros (exported by simple_fuzzer module) to the crate's root
#[macro_use]
pub mod simple_fuzzer;

#[macro_use]
pub mod fuzz_template;
