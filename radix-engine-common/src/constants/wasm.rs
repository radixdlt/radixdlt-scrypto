/// The maximum memory size (per call frame): 64 * 64KiB = 4MiB
pub const MAX_MEMORY_SIZE_IN_PAGES: u32 = 64;

/// The maximum initial table size
pub const MAX_INITIAL_TABLE_SIZE: u32 = 1024;

/// The max number of labels of a table jump, excluding the default
pub const MAX_NUMBER_OF_BR_TABLE_TARGETS: u32 = 256;

/// The max number of global variables
pub const MAX_NUMBER_OF_GLOBALS: u32 = 512;

/// The max number of functions
pub const MAX_NUMBER_OF_FUNCTIONS: u32 = 64 * 1024;

/// The max number of function parameters
pub const MAX_NUMBER_OF_FUNCTION_PARAMS: u32 = 32;

/// The max number of function local variables
pub const MAX_NUMBER_OF_FUNCTION_LOCALS: u32 = 128;

/// The number of entries in the engine cache
pub const WASM_ENGINE_CACHE_SIZE: usize = 1000;

pub const URL_MAX_USERNAME_LEN: usize = 64;
pub const URL_MAX_PASSWORD_LEN: usize = 64;
pub const URL_MAX_DOMAIN_LEN: usize = 128;
pub const URL_MAX_PATH_LEN: usize = 128;
pub const URL_MAX_QUERY_LEN: usize = 512;
pub const URL_MAX_FRAGMENT_LEN: usize = 512;
