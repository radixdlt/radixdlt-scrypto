// In order to distinguish payloads, all of these should be distinct!
// This is particularly important for payloads which will be signed (Transaction / ROLA)

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

pub const SCRYPTO_SBOR_V1_MAX_DEPTH: usize = 64;

pub const MANIFEST_SBOR_V1_MAX_DEPTH: usize = 24;
