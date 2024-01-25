use crate::internal_prelude::*;
use blst::{
    min_pk::{AggregateSignature, Signature},
    BLST_ERROR,
};
use sbor::rust::borrow::ToOwned;
use sbor::rust::fmt;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
use sbor::*;
use utils::copy_u8_array;

/// BLS12-381 ciphersuite v1
/// It has following parameters
///  - hash-to-curve: BLS12381G2_XMD:SHA-256_SSWU_RO
///    - pairing-friendly elliptic curve: BLS12-381
///    - hash function: SHA-256
///    - signature variant: G2 minimal pubkey size
///  - scheme:
///    - proof-of-possession
/// More details: https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-bls-signature-04
pub const BLS12381_CIPHERSITE_V1: &[u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_";

/// Represents a BLS12-381 G2 signature (variant with 96-byte signature and 48-byte public key)
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct Bls12381G2Signature(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl Bls12381G2Signature {
    pub const LENGTH: usize = 96;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn to_native_signature(self) -> Result<Signature, ParseBlsSignatureError> {
        Signature::from_bytes(&self.0).map_err(|err| err.into())
    }

    /// Aggregate multiple signatures into a single one
    pub fn aggregate(signatures: &[Bls12381G2Signature]) -> Result<Self, ParseBlsSignatureError> {
        if !signatures.is_empty() {
            let sig_first = signatures[0].to_native_signature()?;

            let mut agg_sig = AggregateSignature::from_signature(&sig_first);

            for sig in signatures.iter().skip(1) {
                agg_sig.add_signature(&sig.to_native_signature()?, true)?;
            }
            Ok(Bls12381G2Signature(agg_sig.to_signature().to_bytes()))
        } else {
            Err(ParseBlsSignatureError::NoSignatureGiven)
        }
    }
}

impl TryFrom<&[u8]> for Bls12381G2Signature {
    type Error = ParseBlsSignatureError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Bls12381G2Signature::LENGTH {
            return Err(ParseBlsSignatureError::InvalidLength(slice.len()));
        }

        Ok(Bls12381G2Signature(copy_u8_array(slice)))
    }
}

//======
// error
//======

impl From<BLST_ERROR> for ParseBlsSignatureError {
    fn from(error: BLST_ERROR) -> Self {
        let err_msg = format!("{:?}", error);
        Self::BlsError(err_msg)
    }
}

/// Represents an error when retrieving BLS signature from hex or when aggregating.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ParseBlsSignatureError {
    InvalidHex(String),
    InvalidLength(usize),
    NoSignatureGiven,
    // Error returned by underlying BLS library
    BlsError(String),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBlsSignatureError {}

impl fmt::Display for ParseBlsSignatureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// text
//======

impl FromStr for Bls12381G2Signature {
    type Err = ParseBlsSignatureError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseBlsSignatureError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Bls12381G2Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Bls12381G2Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
