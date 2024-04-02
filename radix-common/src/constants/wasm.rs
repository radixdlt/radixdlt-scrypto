/// The maximum memory size (per call frame): 64 * 64KiB = 4MiB
#[cfg(not(feature = "coverage"))]
pub const MAX_MEMORY_SIZE_IN_PAGES: u32 = 64;
#[cfg(feature = "coverage")]
pub const MAX_MEMORY_SIZE_IN_PAGES: u32 = 512;

/// The maximum initial table size
pub const MAX_INITIAL_TABLE_SIZE: u32 = 1024;

/// The max number of labels of a table jump, excluding the default
pub const MAX_NUMBER_OF_BR_TABLE_TARGETS: u32 = 256;

/// The max number of global variables
pub const MAX_NUMBER_OF_GLOBALS: u32 = 512;

/// The max number of functions
pub const MAX_NUMBER_OF_FUNCTIONS: u32 = 8 * 1024;

/// The max number of function parameters
pub const MAX_NUMBER_OF_FUNCTION_PARAMS: u32 = 32;

/// The max number of function local variables
pub const MAX_NUMBER_OF_FUNCTION_LOCALS: u32 = 256;

/// The number of entries in the engine cache
pub const WASM_ENGINE_CACHE_SIZE: usize = 1000;

pub const WASM_EXECUTION_COST_UNITS_BUFFER: u32 = 1_000_000;
