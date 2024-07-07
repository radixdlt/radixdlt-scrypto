mod constants;
mod errors;
#[cfg(feature = "latest-wasmer")]
mod latest_wasmer;
mod prepare;
mod traits;
mod wasm_validator;
mod wasm_validator_config;
#[cfg(feature = "wasmer")]
mod wasmer;
mod wasmi;
mod weights;

#[cfg(feature = "latest-wasmer")]
pub use self::latest_wasmer::*;
#[cfg(feature = "wasmer")]
pub use self::wasmer::*;
#[cfg(feature = "wasmi")]
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

#[cfg(feature = "latest-wasmer")]
pub type DefaultWasmEngine = WasmerEngine;
#[cfg(feature = "latest-wasmer")]
pub type DefaultWasmInstance = WasmerInstance;

#[cfg(feature = "wasmi")]
pub type DefaultWasmEngine = WasmiEngine;
#[cfg(feature = "wasmi")]
pub type DefaultWasmInstance = WasmiInstance;
