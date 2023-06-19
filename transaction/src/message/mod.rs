use crate::internal_prelude::*;

//==============================================================================================================
// ENCRYPTED MESSAGES
//==============================================================================================================
// This module provides an example implementation of message encryption.
//
// Crates chosen from https://cryptography.rs/ - with a preference to https://github.com/RustCrypto for consistency - docs:
// - Main encryption:
//   => AES-GCM: https://docs.rs/aes-gcm/0.10.2/aes_gcm/index.html
// - Static Diffie-Helman:
//   => Curve25519: https://docs.rs/x25519-dalek/1.2.0/x25519_dalek/index.html
//      which also needed to pull in its dependencies explicitly: curve25519-dalek and rand_core
//   => ECDH Secp256k1: https://docs.rs/k256/latest/k256/index.html
// - Key wrapping:
//   => HKDF for key derivation: https://docs.rs/hkdf/0.12.3/hkdf/index.html
//   => AES Key Wrap: https://docs.rs/aes-kw/0.2.1/aes_kw/index.html
//==============================================================================================================

// TODO:
// - Better handling of zeroizing private keys
// - There is a version clash where curve25519-dalek 3.x requires zeroize version = ">=1, <1.4":
//   https://github.com/dalek-cryptography/curve25519-dalek/blob/3.2.1/Cargo.toml
//   BUT other crypto libraries such as https://github.com/RustCrypto/elliptic-curves/blob/master/k256/Cargo.toml
//   require later versions. As they're both on major version 1.x, Cargo just errors.
//
//   INSTEAD I suggest we get rid of x25519_dalek (as it's really not adding much) and update to curve25519-dalek v4
//   for this which has more sane dependency management
// - Implement Secp256k1 handling when the above zeroize issue is resolved:
//   Possibly with `k256 = { version = "0.13.1", default-features = false, features= ["arithmetic", "ecdh", "alloc"], optional = true }`
//   Although that might only support ECDH not static.

//============================================================================
// ENCRYPTED MESSAGE - ENCRYPTION
//============================================================================

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EncryptMessageError {
    NoDecryptors,
    EncodeError(EncodeError),
    AesGcmEncryptionError,
    AesKeyWrapError,
    PublicKeyMappingError(PublicKeyMappingError),
}

impl From<EncodeError> for EncryptMessageError {
    fn from(value: EncodeError) -> Self {
        Self::EncodeError(value)
    }
}

impl From<aes_kw::Error> for EncryptMessageError {
    fn from(_: aes_kw::Error) -> Self {
        Self::AesKeyWrapError
    }
}

impl From<PublicKeyMappingError> for EncryptMessageError {
    fn from(value: PublicKeyMappingError) -> Self {
        Self::PublicKeyMappingError(value)
    }
}

// TODO: Use `secrecy` or `zeroize` to better ensure this ephemeral key doesn't stay in memory
pub struct Aes128BitKey(pub [u8; Self::LENGTH]);

impl Aes128BitKey {
    pub const LENGTH: usize = 16;
}

pub fn encrypt_message(
    message: &PlaintextMessageV1,
    decryptors: &[PublicKey],
) -> Result<EncryptedMessageV1, EncryptMessageError> {
    let message_bytes = manifest_encode(message)?;
    encrypt_message_bytes(&message_bytes, decryptors)
}

