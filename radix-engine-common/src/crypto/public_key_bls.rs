use super::*;
use crate::internal_prelude::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;

/// Represents a BLS public key.
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Categorize, Encode, Decode, BasicDescribe,
)]
#[sbor(transparent)]
pub struct BlsPublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);
/*
impl Describe<ScryptoCustomTypeKind> for BlsPublicKey {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::BLS_PUBLIC_KEY_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::secp256k1_public_key_type_data()
    }
}
*/
impl BlsPublicKey {
    pub const LENGTH: usize = 48;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl TryFrom<&[u8]> for BlsPublicKey {
    type Error = ParseBlsPublicKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != BlsPublicKey::LENGTH {
            return Err(ParseBlsPublicKeyError::InvalidLength(slice.len()));
        }

        Ok(BlsPublicKey(copy_u8_array(slice)))
    }
}

//======
// error
//======

/// Represents an error when parsing ED25519 public key from hex.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseBlsPublicKeyError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseBlsPublicKeyError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseBlsPublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// text
//======

impl FromStr for BlsPublicKey {
    type Err = ParseBlsPublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseBlsPublicKeyError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for BlsPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for BlsPublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
