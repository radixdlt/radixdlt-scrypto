mod constants;
mod errors;
mod prepare;
mod traits;
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

#[cfg(feature = "wasmer")]
pub type DefaultWasmEngine = WasmerEngine;

#[cfg(not(feature = "wasmer"))]
pub type DefaultWasmEngine = WasmiEngine;
