mod constants;
mod errors;
mod prepare;
mod traits;
mod wasm_fee_table;
mod wasm_instrumenter;
mod wasm_validator;
#[cfg(feature = "wasmer")]
mod wasmer;
mod wasmi;

#[cfg(feature = "wasmer")]
pub use self::wasmer::*;
pub use self::wasmi::*;
pub use constants::*;
pub use errors::*;
pub use prepare::*;
pub use traits::*;
pub use wasm_fee_table::*;
pub use wasm_instrumenter::*;
pub use wasm_validator::*;

#[cfg(feature = "wasmer")]
pub type DefaultWasmEngine = WasmerEngine;

#[cfg(not(feature = "wasmer"))]
pub type DefaultWasmEngine = WasmiEngine;
