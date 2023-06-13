#![no_std]

//==========================
// Prefix bytes
//==========================

// In order to distinguish payloads, all of these should be distinct!
// This is particularly important for payloads which will be signed (Transaction / ROLA)

/// 0x5b for [5b]or - (90 in decimal)
pub const BASIC_SBOR_V1_PAYLOAD_PREFIX: u8 = 0x5b; // Duplicated due to dependency issues
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
pub const DEFAULT_MAX_EPOCH_RANGE: u64 = 100;

/// The max transaction size
pub const DEFAULT_MAX_TRANSACTION_SIZE: usize = 1 * 1024 * 1024;

//==========================
// Transaction execution
//==========================

/// The default cost unit limit.
pub const DEFAULT_COST_UNIT_LIMIT: u32 = 100_000_000;

/// The default free credit, for preview only.
pub const DEFAULT_FREE_CREDIT_IN_XRD: u128 = 1_000_000_000_000_000_000_000u128;

/// The default system loan amount, used by transaction executor.
pub const DEFAULT_SYSTEM_LOAN: u32 = 10_000_000;

pub const DEFAULT_MAX_EXECUTION_TRACE_DEPTH: usize = 1;

/// The default max call depth, used by transaction executor.
pub const DEFAULT_MAX_CALL_DEPTH: usize = 8;

/// The default cost unit price.
pub const DEFAULT_COST_UNIT_PRICE: u128 = 100_000_000_000u128;

/// The default USD price
pub const DEFAULT_USD_PRICE: u128 = 14_000_000_000_000_000_000u128;

/// The default maximum WASM memory per transaction (multiple WASM instances up to call depth).
pub const DEFAULT_MAX_WASM_MEM_PER_TRANSACTION: usize = 10 * 1024 * 1024;

/// The default maximum WASM memory per WASM call frame.
pub const DEFAULT_MAX_WASM_MEM_PER_CALL_FRAME: usize = 4 * 1024 * 1024;

/// The default maximum substates reads count per transaction.
pub const DEFAULT_MAX_SUBSTATE_READS_PER_TRANSACTION: usize = 20_000;

/// The default maximum substates writes count per transaction.
pub const DEFAULT_MAX_SUBSTATE_WRITES_PER_TRANSACTION: usize = 5_000;

/// The default maximum substate read and write size.
/// TODO: Apply this limit in create_node too
pub const DEFAULT_MAX_SUBSTATE_SIZE: usize = 4 * 1024 * 1024;

/// The default maximum invoke input args size.
pub const DEFAULT_MAX_INVOKE_INPUT_SIZE: usize = 4 * 1024 * 1024;

/* Fees/tips distribution */
pub const TIPS_PROPOSER_SHARE_PERCENTAGE: u8 = 100;
pub const TIPS_VALIDATOR_SET_SHARE_PERCENTAGE: u8 = 0;
pub const FEES_PROPOSER_SHARE_PERCENTAGE: u8 = 25;
pub const FEES_VALIDATOR_SET_SHARE_PERCENTAGE: u8 = 25;
