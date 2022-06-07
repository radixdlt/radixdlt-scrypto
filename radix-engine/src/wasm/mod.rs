mod constants;
mod errors;
mod prepare;
mod traits;
#[cfg(not(feature = "alloc"))]
mod wasmer;
mod wasmi;

#[cfg(not(feature = "alloc"))]
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
