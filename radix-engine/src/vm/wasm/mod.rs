mod constants;
mod cost_rules;
mod errors;
mod prepare;
mod traits;
mod wasm_instrumenter;
mod wasm_instrumenter_config;
mod wasm_validator;
#[cfg(feature = "wasmer")]
mod wasmer;
mod wasmi;
mod weights;

#[cfg(feature = "wasmer")]
pub use self::wasmer::*;
pub use self::wasmi::*;
pub use constants::*;
pub use cost_rules::*;
pub use errors::*;
pub use prepare::*;
pub use traits::*;
pub use wasm_instrumenter::*;
pub use wasm_instrumenter_config::*;
pub use wasm_validator::*;
pub use weights::*;

#[cfg(feature = "wasmer")]
pub type DefaultWasmEngine = WasmerEngine;
#[cfg(feature = "wasmer")]
pub type DefaultWasmInstance = WasmerInstance;

#[cfg(not(feature = "wasmer"))]
pub type DefaultWasmEngine = WasmiEngine;
#[cfg(not(feature = "wasmer"))]
pub type DefaultWasmInstance = WasmiInstance;

// FIXME: change to code hash
pub type CodeKey = radix_engine_interface::types::PackageAddress;
pub type MeteredCodeKey = (CodeKey, WasmInstrumenterConfig);
