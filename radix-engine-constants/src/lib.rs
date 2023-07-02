#![no_std]

//==========================
// Prefix bytes
//==========================

// In order to distinguish payloads, all of these should be distinct!
// This is particularly important for payloads which will be signed (Transaction / ROLA)

// 0x5b for [5b]or - (90 in decimal)
// The following is exported from the sbor repo, but commented out here to avoid import clashes:
// pub const BASIC_SBOR_V1_PAYLOAD_PREFIX: u8 = 0x5b;

/// 0x5c for [5c]rypto - (91 in decimal)
pub const SCRYPTO_SBOR_V1_PAYLOAD_PREFIX: u8 = 0x5c;
/// 0x4d = M in ASCII for Manifest - (77 in decimal)
pub const MANIFEST_SBOR_V1_PAYLOAD_PREFIX: u8 = 0x4d;

/// The ROLA hash which is signed is created as `hash(ROLA_HASHABLE_PAYLOAD_PREFIX || ..)`
///
/// 0x52 = R in ASCII for ROLA - (82 in decimal)
pub const ROLA_HASHABLE_PAYLOAD_PREFIX: u8 = 0x52;
/// The Transaction hash which is signed is created as:
/// `hash(TRANSACTION_HASHABLE_PAYLOAD_PREFIX || version prefix according to type of transaction payload || ..)`
///
/// 0x54 = T in ASCII for Transaction - (84 in decimal)
pub const TRANSACTION_HASHABLE_PAYLOAD_PREFIX: u8 = 0x54;

//==========================
// Transaction construction
//==========================

pub const DEFAULT_TIP_PERCENTAGE: u16 = 5;

//==========================
// Transaction validation
//==========================

pub const TRANSACTION_VERSION_V1: u8 = 1;
pub const MAX_NUMBER_OF_INTENT_SIGNATURES: usize = 16;
pub const MAX_NUMBER_OF_BLOBS: usize = 64;

/// The minimum value of cost unit limit
pub const DEFAULT_MIN_COST_UNIT_LIMIT: u32 = 1_000_000;

/// The maximum value of cost unit limit
pub const DEFAULT_MAX_COST_UNIT_LIMIT: u32 = 100_000_000;

/// The minimum value of tip percentage
pub const DEFAULT_MIN_TIP_PERCENTAGE: u16 = 0;

/// The maximum value of tip percentage
pub const DEFAULT_MAX_TIP_PERCENTAGE: u16 = u16::MAX;

/// The max epoch range
/// Should be ~ 1 month. The below is ~30 days given 5 minute epochs.
pub const DEFAULT_MAX_EPOCH_RANGE: u64 = 12 * 24 * 30;

/// The max transaction size
pub const DEFAULT_MAX_TRANSACTION_SIZE: usize = 1 * 1024 * 1024;

//==========================
// Transaction execution
//==========================

/// The default system loan amount, used by transaction executor.
pub const DEFAULT_SYSTEM_LOAN: u32 = 10_000_000;

/// The default cost unit limit.
pub const DEFAULT_COST_UNIT_LIMIT: u32 = 100_000_000;

/// The default free credit, for preview only.
pub const DEFAULT_FREE_CREDIT_IN_XRD: &str = "100";

pub const DEFAULT_MAX_EXECUTION_TRACE_DEPTH: usize = 1;

/// The default max call depth, used by transaction executor.
pub const DEFAULT_MAX_CALL_DEPTH: usize = 8;

/// The default max number of substates in track.
pub const DEFAULT_MAX_NUMBER_OF_SUBSTATES_IN_TRACK: usize = 512;

/// The default max number of substates in heap.
pub const DEFAULT_MAX_NUMBER_OF_SUBSTATES_IN_HEAP: usize = 512;

/// The default maximum substate read and write size.
pub const DEFAULT_MAX_SUBSTATE_SIZE: usize = 2 * 1024 * 1024;

/// The default maximum invoke input args size.
pub const DEFAULT_MAX_INVOKE_INPUT_SIZE: usize = 1 * 1024 * 1024;

/// The proposer's share of tips
pub const TIPS_PROPOSER_SHARE_PERCENTAGE: u8 = 100;

/// The validator set's share of tips
pub const TIPS_VALIDATOR_SET_SHARE_PERCENTAGE: u8 = 0;

/// The proposer's share of fees (execution and state expansion)
pub const FEES_PROPOSER_SHARE_PERCENTAGE: u8 = 25;

/// The validator set's share of fees  (execution and state expansion)
pub const FEES_VALIDATOR_SET_SHARE_PERCENTAGE: u8 = 25;

/// The max event size
pub const DEFAULT_MAX_EVENT_SIZE: usize = 64 * 1024;

/// The max log size
pub const DEFAULT_MAX_LOG_SIZE: usize = 64 * 1024;

/// The max panic message size
pub const DEFAULT_MAX_PANIC_MESSAGE_SIZE: usize = 64 * 1024;

/// The max number of events
pub const DEFAULT_MAX_NUMBER_OF_EVENTS: usize = 256;

/// The max number of logs
pub const DEFAULT_MAX_NUMBER_OF_LOGS: usize = 256;

/// The max SBOR size of metadata key
pub const DEFAULT_MAX_METADATA_KEY_STRING_LEN: usize = 100;

/// The max SBOR size of metadata value
pub const DEFAULT_MAX_METADATA_VALUE_SBOR_LEN: usize = 512;

//==========================
// TO BE DEFINED
//==========================

/// The default cost unit price, in XRD.
pub const DEFAULT_COST_UNIT_PRICE_IN_XRD: &str = "0.00000001";

/// The default price for adding a single byte to the substate store, in XRD.
pub const DEFAULT_STATE_EXPANSION_PRICE_IN_XRD: &str = "0.00001";

/// The default USD price, in XRD
pub const DEFAULT_USD_PRICE_IN_XRD: &str = "10";

/// The default maximum that a package or component owner is allowed to set their method royalty to
pub const DEFAULT_MAX_PER_FUNCTION_ROYALTY_IN_XRD: &str = "150.0";
