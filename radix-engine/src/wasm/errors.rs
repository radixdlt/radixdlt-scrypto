use wasmi::HostError;

use crate::fee::FeeReserveError;
use crate::model::InvokeError;
use crate::types::*;

/// Represents an error when validating a WASM file.
#[derive(Debug, PartialEq, Eq, Clone, TypeId, Encode, Decode)]
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
    /// Invalid import section
    InvalidImport(InvalidImport),
    /// Invalid memory section
    InvalidMemory(InvalidMemory),
    /// Invalid table section
    InvalidTable(InvalidTable),
    /// Too many targets in the `br_table` instruction
    TooManyTargetsInBrTable,
    /// Too many functions
    TooManyFunctions,
    /// Too many globals
    TooManyGlobals,
    /// No export section
    NoExportSection,
    /// Missing export
    MissingExport { export_name: String },
    /// The wasm module does not have the `scrypto_alloc` export.
    NoScryptoAllocExport,
    /// The wasm module does not have the `scrypto_free` export.
    NoScryptoFreeExport,
    /// Failed to inject instruction metering
    RejectedByInstructionMetering,
    /// Failed to inject stack metering
    RejectedByStackMetering,
    /// Not instantiatable
    NotInstantiatable,
    /// Not compilable
    NotCompilable,
}

#[derive(Debug, PartialEq, Eq, Clone, TypeId, Encode, Decode)]
pub enum InvalidImport {
    /// The import is not allowed
    ImportNotAllowed,
}

#[derive(Debug, PartialEq, Eq, Clone, TypeId, Encode, Decode)]
pub enum InvalidMemory {
    /// The wasm module has no memory section.
    NoMemorySection,
    /// The memory section is empty.
    EmptyMemorySection,
    /// The memory section contains too many memory definitions.
    TooManyMemories,
    /// The initial memory size is too large.
    InitialMemorySizeLimitExceeded,
    /// The wasm module does not have the `memory` export.
    MemoryNotExported,
}

#[derive(Debug, PartialEq, Eq, Clone, TypeId, Encode, Decode)]
pub enum InvalidTable {
    /// More than one table defined, against WebAssembly MVP spec
    MoreThanOneTable,
    /// Initial table size too large
    InitialTableSizeLimitExceeded,
}

/// Represents an error when invoking an export of a Scrypto module.
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, TypeId)]
pub enum WasmError {
    MemoryAllocError,
    MemoryAccessError,
    InvalidScryptoValue(DecodeError),
    WasmError(String),
    FunctionNotFound,
    InvalidRadixEngineInput,
    MissingReturnData,
    InvalidReturnData,
    CostingError(FeeReserveError),
}

impl fmt::Display for WasmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for WasmError {}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for WasmError {}

impl fmt::Display for InvokeError<WasmError> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl HostError for InvokeError<WasmError> {}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for InvokeError<WasmError> {}
