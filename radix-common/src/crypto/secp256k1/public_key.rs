use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;

/// Represents an uncompressed ECDSA Secp256k1 public key.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Sbor)]
#[sbor(transparent)]
pub struct Secp256k1UncompressedPublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl Secp256k1UncompressedPublicKey {
    pub const LENGTH: usize = 65;
}

/// Represents a compressed ECDSA Secp256k1 public key, which is the default format used in the Radix stack.
#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(
    Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Categorize, Encode, Decode, BasicDescribe,
)]
#[sbor(transparent)]
pub struct Secp256k1PublicKey(
    #[cfg_attr(feature = "serde", serde(with = "hex::serde"))] pub [u8; Self::LENGTH],
);

impl Describe<ScryptoCustomTypeKind> for Secp256k1PublicKey {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::SECP256K1_PUBLIC_KEY_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::secp256k1_public_key_type_data()
    }
}

impl Secp256k1PublicKey {
    pub const LENGTH: usize = 33;

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn to_hash(&self) -> Secp256k1PublicKeyHash {
        Secp256k1PublicKeyHash::new_from_public_key(self)
    }
}

impl TryFrom<&[u8]> for Secp256k1PublicKey {
    type Error = ParseSecp256k1PublicKeyError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() != Secp256k1PublicKey::LENGTH {
            return Err(ParseSecp256k1PublicKeyError::InvalidLength(slice.len()));
        }

        Ok(Secp256k1PublicKey(copy_u8_array(slice)))
    }
}

impl AsRef<Self> for Secp256k1PublicKey {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<[u8]> for Secp256k1PublicKey {
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
pub struct Secp256k1PublicKeyHash(pub [u8; Self::LENGTH]);

impl Describe<ScryptoCustomTypeKind> for Secp256k1PublicKeyHash {
    const TYPE_ID: RustTypeId =
        RustTypeId::WellKnown(well_known_scrypto_custom_types::SECP256K1_PUBLIC_KEY_HASH_TYPE);

    fn type_data() -> ScryptoTypeData<RustTypeId> {
        well_known_scrypto_custom_types::secp256k1_public_key_hash_type_data()
    }
}

impl Secp256k1PublicKeyHash {
    pub const LENGTH: usize = NodeId::RID_LENGTH;

    pub fn new_from_public_key(public_key: &Secp256k1PublicKey) -> Self {
        Self(hash_public_key_bytes(public_key.0))
    }
}

impl HasPublicKeyHash for Secp256k1PublicKey {
    type TypedPublicKeyHash = Secp256k1PublicKeyHash;

    fn get_hash(&self) -> Self::TypedPublicKeyHash {
        Self::TypedPublicKeyHash::new_from_public_key(self)
    }
}

impl IsPublicKeyHash for Secp256k1PublicKeyHash {
    fn get_hash_bytes(&self) -> &[u8; Self::LENGTH] {
        &self.0
    }

    fn into_enum(self) -> PublicKeyHash {
        PublicKeyHash::Secp256k1(self)
    }
}

//======
// error
//======

/// Represents an error when parsing ED25519 public key from hex.
#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub enum ParseSecp256k1PublicKeyError {
    InvalidHex(String),
    InvalidLength(usize),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseSecp256k1PublicKeyError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseSecp256k1PublicKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//======
// text
//======

impl FromStr for Secp256k1PublicKey {
    type Err = ParseSecp256k1PublicKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bytes =
            hex::decode(s).map_err(|_| ParseSecp256k1PublicKeyError::InvalidHex(s.to_owned()))?;
        Self::try_from(bytes.as_slice())
    }
}

impl fmt::Display for Secp256k1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", hex::encode(self.to_vec()))
    }
}

impl fmt::Debug for Secp256k1PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{}", self)
    }
}
