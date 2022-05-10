pub const ENGINE_FUNCTION_INDEX: usize = 0;
pub const ENGINE_FUNCTION_NAME: &str = "radix_engine";
pub const TBD_FUNCTION_INDEX: usize = 1;
pub const TBD_FUNCTION_NAME: &str = "gas";

pub const MODULE_ENV_NAME: &str = "env";

pub const EXPORT_MEMORY: &str = "memory";
pub const EXPORT_SCRYPTO_ALLOC: &str = "scrypto_alloc";
pub const EXPORT_SCRYPTO_FREE: &str = "scrypto_free";

pub const MAX_STACK_DEPTH: u32 = 100;
pub const INSTRUCTION_COST: u32 = 1;
pub const MEMORY_GROW_COST: u32 = 100;
pub const EXPORT_BLUEPRINT_ABI_TBD_LIMIT: u32 = 100_000;
pub const CALL_FUNCTION_TBD_LIMIT: u32 = 100_000;
