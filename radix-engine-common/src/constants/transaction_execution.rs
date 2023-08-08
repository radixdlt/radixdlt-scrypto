/// The system loan amount, used by transaction executor.
pub const SYSTEM_LOAN_AMOUNT: u32 = 10_000_000;

/// The cost unit limit.
pub const COST_UNIT_LIMIT: u32 = 100_000_000;

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
pub const MAX_SUBSTATE_SIZE: usize = 2 * 1024 * 1024;

/// The maximum invoke payload size.
pub const MAX_INVOKE_PAYLOAD_SIZE: usize = 1 * 1024 * 1024;

/// The proposer's share of tips
pub const TIPS_PROPOSER_SHARE_PERCENTAGE: u8 = 100;

/// The validator set's share of tips
pub const TIPS_VALIDATOR_SET_SHARE_PERCENTAGE: u8 = 0;

/// The tips to burn
pub const TIPS_TO_BURN_PERCENTAGE: u8 = 0;

/// The proposer's share of fees (execution and state expansion)
pub const FEES_PROPOSER_SHARE_PERCENTAGE: u8 = 25;

/// The validator set's share of fees  (execution and state expansion)
pub const FEES_VALIDATOR_SET_SHARE_PERCENTAGE: u8 = 25;

/// The fees to burn
pub const FEES_TO_BURN_PERCENTAGE: u8 = 50;

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

//==========================
// TO BE DEFINED
//==========================

/// The cost unit price, in XRD.
pub const COST_UNIT_PRICE_IN_XRD: &str = "0.00000001";

/// The price for adding a single byte to the substate store, in XRD.
pub const STATE_EXPANSION_PRICE_IN_XRD: &str = "0.00001";

/// The USD price, in XRD
pub const USD_PRICE_IN_XRD: &str = "10";

/// The maximum that a package or component owner is allowed to set their method royalty to
pub const MAX_PER_FUNCTION_ROYALTY_IN_XRD: &str = "150.0";
