use crate::data::scrypto::ScryptoCustomTypeKind;
use crate::data::scrypto::{ScryptoCustomValueKind, ScryptoEncoder};
use crate::*;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::prelude::*;
use sbor::*;
use utils::copy_u8_array;

// TODO: Unify with Address
pub const ENTITY_BYTES_LENGTH: usize = 1;
pub const OBJECT_HASH_LENGTH: usize = 26;
pub const OBJECT_HASH_END: usize = ENTITY_BYTES_LENGTH + OBJECT_HASH_LENGTH;
pub const OBJECT_INDEX_LENGTH: usize = 4;
pub const OBJECT_ID_LENGTH: usize = ENTITY_BYTES_LENGTH + OBJECT_HASH_LENGTH + OBJECT_INDEX_LENGTH;

// TODO: it's still up to debate whether this should be an enum OR dedicated types for each variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Own {
    Bucket([u8; OBJECT_ID_LENGTH]),
    Proof([u8; OBJECT_ID_LENGTH]),
    Vault([u8; OBJECT_ID_LENGTH]),
    Object([u8; OBJECT_ID_LENGTH]),
    KeyValueStore([u8; OBJECT_ID_LENGTH]),
}

impl Own {
    pub fn id(&self) -> [u8; OBJECT_ID_LENGTH] {
        match self {
            Own::Bucket(v)
            | Own::Object(v)
            | Own::Proof(v)
            | Own::Vault(v)
            | Own::KeyValueStore(v) => *v,
        }
    }
    pub fn vault_id(&self) -> [u8; OBJECT_ID_LENGTH] {
        match self {
            Own::Vault(v) => *v,
            _ => panic!("Not a vault ownership"),
        }
    }
    pub fn key_value_store_id(&self) -> [u8; OBJECT_ID_LENGTH] {
        match self {
            Own::KeyValueStore(v) => *v,
            _ => panic!("Not a kv-store ownership"),
        }
    }
    pub fn bucket_id(&self) -> [u8; OBJECT_ID_LENGTH] {
        match self {
            Own::Bucket(v) => *v,
            _ => panic!("Not a bucket ownership"),
        }
    }
    pub fn proof_id(&self) -> [u8; OBJECT_ID_LENGTH] {
        match self {
            Own::Proof(v) => *v,
            _ => panic!("Not a proof ownership"),
        }
    }
    pub fn kv_store_id(&self) -> [u8; OBJECT_ID_LENGTH] {
        match self {
            Own::KeyValueStore(v) => v.clone(),
            _ => panic!("Not a key-value store ownership"),
        }
    }
}

impl Own {
    pub fn encode_body_common<X: CustomValueKind, E: Encoder<X>>(
        &self,
        encoder: &mut E,
    ) -> Result<(), EncodeError> {
        match self {
            Own::Bucket(v) => {
                encoder.write_byte(0)?;
                encoder.write_slice(v)?;
            }
            Own::Proof(v) => {
                encoder.write_byte(1)?;
                encoder.write_slice(v)?;
            }
            Own::Vault(v) => {
                encoder.write_byte(2)?;
                encoder.write_slice(v)?;
            }
            Own::Object(v) => {
                encoder.write_byte(3)?;
                encoder.write_slice(v)?;
            }
            Own::KeyValueStore(v) => {
                encoder.write_byte(4)?;
                encoder.write_slice(v)?;
            }
        }
        Ok(())
    }

    pub fn decode_body_common<X: CustomValueKind, D: Decoder<X>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        match decoder.read_byte()? {
            0 => Ok(Self::Bucket(copy_u8_array(
                decoder.read_slice(OBJECT_ID_LENGTH)?,
            ))),
            1 => Ok(Self::Proof(copy_u8_array(
                decoder.read_slice(OBJECT_ID_LENGTH)?,
            ))),
            2 => Ok(Self::Vault(copy_u8_array(
                decoder.read_slice(OBJECT_ID_LENGTH)?,
            ))),
            3 => Ok(Self::Object(copy_u8_array(
                decoder.read_slice(OBJECT_ID_LENGTH)?,
            ))),
            4 => Ok(Self::KeyValueStore(copy_u8_array(
                decoder.read_slice(OBJECT_ID_LENGTH)?,
            ))),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut encoder = ScryptoEncoder::new(&mut buffer, 1);
        self.encode_body_common(&mut encoder).unwrap();
        buffer
    }
}

//========
// error
//========

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseOwnError {
    InvalidLength(usize),
    UnknownVariant(u8),
}

#[cfg(not(feature = "alloc"))]
impl std::error::Error for ParseOwnError {}

#[cfg(not(feature = "alloc"))]
impl fmt::Display for ParseOwnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

//========
// binary
//========

impl Categorize<ScryptoCustomValueKind> for Own {
    #[inline]
    fn value_kind() -> ValueKind<ScryptoCustomValueKind> {
        ValueKind::Custom(ScryptoCustomValueKind::Own)
    }
}

impl<E: Encoder<ScryptoCustomValueKind>> Encode<ScryptoCustomValueKind, E> for Own {
    #[inline]
    fn encode_value_kind(&self, encoder: &mut E) -> Result<(), EncodeError> {
        encoder.write_value_kind(Self::value_kind())
    }

    #[inline]
    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        self.encode_body_common(encoder)
    }
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Own {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        Self::decode_body_common(decoder)
    }
}

impl Describe<ScryptoCustomTypeKind> for Own {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::well_known(crate::data::scrypto::well_known_scrypto_custom_types::OWN_ID);
}
