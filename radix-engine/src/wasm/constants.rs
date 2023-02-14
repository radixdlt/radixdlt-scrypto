pub const CONSUME_BUFFER_FUNCTION_ID: usize = 0x10;
pub const CONSUME_BUFFER_FUNCTION_NAME: &str = "consume_buffer";
pub const CONSUME_COST_UNITS_FUNCTION_ID: usize = 0x11;
pub const CONSUME_COST_UNITS_FUNCTION_NAME: &str = "gas";

pub const NEW_PACKAGE_FUNCTION_ID: usize = 0x20;
pub const NEW_PACKAGE_FUNCTION_NAME: &str = "new_package";
pub const CALL_FUNCTION_FUNCTION_ID: usize = 0x21;
pub const CALL_FUNCTION_FUNCTION_NAME: &str = "call_function";

pub const NEW_COMPONENT_FUNCTION_ID: usize = 0x30;
pub const NEW_COMPONENT_FUNCTION_NAME: &str = "new_component";
pub const NEW_KEY_VALUE_STORE_FUNCTION_ID: usize = 0x31;
pub const NEW_KEY_VALUE_STORE_FUNCTION_NAME: &str = "new_key_value_store";
pub const GLOBALIZE_COMPONENT_FUNCTION_ID: usize = 0x32;
pub const GLOBALIZE_COMPONENT_FUNCTION_NAME: &str = "globalize_component";
pub const CALL_METHOD_FUNCTION_ID: usize = 0x33;
pub const CALL_METHOD_FUNCTION_NAME: &str = "call_method";
pub const LOOKUP_GLOBAL_COMPONENT_FUNCTION_ID: usize = 0x34;
pub const LOOKUP_GLOBAL_COMPONENT_FUNCTION_NAME: &str = "lookup_global_component";
pub const GET_COMPONENT_TYPE_INFO_FUNCTION_ID: usize = 0x35;
pub const GET_COMPONENT_TYPE_INFO_FUNCTION_NAME: &str = "get_component_type_info";

pub const LOCK_SUBSTATE_FUNCTION_ID: usize = 0x40;
pub const LOCK_SUBSTATE_FUNCTION_NAME: &str = "lock_substate";
pub const READ_SUBSTATE_FUNCTION_ID: usize = 0x41;
pub const READ_SUBSTATE_FUNCTION_NAME: &str = "read_substate";
pub const WRITE_SUBSTATE_FUNCTION_ID: usize = 0x42;
pub const WRITE_SUBSTATE_FUNCTION_NAME: &str = "write_substate";
pub const DROP_LOCK_FUNCTION_ID: usize = 0x43;
pub const DROP_LOCK_FUNCTION_NAME: &str = "drop_lock";

// Under active refactoring
pub const GET_ACTOR_FUNCTION_ID: usize = 0xf0;
pub const GET_ACTOR_FUNCTION_NAME: &str = "get_actor";
pub const CALL_NATIVE_FUNCTION_ID: usize = 0xf1;
pub const CALL_NATIVE_FUNCTION_NAME: &str = "call_native";
pub const DROP_NODE_FUNCTION_ID: usize = 0xf2;
pub const DROP_NODE_FUNCTION_NAME: &str = "drop_node";

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
