use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;

/// Represents an ED25519 public key.
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Categorize, Encode, Decode, BasicDescribe,
)]
#[sbor(transparent)]
pub struct Ed25519PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl Describe<ScryptoCustomTypeKind> for Ed25519PublicKey {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::ED25519_PUBLIC_KEY_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::ed25519_public_key_type_data()
    }
}

impl Ed25519PublicKey {
    pub const LENGTH: usize = 32;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn to_hash(&self) -> Ed25519PublicKeyHash {
        Ed25519PublicKeyHash::new_from_public_key(self)
    }
}

impl TryFrom<&[u8]> for Ed25519PublicKey {
    type Error = ParseEd25519PublicKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Ed25519PublicKey::LENGTH {
            return Err(ParseEd25519PublicKeyError::InvalidLength(slice.len()));
        }

        Ok(Ed25519PublicKey(copy_u8_array(slice)))
    }
}

impl AsRef<Self> for Ed25519PublicKey {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<[u8]> for Ed25519PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

//======
// hash
//======

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Categorize, Encode, Decode, BasicDescribe)]
#[sbor(transparent)]
pub struct Ed25519PublicKeyHash(pub [u8; Self::LENGTH]);

impl Describe<ScryptoCustomTypeKind> for Ed25519PublicKeyHash {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::ED25519_PUBLIC_KEY_HASH_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::ed25519_public_key_hash_type_data()
    }
}

impl Ed25519PublicKeyHash {
    pub const LENGTH: usize = NodeId::RID_LENGTH;

    pub fn new_from_public_key(public_key: &Ed25519PublicKey) -> Self {
        Self(hash_public_key_bytes(public_key.0))
    }
}

impl HasPublicKeyHash for Ed25519PublicKey {
    type TypedPublicKeyHash = Ed25519PublicKeyHash;

    fn get_hash(&self) -> Self::TypedPublicKeyHash {
        Self::TypedPublicKeyHash::new_from_public_key(self)
    }
}

impl IsPublicKeyHash for Ed25519PublicKeyHash {
    fn get_hash_bytes(&self) -> &[u8; Self::LENGTH] {
        &self.0
    }

    fn into_enum(self) -> PublicKeyHash {
        PublicKeyHash::Ed25519(self)
    }
}

//======
// error
//======

/// Represents an error when parsing ED25519 public key from hex.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ParseEd25519PublicKeyError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseEd25519PublicKeyError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseEd25519PublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// text
//======

impl FromStr for Ed25519PublicKey {
    type Err = ParseEd25519PublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseEd25519PublicKeyError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Ed25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Ed25519PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
