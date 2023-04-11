mod constants;
mod cost_rules;
mod errors;
mod prepare;
mod traits;
mod wasm_instrumenter;
mod wasm_metering_config;
mod wasm_validator;
#[cfg(feature = "wasmer")]
mod wasmer;
mod wasmi;

#[cfg(feature = "wasmer")]
pub use self::wasmer::*;
pub use self::wasmi::*;
pub use constants::*;
pub use cost_rules::*;
pub use errors::*;
pub use prepare::*;
pub use traits::*;
pub use wasm_instrumenter::*;
pub use wasm_metering_config::*;
pub use wasm_validator::*;

#[cfg(feature = "wasmer")]
pub type DefaultWasmEngine = WasmerEngine;
#[cfg(feature = "wasmer")]
pub type DefaultWasmInstance = WasmerInstance;

#[cfg(not(feature = "wasmer"))]
pub type DefaultWasmEngine = WasmiEngine;
#[cfg(not(feature = "wasmer"))]
pub type DefaultWasmInstance = WasmiInstance;

// TODO: expand if package is upgradable.
pub type CodeKey = radix_engine_interface::types::PackageAddress;
pub type MeteredCodeKey = (CodeKey, WasmMeteringConfig);
