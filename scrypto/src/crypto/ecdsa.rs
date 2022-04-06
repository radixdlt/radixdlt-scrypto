use p256::ecdsa::{signature::Signer, Signature, SigningKey};
use p256::ecdsa::{signature::Verifier, VerifyingKey};
use p256::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};

use p256::{EncodedPoint, PublicKey, SecretKey};
use sbor::*;

use crate::rust::borrow::ToOwned;
use crate::rust::fmt;
use crate::rust::str::FromStr;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::types::{scrypto_type, ScryptoType};

/// Represents an ECDSA private key.
#[derive(Clone, PartialEq, Eq)]
pub struct EcdsaPrivateKey(SecretKey);

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

impl EcdsaPrivateKey {
    pub const LENGTH: usize = 32;

    pub fn public_key(&self) -> EcdsaPublicKey {
        EcdsaPublicKey(self.0.public_key())
    }

    pub fn sign(&self, msg: &[u8]) -> EcdsaSignature {
        let signing_key = SigningKey::from(self.0.clone());
        EcdsaSignature(signing_key.sign(msg))
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
        let verifier = VerifyingKey::from_encoded_point(&pk.0.to_encoded_point(false)).unwrap();
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

/// Represents an error when parsing ECDSA private key from hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseEcdsaPrivateKeyError {
    InvalidHex(String),
    InvalidLength(usize),
    InvalidKey,
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEcdsaPrivateKeyError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseEcdsaPrivateKeyError {
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

impl TryFrom<&[u8]> for EcdsaPrivateKey {
    type Error = ParseEcdsaPrivateKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != EcdsaPrivateKey::LENGTH {
            return Err(ParseEcdsaPrivateKeyError::InvalidLength(slice.len()));
        }
        let sk =
            SecretKey::from_be_bytes(slice).map_err(|_| ParseEcdsaPrivateKeyError::InvalidKey)?;
        Ok(Self(sk))
    }
}

impl EcdsaPrivateKey {
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_be_bytes().as_slice().to_vec()
    }
}

// Temporarily for simulator
scrypto_type!(EcdsaPrivateKey, ScryptoType::EcdsaPrivateKey, Vec::new());

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

impl FromStr for EcdsaPrivateKey {
    type Err = ParseEcdsaPrivateKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseEcdsaPrivateKeyError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for EcdsaPrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for EcdsaPrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::string::ToString;

    #[test]
    fn sign_and_verify() {
        // From: https://asecuritysite.com/ecc/rust_ecdsa2
        let test_sk = "f348b118fa83ef25ecc9057da28e218ad29da650b0c3508defbb3541747180d2";
        let test_pk = "040c1ed3c9a585d0a756c63109606028012e12d66fa43dcdf272e2e03fe0c16e160b12877a334059614343ee289fe9d8b1752aacd79d2f22a4b2cc99ba54de5d02";
        let test_message = "Hello World!";
        let test_signature = "7a075925c0d2d454dbfe188779da3317a82ee152233c78e550a8424c7ed1cbc67f867375a307bb28c00d792a0182de7868f65e3471f74c852f419fe0bc9791b4";
        let sk = EcdsaPrivateKey::from_str(test_sk).unwrap();
        let pk = EcdsaPublicKey::from_str(test_pk).unwrap();
        let sig = EcdsaSignature::from_str(test_signature).unwrap();
        assert_eq!(sk.public_key(), pk);
        assert_eq!(sk.sign(test_message.as_bytes()), sig);
        assert!(EcdsaVerifier::verify(test_message.as_bytes(), &pk, &sig));

        assert_eq!(EcdsaPrivateKey::from_str(&sk.to_string()).unwrap(), sk);
        assert_eq!(EcdsaPublicKey::from_str(&pk.to_string()).unwrap(), pk);
        assert_eq!(EcdsaSignature::from_str(&sig.to_string()).unwrap(), sig);
    }
}
