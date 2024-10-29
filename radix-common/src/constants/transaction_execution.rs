/// The execution cost units loaned from system
pub const EXECUTION_COST_UNIT_LOAN: u32 = 4_000_000;

/// The execution cost unit limit.
pub const EXECUTION_COST_UNIT_LIMIT: u32 = 100_000_000;

/// The finalization cost unit limit.
pub const FINALIZATION_COST_UNIT_LIMIT: u32 = 50_000_000;

/// The free credit amount used by preview.
/// 1000000 XRD = $60000, which should be enough to cover all sensible preview requests.
pub const PREVIEW_CREDIT_IN_XRD: &str = "1000000";

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
#[cfg(not(feature = "coverage"))]
pub const MAX_SUBSTATE_VALUE_SIZE: usize = 2 * 1024 * 1024;
#[cfg(feature = "coverage")]
pub const MAX_SUBSTATE_VALUE_SIZE: usize = 64 * 1024 * 1024;

/// The maximum invoke payload size.
#[cfg(not(feature = "coverage"))]
pub const MAX_INVOKE_PAYLOAD_SIZE: usize = 1 * 1024 * 1024;
#[cfg(feature = "coverage")]
pub const MAX_INVOKE_PAYLOAD_SIZE: usize = 32 * 1024 * 1024;

/// The proposer's share of tips
pub const TIPS_PROPOSER_SHARE_PERCENTAGE: u8 = 100;

/// The validator set's share of tips
pub const TIPS_VALIDATOR_SET_SHARE_PERCENTAGE: u8 = 0;

/// The proposer's share of network fees (execution, finalization and storage)
pub const NETWORK_FEES_PROPOSER_SHARE_PERCENTAGE: u8 = 25;

/// The validator set's share of network fees (execution, finalization and storage)
pub const NETWORK_FEES_VALIDATOR_SET_SHARE_PERCENTAGE: u8 = 25;

/// The max event size
pub const MAX_EVENT_SIZE: usize = 32 * 1024;

/// The max log size
pub const MAX_LOG_SIZE: usize = 32 * 1024;

/// The max panic message size
pub const MAX_PANIC_MESSAGE_SIZE: usize = 32 * 1024;

/// The max number of events
pub const MAX_NUMBER_OF_EVENTS: usize = 256;

/// The max number of logs
pub const MAX_NUMBER_OF_LOGS: usize = 256;

/// The max SBOR size of metadata key
pub const MAX_METADATA_KEY_STRING_LEN: usize = 100;

/// The max SBOR size of metadata value
pub const MAX_METADATA_VALUE_SBOR_LEN: usize = 4096;

/// The max length of a URL in metadata
pub const MAX_URL_LENGTH: usize = 1024;

/// The max length of an Origin in metadata
pub const MAX_ORIGIN_LENGTH: usize = 1024;

/// The max depth of an access rule, to protect unbounded native stack usage
pub const MAX_ACCESS_RULE_DEPTH: usize = 8;

/// The max number of composite requirements in an access rule
pub const MAX_COMPOSITE_REQUIREMENTS: usize = 64;

/// The max number of roles in a Role Specification
pub const MAX_ROLES: usize = 50;

/// The max length of a role name
pub const MAX_ROLE_NAME_LEN: usize = 100;

/// The max length of a feature name
pub const MAX_FEATURE_NAME_LEN: usize = 100;

/// The max length of an event name
pub const MAX_EVENT_NAME_LEN: usize = 100;

/// The max length of a registered type name
pub const MAX_TYPE_NAME_LEN: usize = 100;

/// The max length of a blueprint identifier
pub const MAX_BLUEPRINT_NAME_LEN: usize = 100;

/// The max length of a function name
pub const MAX_FUNCTION_NAME_LEN: usize = 256;

/// The max number of fields a blueprint can have
pub const MAX_NUMBER_OF_BLUEPRINT_FIELDS: usize = 0xFF;

/// The price of execution cost unit, in XRD.
pub const EXECUTION_COST_UNIT_PRICE_IN_XRD: &str = "0.00000005";

/// The price of finalization cost unit, in XRD.
pub const FINALIZATION_COST_UNIT_PRICE_IN_XRD: &str = "0.00000005";

/// The price for adding a single byte to state storage, in XRD. 1 MB = 6 USD
pub const STATE_STORAGE_PRICE_IN_XRD: &str = "0.00009536743";

/// The price for adding a single byte to archive storage, in XRD. 1 MB = 6 USD
/// This is primarily for transaction payload, events and logs.
pub const ARCHIVE_STORAGE_PRICE_IN_XRD: &str = "0.00009536743";

/// The USD price, in XRD. 1 XRD = 0.06 USD
pub const USD_PRICE_IN_XRD: &str = "16.666666666666666666";

/// The maximum that a package or component owner is allowed to set their method royalty to. 10 USD
pub const MAX_PER_FUNCTION_ROYALTY_IN_XRD: &str = "166.666666666666666666";
