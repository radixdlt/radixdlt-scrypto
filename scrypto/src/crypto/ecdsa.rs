use p256::ecdsa::{signature::Signer, Signature, SigningKey};
use p256::ecdsa::{signature::Verifier, VerifyingKey};
use p256::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
use p256::{EncodedPoint, PublicKey, SecretKey};
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::types::{scrypto_type, ScryptoType};

/// Represents an ECDSA public key.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EcdsaPublicKey(PublicKey);

/// Represents an ECDSA signature.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct EcdsaSignature(Signature);

/// Represents an error ocurred when validating a signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SignatureValidationError {}

/// Ecdsa signature verifier.
pub struct EcdsaVerifier;

/// Represents an ECDSA private key.
///
/// **Warning: ** This may be removed as whether signing capability should be provided by
/// Scrypto crypto library is controversial.
///
/// TODO: relocate to to another crate if not to be supported
///
pub struct EcdsaPrivateKey(SecretKey);

impl EcdsaPrivateKey {
    /* all public methods are confined to this impl */

    pub const LENGTH: usize = 32;

    pub fn public_key(&self) -> EcdsaPublicKey {
        EcdsaPublicKey(self.0.public_key())
    }

    pub fn sign(&self, msg: &[u8]) -> EcdsaSignature {
        let signer = SigningKey::from(&self.0);
        EcdsaSignature(signer.sign(msg))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.to_be_bytes().as_slice().to_vec()
    }

    pub fn from_bytes(slice: &[u8]) -> Result<Self, ()> {
        if slice.len() != EcdsaPrivateKey::LENGTH {
            return Err(());
        }
        Ok(Self(SecretKey::from_be_bytes(slice).map_err(|_| ())?))
    }
}

impl EcdsaPublicKey {
    // uncompressed
    pub const LENGTH: usize = 65;
}

impl EcdsaSignature {
    pub const LENGTH: usize = 64;
}

impl EcdsaVerifier {
    pub fn verify(msg: &[u8], pk: &EcdsaPublicKey, sig: &EcdsaSignature) -> bool {
        let verifier = VerifyingKey::from(pk.0);
        verifier.verify(msg, &sig.0).is_ok()
    }
}

//======
// error
//======

/// Represents an error when parsing ECDSA public key from hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseEcdsaPublicKeyError {
    InvalidHex(String),
    InvalidLength(usize),
    InvalidKey,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEcdsaPublicKeyError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseEcdsaPublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseEcdsaSignatureError {
    InvalidHex(String),
    InvalidLength(usize),
    InvalidSignature,
}

/// Represents an error when parsing ECDSA signature from hex.
#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEcdsaSignatureError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseEcdsaSignatureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// binary
//======

impl TryFrom<&[u8]> for EcdsaPublicKey {
    type Error = ParseEcdsaPublicKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != EcdsaPublicKey::LENGTH {
            return Err(ParseEcdsaPublicKeyError::InvalidLength(slice.len()));
        }

        let pk = PublicKey::from_encoded_point(
            &EncodedPoint::from_bytes(slice).map_err(|_| ParseEcdsaPublicKeyError::InvalidKey)?,
        );
        if pk.is_some().unwrap_u8() > 0 {
            Ok(EcdsaPublicKey(pk.unwrap()))
        } else {
            Err(ParseEcdsaPublicKeyError::InvalidKey)
        }
    }
}

impl EcdsaPublicKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_encoded_point(false).as_bytes().to_vec()
    }
}

scrypto_type!(EcdsaPublicKey, ScryptoType::EcdsaPublicKey, Vec::new());

impl TryFrom<&[u8]> for EcdsaSignature {
    type Error = ParseEcdsaSignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != EcdsaSignature::LENGTH {
            return Err(ParseEcdsaSignatureError::InvalidLength(slice.len()));
        }

        let signature =
            Signature::try_from(slice).map_err(|_| ParseEcdsaSignatureError::InvalidSignature)?;
        Ok(EcdsaSignature(signature))
    }
}

impl EcdsaSignature {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

scrypto_type!(EcdsaSignature, ScryptoType::EcdsaSignature, Vec::new());

//======
// text
//======

impl FromStr for EcdsaPublicKey {
    type Err = ParseEcdsaPublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseEcdsaPublicKeyError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for EcdsaPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for EcdsaPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

impl FromStr for EcdsaSignature {
    type Err = ParseEcdsaSignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseEcdsaSignatureError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for EcdsaSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for EcdsaSignature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::Hash;

    #[test]
    fn sign_and_verify() {
        // From Babylon Wallet PoC
        let test_sk = "0000000000000000000000000000000000000000000000000000000000000001";
        let test_pk = "046b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c2964fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5";
        let test_message = "{\"a\":\"banan\"}";
        let test_hash = "c43a1e3a7e822c97004267324ba8df88d114ab3e019d0e85eccb1ff8592d6d36";
        let test_signature = "468764c570758020eb8392e40de5805757d6e563a507f12ddde56463c23820e10401cae1684cb350bc3ecb45965ee259964f931eb4c165cd1a270fc538b65a75";
        let sk = EcdsaPrivateKey::from_bytes(&hex::decode(test_sk).unwrap()).unwrap();
        let pk = EcdsaPublicKey::from_str(test_pk).unwrap();
        let hash = Hash::from_str(test_hash).unwrap();
        let sig = EcdsaSignature::from_str(test_signature).unwrap();

        assert_eq!(sk.public_key(), pk);
        assert_eq!(crate::crypto::hash(test_message), hash);
        assert_eq!(sk.sign(test_message.as_bytes()), sig);
        assert!(EcdsaVerifier::verify(test_message.as_bytes(), &pk, &sig));
    }
}
