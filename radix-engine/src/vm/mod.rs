mod native_vm;
mod scrypto_vm;
mod vm;

/// Wasm validation, instrumentation and execution.
pub mod wasm;
/// Wasm runtime implementation.
pub mod wasm_runtime;

pub use native_vm::*;
pub use scrypto_vm::*;
pub use vm::*;
