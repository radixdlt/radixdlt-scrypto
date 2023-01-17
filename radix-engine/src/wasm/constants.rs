pub const CONSUME_BUFFER_FUNCTION_ID: usize = 0;
pub const CONSUME_BUFFER_FUNCTION_NAME: &str = "consume_buffer";
pub const INVOKE_METHOD_FUNCTION_ID: usize = 1;
pub const INVOKE_METHOD_FUNCTION_NAME: &str = "invoke_method";
pub const INVOKE_FUNCTION_ID: usize = 2;
pub const INVOKE_FUNCTION_NAME: &str = "invoke";
pub const CREATE_NODE_FUNCTION_ID: usize = 3;
pub const CREATE_NODE_FUNCTION_NAME: &str = "create_node";
pub const GET_VISIBLE_NODES_FUNCTION_ID: usize = 4;
pub const GET_VISIBLE_NODES_FUNCTION_NAME: &str = "get_visible_nodes";
pub const DROP_NODE_FUNCTION_ID: usize = 5;
pub const DROP_NODE_FUNCTION_NAME: &str = "drop_node";
pub const LOCK_SUBSTATE_FUNCTION_ID: usize = 6;
pub const LOCK_SUBSTATE_FUNCTION_NAME: &str = "lock_substate";
pub const READ_SUBSTATE_FUNCTION_ID: usize = 7;
pub const READ_SUBSTATE_FUNCTION_NAME: &str = "read_substate";
pub const WRITE_SUBSTATE_FUNCTION_ID: usize = 8;
pub const WRITE_SUBSTATE_FUNCTION_NAME: &str = "write_substate";
pub const UNLOCK_SUBSTATE_FUNCTION_ID: usize = 9;
pub const UNLOCK_SUBSTATE_FUNCTION_NAME: &str = "unlock_substate";
pub const GET_ACTOR_FUNCTION_ID: usize = 10;
pub const GET_ACTOR_FUNCTION_NAME: &str = "get_actor";
pub const CONSUME_COST_UNITS_FUNCTION_ID: usize = 11;
pub const CONSUME_COST_UNITS_FUNCTION_NAME: &str = "gas";

pub const MODULE_ENV_NAME: &str = "env";
pub const EXPORT_MEMORY: &str = "memory";

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
