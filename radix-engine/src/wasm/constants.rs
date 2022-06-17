pub const RADIX_ENGINE_FUNCTION_INDEX: usize = 0;
pub const RADIX_ENGINE_FUNCTION_NAME: &str = "radix_engine";
pub const CONSUME_COST_UNITS_FUNCTION_INDEX: usize = 1;
pub const CONSUME_COST_UNITS_FUNCTION_NAME: &str = "gas";

pub const MODULE_ENV_NAME: &str = "env";

pub const EXPORT_MEMORY: &str = "memory";
pub const EXPORT_SCRYPTO_ALLOC: &str = "scrypto_alloc";
pub const EXPORT_SCRYPTO_FREE: &str = "scrypto_free";

/// The maximum initial memory size: `64 Pages * 64 KiB per Page = 4 MiB`
pub const DEFAULT_MAX_INITIAL_MEMORY_SIZE_PAGES: u32 = 64;

/// The maximum initial table size
pub const DEFAULT_MAX_INITIAL_TABLE_SIZE: u32 = 1024;

/// The max number of labels of a table jump, excluding the default
pub const DEFAULT_MAX_NUMBER_OF_BR_TABLE_TARGETS: u32 = 256;

/// The max number of global variables
pub const DEFAULT_MAX_NUMBER_OF_GLOBALS: u32 = 512;

/// The max number of functions
pub const DEFAULT_MAX_NUMBER_OF_FUNCTIONS: u32 = 64 * 1024;
