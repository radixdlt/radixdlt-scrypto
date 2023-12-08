use crate::internal_prelude::*;
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
#[derive(
    Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Categorize, Encode, Decode, BasicDescribe,
)]
#[sbor(transparent)]
pub struct Bls12381G2Signature(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl Describe<ScryptoCustomTypeKind> for Bls12381G2Signature {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::BLS12381G2_SIGNATURE_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::bls12381g2_signature_type_data()
    }
}

impl Bls12381G2Signature {
    pub const LENGTH: usize = 96;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ParseBlsSignatureError {
    InvalidHex(String),
    InvalidLength(usize),
}

/// Represents an error when parsing BLS signature from hex.
#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBlsSignatureError {}

#[cfg(not(feature = "alloc"))]
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
