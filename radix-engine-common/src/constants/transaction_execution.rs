/// The execution cost units loaned from system
pub const EXECUTION_COST_UNIT_LOAN: u32 = 10_000_000;

/// The execution cost unit limit.
pub const EXECUTION_COST_UNIT_LIMIT: u32 = 100_000_000;

/// The finalization cost unit limit.
pub const FINALIZATION_COST_UNIT_LIMIT: u32 = 50_000_000;

/// The free credit, for preview only.
pub const FREE_CREDIT_IN_XRD: &str = "100";

pub const MAX_EXECUTION_TRACE_DEPTH: usize = 16;

/// The max call depth, used by transaction executor.
pub const MAX_CALL_DEPTH: usize = 8;

/// The max total heap substate size.
pub const MAX_HEAP_SUBSTATE_TOTAL_BYTES: usize = 64 * 1024 * 1024;

/// The max total track substate size.
pub const MAX_TRACK_SUBSTATE_TOTAL_BYTES: usize = 64 * 1024 * 1024;

/// The maximum substate key read and write size
pub const MAX_SUBSTATE_KEY_SIZE: usize = 1024;

/// The maximum substate read and write size.
pub const MAX_SUBSTATE_VALUE_SIZE: usize = 2 * 1024 * 1024;

/// The maximum invoke payload size.
pub const MAX_INVOKE_PAYLOAD_SIZE: usize = 1 * 1024 * 1024;

/// The proposer's share of tips
pub const TIPS_PROPOSER_SHARE_PERCENTAGE: u8 = 100;

/// The validator set's share of tips
pub const TIPS_VALIDATOR_SET_SHARE_PERCENTAGE: u8 = 0;

/// The tips to burn
pub const TIPS_TO_BURN_PERCENTAGE: u8 = 0;

/// The proposer's share of network fees (execution, finalization and storage)
pub const NETWORK_FEES_PROPOSER_SHARE_PERCENTAGE: u8 = 25;

/// The validator set's share of network fees (execution, finalization and storage)
pub const NETWORK_FEES_VALIDATOR_SET_SHARE_PERCENTAGE: u8 = 25;

/// The network fees (execution, finalization and storage) to burn
pub const NETWORK_FEES_TO_BURN_PERCENTAGE: u8 = 50;

/// The max event size
pub const MAX_EVENT_SIZE: usize = 64 * 1024;

/// The max log size
pub const MAX_LOG_SIZE: usize = 64 * 1024;

/// The max panic message size
pub const MAX_PANIC_MESSAGE_SIZE: usize = 64 * 1024;

/// The max number of events
pub const MAX_NUMBER_OF_EVENTS: usize = 256;

/// The max number of logs
pub const MAX_NUMBER_OF_LOGS: usize = 256;

/// The max SBOR size of metadata key
pub const MAX_METADATA_KEY_STRING_LEN: usize = 100;

/// The max SBOR size of metadata value
pub const MAX_METADATA_VALUE_SBOR_LEN: usize = 512;

/// The max depth of an access rule, to protect unbounded native stack useage
pub const MAX_ACCESS_RULE_DEPTH: usize = 8;

/// The max number of access rule nodes in an access rule
pub const MAX_ACCESS_RULE_NODES: usize = 64;

/// The max number of roles in a Role Specification
pub const MAX_ROLES: usize = 50;

/// The max number of roles in a Role Specification
pub const MAX_ROLE_NAME_LEN: usize = 100;

//==========================
// TO BE DEFINED
//==========================

/// The price of execution cost unit, in XRD.
pub const EXECUTION_COST_UNIT_PRICE_IN_XRD: &str = "0.00000001";

/// The price of finalization cost unit, in XRD.
pub const FINALIZATION_COST_UNIT_PRICE_IN_XRD: &str = "0.00000001";

/// The price for adding a single byte to the substate store, in XRD.
pub const STORAGE_PRICE_IN_XRD: &str = "0.00001";

/// The USD price, in XRD
pub const USD_PRICE_IN_XRD: &str = "10";

/// The maximum that a package or component owner is allowed to set their method royalty to
pub const MAX_PER_FUNCTION_ROYALTY_IN_XRD: &str = "150.0";
