// In order to distinguish payloads, all of these should be distinct!
// This is particularly important for payloads which will be signed (Transaction / ROLA)

/// 0x5c for [5c]rypto - (92 in decimal)
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

pub const SCRYPTO_SBOR_V1_MAX_DEPTH: usize = 64;

pub const MANIFEST_SBOR_V1_MAX_DEPTH: usize = 24;

/// Depth limit for the default value of a transient substate
pub const TRANSIENT_SUBSTATE_DEFAULT_VALUE_MAX_DEPTH: usize = BLUEPRINT_PAYLOAD_MAX_DEPTH;

/// Depth limit for various types of blueprint payload
/// - Function inputs and outputs
/// - Events
/// - Object Field
/// - Object KeyValue/Index/SortedIndex collection entry keys and values
pub const BLUEPRINT_PAYLOAD_MAX_DEPTH: usize = 48;

/// Depth limit for the key and value of an entry in `KeyValueStore`
pub const KEY_VALUE_STORE_PAYLOAD_MAX_DEPTH: usize = 48;

// Note that non-fungible data (transparent ScryptoValue) is soft limited by the function IO
// do we want a hard limit on it?
