/// Provides a transaction fuzzing.
pub mod transaction;
/// Utility functions.
pub mod utils;

// Let fuzz_loop() be visible in crate's root
// to let it be called by fuzz! macro
#[cfg(feature = "simple-fuzzer")]
pub use crate::utils::simple_fuzzer::fuzz_loop;
