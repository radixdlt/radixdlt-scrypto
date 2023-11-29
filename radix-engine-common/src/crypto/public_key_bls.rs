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

impl Describe<ScryptoCustomTypeKind> for BlsPublicKey {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::BLS_PUBLIC_KEY_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::bls_public_key_type_data()
    }
}

impl BlsPublicKey {
    pub const LENGTH: usize = 48;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn to_hash(&self) -> BlsPublicKeyHash {
        BlsPublicKeyHash::new_from_public_key(self)
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
// hash
//======

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Categorize, Encode, Decode, BasicDescribe)]
#[sbor(transparent)]
pub struct BlsPublicKeyHash(pub [u8; Self::LENGTH]);

impl Describe<ScryptoCustomTypeKind> for BlsPublicKeyHash {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::BLS_PUBLIC_KEY_HASH_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::bls_public_key_hash_type_data()
    }
}

impl BlsPublicKeyHash {
    pub const LENGTH: usize = NodeId::RID_LENGTH;

    pub fn new_from_public_key(public_key: &BlsPublicKey) -> Self {
        Self(hash_public_key_bytes(public_key.0))
    }
}

impl HasPublicKeyHash for BlsPublicKey {
    type TypedPublicKeyHash = BlsPublicKeyHash;

    fn get_hash(&self) -> Self::TypedPublicKeyHash {
        Self::TypedPublicKeyHash::new_from_public_key(self)
    }
}

impl IsPublicKeyHash for BlsPublicKeyHash {
    fn get_hash_bytes(&self) -> &[u8; Self::LENGTH] {
        &self.0
    }

    fn into_enum(self) -> PublicKeyHash {
        PublicKeyHash::Bls(self)
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
