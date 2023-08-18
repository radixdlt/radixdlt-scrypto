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

/// The max number of substates in track.
pub const MAX_NUMBER_OF_SUBSTATES_IN_TRACK: usize = 512;

/// The max number of substates in heap.
pub const MAX_NUMBER_OF_SUBSTATES_IN_HEAP: usize = 512;

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

/// The proposer's share of network fees (execution, finalization and storage)
pub const NETWORK_FEES_PROPOSER_SHARE_PERCENTAGE: u8 = 25;

/// The validator set's share of network fees (execution, finalization and storage)
pub const NETWORK_FEES_VALIDATOR_SET_SHARE_PERCENTAGE: u8 = 25;

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

/// The price of execution cost unit in XRD.
pub const EXECUTION_COST_UNIT_PRICE_IN_XRD: &str = "0.00000005";

/// The price of finalization cost unit in XRD.
pub const FINALIZATION_COST_UNIT_PRICE_IN_XRD: &str = "0.00000005";

/// The price for adding a single byte to the substate store in XRD: 1 MB = 10 USD
pub const STATE_STORAGE_PRICE_IN_XRD: &str = "0.000158945719401033"; // per byte

/// The USD price in XRD: 1 XRD = 0.06 USD
pub const USD_PRICE_IN_XRD: &str = "16.666666666666666666";

/// The maximum that a package or component owner is allowed to set their method royalty to: 10 USD
pub const MAX_PER_FUNCTION_ROYALTY_IN_XRD: &str = "166.666666666666666666";