pub fn encrypt_message_bytes(
    message_bytes: &[u8],
    decryptors: &[PublicKey],
) -> Result<EncryptedMessageV1, EncryptMessageError> {
    if decryptors.len() == 0 {
        return Err(EncryptMessageError::NoDecryptors);
    }

    let (encrypted, key_to_protect) = encrypt_plaintext_bytes(message_bytes)?;
    let mut public_keys_by_curve: BTreeMap<CurveType, Vec<&PublicKey>> = BTreeMap::default();
    for public_key in decryptors.iter() {
        let curve_type = CurveType::of(&public_key);
        public_keys_by_curve
            .entry(curve_type)
            .or_default()
            .push(&public_key);
    }

    let mut decryptors_by_curve: BTreeMap<CurveType, DecryptorsByCurve> = BTreeMap::default();
    for (curve_type, keys) in public_keys_by_curve {
        match curve_type {
            CurveType::Secp256k1 => {
                let dh_ephemeral_secret = Secp256k1Secret::new();
                let mut decryptors: BTreeMap<PublicKeyFingerprint, AesWrapped128BitKey> =
                    Default::default();

                for public_key in keys.into_iter() {
                    let secp256k1_public_key = match public_key {
                        PublicKey::Secp256k1(key) => key,
                        _ => {
                            panic!("Impossible public key type as mapped under curve type earlier")
                        }
                    };
                    let shared_secret = dh_ephemeral_secret
                        .create_unhashed_x_coord_shared_secret_bytes(secp256k1_public_key)?;
                    let wrapped_key = wrap_key(shared_secret.as_slice(), &key_to_protect)?;
                    let fingerprint = public_key.get_hash().into();

                    decryptors.insert(fingerprint, wrapped_key);
                }
                decryptors_by_curve.insert(
                    CurveType::Secp256k1,
                    DecryptorsByCurve::Secp256k1 {
                        dh_ephemeral_public_key: dh_ephemeral_secret.to_public_key(),
                        decryptors,
                    },
                );
            }
            CurveType::Ed25519 => {
                let dh_ephemeral_secret = Ed25519Secret::new();
                let mut decryptors: BTreeMap<PublicKeyFingerprint, AesWrapped128BitKey> =
                    Default::default();

                for public_key in keys.into_iter() {
                    let ed25519_public_key = match public_key {
                        PublicKey::Ed25519(key) => key,
                        _ => {
                            panic!("Impossible public key type as mapped under curve type earlier")
                        }
                    };
                    let shared_secret = dh_ephemeral_secret
                        .create_montgomery_u_coord_dh_shared_secret_bytes(&ed25519_public_key)?;
                    let wrapped_key = wrap_key(shared_secret.as_slice(), &key_to_protect)?;
                    let fingerprint = public_key.get_hash().into();

                    decryptors.insert(fingerprint, wrapped_key);
                }
                decryptors_by_curve.insert(
                    CurveType::Ed25519,
                    DecryptorsByCurve::Ed25519 {
                        dh_ephemeral_public_key: dh_ephemeral_secret.to_public_key(),
                        decryptors,
                    },
                );
            }
        }
    }

    Ok(EncryptedMessageV1 {
        encrypted,
        decryptors_by_curve,
    })
}

fn encrypt_plaintext_bytes(
    message_bytes: &[u8],
) -> Result<(AesGcmPayload, Aes128BitKey), EncryptMessageError> {
    use aes_gcm::aead::*;
    use aes_gcm::Aes128Gcm;

    // Note - whilst we could use AES-GCM-SIV here to create a deterministic nonce, it doesn't have widespread support
    // so instead, we use AES-GCM with a generated nonce.
    let key = Aes128Gcm::generate_key(OsRng);
    let cipher = Aes128Gcm::new(&key);
    let nonce = Aes128Gcm::generate_nonce(&mut OsRng);

    const NONCE_LENGTH: usize = 12;

    let ciphertext_and_mac = cipher
        .encrypt(&nonce, message_bytes)
        .map_err(|_| EncryptMessageError::AesGcmEncryptionError)?;

    let mut output = Vec::<u8>::with_capacity(NONCE_LENGTH + ciphertext_and_mac.len());

    output.extend_from_slice(&nonce);
    output.extend_from_slice(&ciphertext_and_mac);

    Ok((
        AesGcmPayload(output),
        Aes128BitKey(copy_u8_array(key.as_slice())),
    ))
}

