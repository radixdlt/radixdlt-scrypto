/// Scrypto blueprint ABI.
pub mod abi {
    pub use scrypto_abi::*;
}
/// Cryptography library.
pub mod crypto;
pub mod math;
pub mod misc;
// Export macros
mod macros;
pub use macros::*;

// This is to make derives work within this crate.
// See: https://users.rust-lang.org/t/how-can-i-use-my-derive-macro-from-the-crate-that-declares-the-trait/60502
extern crate self as utils;
