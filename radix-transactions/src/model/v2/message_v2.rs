use super::*;
use crate::internal_prelude::*;

/// Transaction messages as per REP-70.
/// The only difference from V1 is that the AES key has been increased
/// up to 256 bits for the encrypted messages.
#[derive(Debug, Clone, Eq, PartialEq, Default, ManifestSbor, ScryptoDescribe)]
pub enum MessageV2 {
    #[default]
    None,
    Plaintext(PlaintextMessageV1),
    Encrypted(EncryptedMessageV2),
}

impl TransactionPartialPrepare for MessageV2 {
    type Prepared = PreparedMessageV2;
}

//============================================================================
// ENCRYPTED MESSAGE
//============================================================================

/// A `PlaintextMessageV1` encrypted with "MultiPartyECIES" for a number of decryptors (public keys).
///
/// First, a `PlaintextMessageV1` should be created, and encoded as `manifest_sbor_encode(plaintext_message)`
/// to get the plaintext message payload bytes.
///
/// The plaintext message payload bytes are encrypted via (256-bit) AES-GCM with an ephemeral symmetric key.
///
/// The (256-bit) AES-GCM symmetric key is encrypted separately for each decryptor public key via (256-bit) AES-KeyWrap.
/// AES-KeyWrap uses a key derived via a KDF (Key Derivation Function) using a shared secret.
/// For each decryptor public key, we create a shared curve point `G` via static Diffie-Helman between the
/// decryptor public key, and a per-transaction ephemeral public key for that curve type.
/// We then use that shared secret with a key derivation function to create the (256-bit) KEK (Key Encrypting Key):
/// `KEK = HKDF(hash: Blake2b, secret: x co-ord of G, salt: [], length: 256 bits)`.
///
/// Note:
/// - For ECDH, the secret we use is the `x` coordinate of the shared public point, unhashed. This ECDH output is
///   known as ASN1 X9.63 variant of ECDH. Be careful - libsecp256k1 uses another non-standard variant.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct EncryptedMessageV2 {
    pub encrypted: AesGcmPayload,
    // Note we use a collection here rather than a struct to be forward-compatible to adding more curve types.
    // The engine should validate each DecryptorsByCurve matches the CurveType.
    pub decryptors_by_curve: IndexMap<CurveType, DecryptorsByCurveV2>,
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub enum DecryptorsByCurveV2 {
    Ed25519 {
        dh_ephemeral_public_key: Ed25519PublicKey,
        decryptors: IndexMap<PublicKeyFingerprint, AesWrapped256BitKey>,
    },
    Secp256k1 {
        dh_ephemeral_public_key: Secp256k1PublicKey,
        decryptors: IndexMap<PublicKeyFingerprint, AesWrapped256BitKey>,
    },
}

impl DecryptorsByCurveV2 {
    pub fn curve_type(&self) -> CurveType {
        match self {
            Self::Ed25519 { .. } => CurveType::Ed25519,
            Self::Secp256k1 { .. } => CurveType::Secp256k1,
        }
    }

    pub fn number_of_decryptors(&self) -> usize {
        match self {
            Self::Ed25519 { decryptors, .. } => decryptors.len(),
            Self::Secp256k1 { decryptors, .. } => decryptors.len(),
        }
    }
}

/// The wrapped key bytes from applying 256-bit AES-KeyWrap from RFC-3394
/// to the 256-bit message ephemeral public key, with the secret KEK provided by
/// static Diffie-Helman between the decryptor public key, and the `dh_ephemeral_public_key`
/// for that curve type.
///
/// This must be serialized as per https://www.ietf.org/rfc/rfc3394.txt as `IV || Cipher` where:
/// * IV: First 8 bytes
/// * Cipher: The wrapped 256 bit key, encoded as four 64 bit blocks
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct AesWrapped256BitKey(pub [u8; Self::LENGTH]);

impl AesWrapped256BitKey {
    /// 8 bytes IV, and then 32 bytes of the encoded key
    pub const LENGTH: usize = 40;
}

//============================================================================
// PREPARATION
//============================================================================

pub type PreparedMessageV2 = SummarizedRawValueBody<MessageV2>;

// TODO: Add tests with a canonical implementation of message encryption/decryption,
// and corresponding test vectors for other implementers.