//============================================================================
// ENCRYPTED MESSAGE  - DECRYPTION
//============================================================================

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DecryptMessageError {
    PayloadTooShort,
    AesGcmDecryptionError,
    DecodeError(DecodeError),
    DecryptorNotFoundForKey,
    MismatchingDecryptorsForCurve,
    AesKeyWrapError(String),
    PublicKeyMappingError(PublicKeyMappingError),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PublicKeyMappingError {
    PointDecompressionError,
}

impl From<DecodeError> for DecryptMessageError {
    fn from(value: DecodeError) -> Self {
        Self::DecodeError(value)
    }
}

impl From<aes_kw::Error> for DecryptMessageError {
    fn from(value: aes_kw::Error) -> Self {
        Self::AesKeyWrapError(value.to_string())
    }
}

impl From<PublicKeyMappingError> for DecryptMessageError {
    fn from(value: PublicKeyMappingError) -> Self {
        Self::PublicKeyMappingError(value)
    }
}

pub fn decrypt_message(
    message: &EncryptedMessageV1,
    private_key: &PrivateKey,
) -> Result<PlaintextMessageV1, DecryptMessageError> {
    let plaintext_bytes = decrypt_message_bytes(message, private_key)?;
    Ok(manifest_decode(&plaintext_bytes)?)
}

pub fn decrypt_message_bytes(
    message: &EncryptedMessageV1,
    private_key: &PrivateKey,
) -> Result<Vec<u8>, DecryptMessageError> {
    let decryptor_public_key = private_key.public_key();
    let curve_type = CurveType::of(&decryptor_public_key);
    let fingerprint: PublicKeyFingerprint = private_key.public_key().get_hash().into();

    let message_key = match private_key {
        PrivateKey::Secp256k1(private_key) => {
            let decryptors = message
                .decryptors_by_curve
                .get(&curve_type)
                .ok_or(DecryptMessageError::DecryptorNotFoundForKey)?;
            let (dh_ephemeral_public_key, wrapped_keys) = decryptors
                .as_secp256k1()
                .ok_or(DecryptMessageError::MismatchingDecryptorsForCurve)?;
            let wrapped_key = wrapped_keys
                .get(&fingerprint)
                .ok_or(DecryptMessageError::DecryptorNotFoundForKey)?;
            let dh_secret = Secp256k1Secret::from_private_key(private_key);
            let shared_secret =
                dh_secret.create_unhashed_x_coord_shared_secret_bytes(dh_ephemeral_public_key)?;
            unwrap_key(&shared_secret, wrapped_key)?
        }
        PrivateKey::Ed25519(private_key) => {
            let decryptors = message
                .decryptors_by_curve
                .get(&curve_type)
                .ok_or(DecryptMessageError::DecryptorNotFoundForKey)?;
            let (dh_ephemeral_public_key, wrapped_keys) = decryptors
                .as_ed25519()
                .ok_or(DecryptMessageError::MismatchingDecryptorsForCurve)?;
            let wrapped_key = wrapped_keys
                .get(&fingerprint)
                .ok_or(DecryptMessageError::DecryptorNotFoundForKey)?;

            let dh_secret = Ed25519Secret::from_private_key(private_key);
            let shared_secret = dh_secret
                .create_montgomery_u_coord_dh_shared_secret_bytes(dh_ephemeral_public_key)?;
            unwrap_key(&shared_secret, wrapped_key)?
        }
    };

    decrypt_plaintext_bytes(&message.encrypted, &message_key)
}

fn decrypt_plaintext_bytes(
    payload: &AesGcmPayload,
    key: &Aes128BitKey,
) -> Result<Vec<u8>, DecryptMessageError> {
    use aes_gcm::aead::*;
    use aes_gcm::Aes128Gcm;

    if payload.0.len() < 12 {
        return Err(DecryptMessageError::PayloadTooShort);
    }

    let key = Key::<Aes128Gcm>::clone_from_slice(&key.0);
    let cipher = Aes128Gcm::new(&key);

    let nonce = Nonce::<Aes128Gcm>::clone_from_slice(&payload.0[0..12]);
    let ciphertext_and_mac = &payload.0[12..];
    let plaintext_bytes = cipher
        .decrypt(&nonce, ciphertext_and_mac)
        .map_err(|_| DecryptMessageError::AesGcmDecryptionError)?;

    Ok(plaintext_bytes)
}

//============================================================================
// ENCRYPTED MESSAGE - COMMON
//============================================================================

fn wrap_key(
    shared_secret: &[u8],
    key_to_protect: &Aes128BitKey,
) -> Result<AesWrapped128BitKey, aes_kw::Error> {
    let kek = derive_key_encrypting_key(shared_secret);

    let mut wrapped = [0u8; AesWrapped128BitKey::LENGTH];
    kek.wrap(&key_to_protect.0, &mut wrapped)?;

    Ok(AesWrapped128BitKey(wrapped))
}

fn unwrap_key(
    shared_secret: &[u8],
    wrapped_key: &AesWrapped128BitKey,
) -> Result<Aes128BitKey, aes_kw::Error> {
    let kek = derive_key_encrypting_key(shared_secret);

    let mut unwrapped_key = [0u8; Aes128BitKey::LENGTH];
    kek.unwrap(&wrapped_key.0, &mut unwrapped_key)?;

    Ok(Aes128BitKey(unwrapped_key))
}

fn derive_key_encrypting_key(shared_secret: &[u8]) -> aes_kw::KekAes256 {
    // See https://docs.rs/hmac/0.12.1/hmac/ for details on why you need to use SimpleHmac
    let mut kek = [0u8; 32];
    let hkdf = hkdf::Hkdf::<Blake2b256, hmac::SimpleHmac<Blake2b256>>::new(None, shared_secret);
    // Safe unwrap - the error is only if the kek has an invalid length - but this 32 bytes is correct
    hkdf.expand(&[], &mut kek).unwrap();

    aes_kw::KekAes256::from(kek)
}

/// This forms a translation layer between x25519_dalek which represents the secret bytes in Montgomery form,
/// and Radix / ed25119_dalek which persists key bytes as the CompressedEdwardsY representation
struct Ed25519Secret(x25519_dalek::StaticSecret);

impl Ed25519Secret {
    fn new() -> Self {
        Ed25519Secret(x25519_dalek::StaticSecret::new(rand_core::OsRng))
    }

    fn from_private_key(private_key: &Ed25519PrivateKey) -> Self {
        Self(x25519_dalek::StaticSecret::from(
            private_key.to_scalar_bytes(),
        ))
    }

    /// Performs static diffie-helman to create a shared secret, and returns the shared secret
    /// of the (u)-coordinate of a point on the Montgomery form of Curve25519 or its twist.
    fn create_montgomery_u_coord_dh_shared_secret_bytes(
        &self,
        public_key: &Ed25519PublicKey,
    ) -> Result<[u8; 32], PublicKeyMappingError> {
        let dh_public_key = Self::public_key_to_dh_public_key(public_key)?;
        let dh_shared_secret = self.0.diffie_hellman(&dh_public_key);
        Ok(dh_shared_secret.to_bytes())
    }

    fn to_public_key(&self) -> Ed25519PublicKey {
        let scalar = curve25519_dalek::scalar::Scalar::from_bits(self.0.to_bytes());
        let point = &scalar * &curve25519_dalek::constants::ED25519_BASEPOINT_TABLE;
        let compressed = point.compress();
        Ed25519PublicKey(compressed.0)
    }

    fn public_key_to_dh_public_key(
        public_key: &Ed25519PublicKey,
    ) -> Result<x25519_dalek::PublicKey, PublicKeyMappingError> {
        let compressed = curve25519_dalek::edwards::CompressedEdwardsY::from_slice(&public_key.0);
        let point = compressed
            .decompress()
            .ok_or(PublicKeyMappingError::PointDecompressionError)?;
        let montgomery_point = point.to_montgomery();
        Ok(x25519_dalek::PublicKey::from(montgomery_point.0))
    }
}

struct Secp256k1Secret();

impl Secp256k1Secret {
    fn new() -> Self {
        todo!()
    }

    fn from_private_key(_private_key: &Secp256k1PrivateKey) -> Self {
        todo!()
    }

    /// Performs static diffie-helman to create a shared secret, and returns the shared secret
    /// of the unhashed x-coordinate of the shared point. This is known as the ASN1 X9.63 variant of ECDH.
    fn create_unhashed_x_coord_shared_secret_bytes(
        &self,
        _public_key: &Secp256k1PublicKey,
    ) -> Result<[u8; 32], PublicKeyMappingError> {
        todo!()
    }

    fn to_public_key(&self) -> Secp256k1PublicKey {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn ed25519_encryption_is_invertible() {
        let plaintext = "Hamburgers".as_bytes().to_vec();
        let private_key_1 = PrivateKey::from(
            Ed25519PrivateKey::from_scalar_bytes_with_empty_nonce(
                &hex::decode("182cdb27f613348aca4c6e34f350a8f9461fa03e0be1c90697937c74aef1c34a")
                    .unwrap(),
            )
            .unwrap(),
        );
        let public_key_1 = PublicKey::from(private_key_1.public_key());
        let private_key_2 = PrivateKey::from(
            Ed25519PrivateKey::from_scalar_bytes_with_empty_nonce(
                &hex::decode("f8290610725bc21d7067deae93acedd1a8981d3eb6c1d89b39dd82fe1daea150")
                    .unwrap(),
            )
            .unwrap(),
        );
        let public_key_2 = PublicKey::from(private_key_2.public_key());
        let encrypted = encrypt_message_bytes(&plaintext, &[public_key_1, public_key_2]).unwrap();

        let decrypted_1 = decrypt_message_bytes(&encrypted, &private_key_1).unwrap();
        assert_eq!(plaintext, decrypted_1);
        let decrypted_2 = decrypt_message_bytes(&encrypted, &private_key_2).unwrap();
        assert_eq!(plaintext, decrypted_2);
    }

    // TODO: Add tests with a canonical implementation of message encryption/decryption,
    // and corresponding test vectors for other implementers.
    #[test]
    #[ignore = "Doesn't work at present - see slack"]
    pub fn test_some_curve25519_test_vectors() {
        // From our other implementation
        let curve25519_private_key = Ed25519PrivateKey::from_seed_bytes(
            &hex::decode("880237bde777a014c5a8cb6f25734168919cda52a8789e483c95acac76b38478")
                .unwrap(),
        )
        .unwrap();
        let curve25519_public_key_1 = curve25519_private_key.public_key();
        let private_key_1 = PrivateKey::from(curve25519_private_key);
        assert_eq!(
            &curve25519_public_key_1,
            &Ed25519PublicKey(
                hex::decode("8a7e5a25f7f6d88c53530547ad0e676ae10fdf2e18f2f7281d6d8791ef6ce042")
                    .unwrap()
                    .try_into()
                    .unwrap()
            )
        );
        let encrypted_message = EncryptedMessageV1 {
            encrypted: AesGcmPayload(
                hex::decode("6a40ccc2518d35ae4e8ec42f4b168b6005b01bd8c3fa3fd1ba26974155a30c9949")
                    .unwrap(),
            ),
            decryptors_by_curve: btreemap!(
                CurveType::Ed25519 => DecryptorsByCurve::Ed25519 {
                    dh_ephemeral_public_key: Ed25519PublicKey(hex::decode("f8993156146bc0c3513691d2f77094dfe548b27c7397ed27e73a34b49444e673").unwrap().try_into().unwrap()),
                    decryptors: btreemap!(
                        PublicKeyFingerprint::from(private_key_1.public_key().get_hash())
                            => AesWrapped128BitKey(hex::decode("d06d379de6f836fc775059bf5eeb67274e9d2116bc761eb8").unwrap().try_into().unwrap())
                    ),
                }
            ),
        };
        decrypt_message_bytes(&encrypted_message, &private_key_1).expect("Could be decrypted");
    }
}
