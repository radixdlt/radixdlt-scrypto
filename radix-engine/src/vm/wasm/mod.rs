mod constants;
mod errors;
mod prepare;
mod traits;
mod wasm_validator;
mod wasm_validator_config;
#[cfg(feature = "wasmer")]
mod wasmer;
mod wasmi;
mod weights;

#[cfg(feature = "wasmer")]
pub use self::wasmer::*;
pub use self::wasmi::*;
pub use constants::*;
pub use errors::*;
pub use prepare::*;
pub use traits::*;
pub use wasm_validator::*;
pub use wasm_validator_config::*;
pub use weights::*;

#[cfg(feature = "wasmer")]
pub type DefaultWasmEngine = WasmerEngine;
#[cfg(feature = "wasmer")]
pub type DefaultWasmInstance = WasmerInstance;

#[cfg(not(feature = "wasmer"))]
pub type DefaultWasmEngine = WasmiEngine;
#[cfg(not(feature = "wasmer"))]
pub type DefaultWasmInstance = WasmiInstance;
