use sbor::rust::fmt;
use sbor::rust::string::String;
use sbor::DecodeError;
use wasmi::HostError;

// TODO: this is the only place in the `wasm` module that introduces circular dependencies. Better solution?
use crate::engine::RuntimeError;
use crate::fee::CostUnitCounterError;

/// Represents an error when validating a WASM file.
#[derive(Debug, PartialEq, Clone)]
pub enum PrepareError {
    /// Failed to deserialize.
    /// See https://webassembly.github.io/spec/core/syntax/index.html
    DeserializationError,
    /// Failed to validate
    /// See https://webassembly.github.io/spec/core/valid/index.html
    ValidationError,
    /// Failed to serialize.
    SerializationError,
    /// The wasm module contains a start function.
    StartFunctionNotAllowed,
    /// The wasm module uses float points.
    FloatingPointNotAllowed,
    /// Invalid imports
    InvalidImports,
    /// The wasm module has no memory definition.
    NoMemory,
    /// The wasm module has too many memory definitions.
    TooManyMemories,
    /// Invalid memory definition.
    NonStandardMemory,
    /// The wasm module does not have the `memory` export.
    NoMemoryExport,
    /// The wasm module does not have the `scrypto_alloc` export.
    NoScryptoAllocExport,
    /// The wasm module does not have the `scrypto_free` export.
    NoScryptoFreeExport,
    /// Failed to inject instruction metering
    RejectedByInstructionMetering,
    /// Failed to inject stack metering
    RejectedByStackMetering,
}

/// Represents an error when invoking an export of a Scrypto module.
#[derive(Debug, PartialEq, Clone)]
pub enum InvokeError {
    MemoryAllocError,

    MemoryAccessError,

    InvalidScryptoValue(DecodeError),

    WasmError(String),

    RuntimeError(RuntimeError),

    FunctionNotFound,

    InvalidCallData,

    MissingReturnData,

    InvalidReturnData,

    MeteringError(CostUnitCounterError),
}

impl fmt::Display for InvokeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for InvokeError {}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for InvokeError {}
