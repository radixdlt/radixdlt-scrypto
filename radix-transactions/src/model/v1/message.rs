use super::*;
use crate::internal_prelude::*;

/// Transaction messages as per REP-70
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub enum MessageV1 {
    None,
    Plaintext(PlaintextMessageV1),
    Encrypted(EncryptedMessageV1),
}

impl Default for MessageV1 {
    fn default() -> Self {
        Self::None
    }
}

//============================================================================
// PLAINTEXT MESSAGE
//============================================================================

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct PlaintextMessageV1 {
    pub mime_type: String,
    pub message: MessageContentsV1,
}

impl PlaintextMessageV1 {
    pub fn text(message: impl Into<String>) -> Self {
        Self {
            mime_type: "text/plain".to_string(),
            message: MessageContentsV1::String(message.into()),
        }
    }
}

/// We explicitly mark content as either String or Bytes - this distinguishes (along with the mime type)
/// whether the message is intended to be displayable as text, or not.
///
/// This data model ensures that messages intended to be displayable as text are valid unicode strings.
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub enum MessageContentsV1 {
    String(String),
    Bytes(Vec<u8>),
}

impl MessageContentsV1 {
    pub fn len(&self) -> usize {
        match self {
            MessageContentsV1::String(message) => message.len(),
            MessageContentsV1::Bytes(message) => message.len(),
        }
    }
}

//============================================================================
// ENCRYPTED MESSAGE
//============================================================================

/// A `PlaintextMessageV1` encrypted with "MultiPartyECIES" for a number of decryptors (public keys).
///
/// First, a `PlaintextMessageV1` should be created, and encoded as `manifest_sbor_encode(plaintext_message)`
/// to get the plaintext message payload bytes.
///
/// The plaintext message payload bytes are encrypted via (128-bit) AES-GCM with an ephemeral symmetric key.
///
/// The (128-bit) AES-GCM symmetric key is encrypted separately for each decryptor public key via (256-bit) AES-KeyWrap.
/// AES-KeyWrap uses a key derived via a KDF (Key Derivation Function) using a shared secret.
/// For each decryptor public key, we create a shared curve point `G` via static Diffie-Helman between the
/// decryptor public key, and a per-transaction ephemeral public key for that curve type.
/// We then use that shared secret with a key derivation function to create the (256-bit) KEK (Key Encrypting Key):
/// `KEK = HKDF(hash: Blake2b, secret: x co-ord of G, salt: [], length: 256 bits)`.
///
/// Note:
/// - For ECDH, the secret we use is the `x` coordinate of the shared public point, unhashed. This ECDH output is
///   known as ASN1 X9.63 variant of ECDH. Be careful - libsecp256k1 uses another non-standard variant.
/// - We persist 128-bit symmetric keys because we wish to save on payload size, and:
///   * 128-bit AES is considered secure enough for most use cases (EG bitcoin hash rate is only 2^93 / year)
///   * It's being used with a transient key - so a hypothetical successful attack would only decrypt one message
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct EncryptedMessageV1 {
    pub encrypted: AesGcmPayload,
    // Note we use a collection here rather than a struct to be forward-compatible to adding more curve types.
    // The engine should validate each DecryptorsByCurve matches the CurveType.
    pub decryptors_by_curve: IndexMap<CurveType, DecryptorsByCurve>,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ManifestSbor, ScryptoDescribe,
)]
pub enum CurveType {
    Ed25519,
    Secp256k1,
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub enum DecryptorsByCurve {
    Ed25519 {
        dh_ephemeral_public_key: Ed25519PublicKey,
        decryptors: IndexMap<PublicKeyFingerprint, AesWrapped128BitKey>,
    },
    Secp256k1 {
        dh_ephemeral_public_key: Secp256k1PublicKey,
        decryptors: IndexMap<PublicKeyFingerprint, AesWrapped128BitKey>,
    },
}

impl DecryptorsByCurve {
    pub fn curve_type(&self) -> CurveType {
        match self {
            DecryptorsByCurve::Ed25519 { .. } => CurveType::Ed25519,
            DecryptorsByCurve::Secp256k1 { .. } => CurveType::Secp256k1,
        }
    }

    pub fn number_of_decryptors(&self) -> usize {
        match self {
            DecryptorsByCurve::Ed25519 { decryptors, .. } => decryptors.len(),
            DecryptorsByCurve::Secp256k1 { decryptors, .. } => decryptors.len(),
        }
    }
}

/// The last 8 bytes of the Blake2b-256 hash of the public key bytes,
/// in their standard Radix byte-serialization.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct PublicKeyFingerprint(pub [u8; Self::LENGTH]);

impl PublicKeyFingerprint {
    pub const LENGTH: usize = 8;
}

impl From<PublicKey> for PublicKeyFingerprint {
    fn from(value: PublicKey) -> Self {
        value.get_hash().into()
    }
}

impl From<PublicKeyHash> for PublicKeyFingerprint {
    fn from(value: PublicKeyHash) -> Self {
        let hash_bytes = value.get_hash_bytes();
        let fingerprint_bytes = &hash_bytes[(hash_bytes.len() - Self::LENGTH)..hash_bytes.len()];
        PublicKeyFingerprint(copy_u8_array(fingerprint_bytes))
    }
}

/// The (128-bit) AES-GCM encrypted bytes of the payload.
///
/// This must be serialized as the concatenation `Nonce/IV || Cipher || Tag/MAC` where:
/// * Nonce/IV: 12 bytes
/// * Cipher(text): Variable length
/// * Tag/MAC: 16 bytes
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct AesGcmPayload(pub Vec<u8>);

/// The wrapped key bytes from applying 256-bit AES-KeyWrap from RFC-3394
/// to the 128-bit message ephemeral public key, with the secret KEK provided by
/// static Diffie-Helman between the decryptor public key, and the `dh_ephemeral_public_key`
/// for that curve type.
///
/// This must be serialized as per https://www.ietf.org/rfc/rfc3394.txt as `IV || Cipher` where:
/// * IV: First 8 bytes
/// * Cipher: The wrapped 128 bit key, encoded as two 64 bit blocks
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct AesWrapped128BitKey(pub [u8; Self::LENGTH]);

impl AesWrapped128BitKey {
    /// 8 bytes IV, and then the encoded key
    pub const LENGTH: usize = 24;
}

//============================================================================
// PREPARATION
//============================================================================

#[allow(deprecated)]
pub type PreparedMessageV1 = SummarizedRawFullValue<MessageV1>;

// TODO: Add tests with a canonical implementation of message encryption/decryption,
// and corresponding test vectors for other implementers.
