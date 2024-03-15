use crate::internal_prelude::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use blst::{
    min_pk::{AggregatePublicKey, PublicKey},
    BLST_ERROR,
};

/// Represents a BLS12-381 G1 public key.
#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct Bls12381G1PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl Bls12381G1PublicKey {
    pub const LENGTH: usize = 48;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    fn to_native_public_key(self) -> Result<PublicKey, ParseBlsPublicKeyError> {
        PublicKey::from_bytes(&self.0).map_err(|err| err.into())
    }

    /// Aggregate multiple public keys into a single one
    pub fn aggregate(public_keys: &[Bls12381G1PublicKey]) -> Result<Self, ParseBlsPublicKeyError> {
        if !public_keys.is_empty() {
            let pk_first = public_keys[0].to_native_public_key()?;

            let mut agg_pk = AggregatePublicKey::from_public_key(&pk_first);

            for pk in public_keys.iter().skip(1) {
                agg_pk.add_public_key(&pk.to_native_public_key()?, true)?;
            }
            Ok(Bls12381G1PublicKey(agg_pk.to_public_key().to_bytes()))
        } else {
            Err(ParseBlsPublicKeyError::NoPublicKeysGiven)
        }
    }
}

impl TryFrom<&[u8]> for Bls12381G1PublicKey {
    type Error = ParseBlsPublicKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Bls12381G1PublicKey::LENGTH {
            return Err(ParseBlsPublicKeyError::InvalidLength(slice.len()));
        }

        Ok(Bls12381G1PublicKey(copy_u8_array(slice)))
    }
}

//======
// error
//======

impl From<BLST_ERROR> for ParseBlsPublicKeyError {
    fn from(error: BLST_ERROR) -> Self {
        let err_msg = format!("{:?}", error);
        Self::BlsError(err_msg)
    }
}

/// Represents an error when retrieving BLS public key from hex or when aggregating.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ParseBlsPublicKeyError {
    InvalidHex(String),
    InvalidLength(usize),
    NoPublicKeysGiven,
    // Error returned by underlying BLS library
    BlsError(String),
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

impl FromStr for Bls12381G1PublicKey {
    type Err = ParseBlsPublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes = hex::decode(s).map_err(|_| ParseBlsPublicKeyError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Bls12381G1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Bls12381G1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
