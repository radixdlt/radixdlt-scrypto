use crate::abi::*;
use crate::api::types::*;
use crate::data::ScryptoCustomValueKind;
use crate::*;
#[cfg(not(feature = "alloc"))]
use sbor::rust::fmt;
use sbor::rust::fmt::Debug;
use utils::copy_u8_array;

// TODO: it's still up to debate whether this should be an enum OR dedicated types for each variant.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Own {
    Bucket(BucketId),
    Proof(ProofId),
    Vault(VaultId),
    Component(ComponentId),
    KeyValueStore(KeyValueStoreId),
}

impl Own {
    pub fn vault_id(&self) -> VaultId {
        match self {
            Own::Vault(v) => v.clone(),
            _ => panic!("Not a vault ownership"),
        }
    }
    pub fn bucket_id(&self) -> BucketId {
        match self {
            Own::Bucket(v) => v.clone(),
            _ => panic!("Not a bucket ownership"),
        }
    }
    pub fn proof_id(&self) -> ProofId {
        match self {
            Own::Proof(v) => v.clone(),
            _ => panic!("Not a proof ownership"),
        }
    }
}

impl From<Bucket> for Own {
    fn from(bucket: Bucket) -> Self {
        Own::Bucket(bucket.0)
    }
}

impl From<Proof> for Own {
    fn from(proof: Proof) -> Self {
        Own::Proof(proof.0)
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
        match self {
            Own::Bucket(v) => {
                encoder.write_byte(0)?;
                encoder.write_slice(&v.to_le_bytes())?;
            }
            Own::Proof(v) => {
                encoder.write_byte(1)?;
                encoder.write_slice(&v.to_le_bytes())?;
            }
            Own::Vault(v) => {
                encoder.write_byte(2)?;
                encoder.write_slice(v)?;
            }
            Own::Component(v) => {
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
}

impl<D: Decoder<ScryptoCustomValueKind>> Decode<ScryptoCustomValueKind, D> for Own {
    fn decode_body_with_value_kind(
        decoder: &mut D,
        value_kind: ValueKind<ScryptoCustomValueKind>,
    ) -> Result<Self, DecodeError> {
        decoder.check_preloaded_value_kind(value_kind, Self::value_kind())?;
        match decoder.read_byte()? {
            0 => Ok(Self::Bucket(u32::from_le_bytes(copy_u8_array(
                decoder.read_slice(4)?,
            )))),
            1 => Ok(Self::Proof(u32::from_le_bytes(copy_u8_array(
                decoder.read_slice(4)?,
            )))),
            2 => Ok(Self::Vault(copy_u8_array(decoder.read_slice(36)?))),
            3 => Ok(Self::Component(copy_u8_array(decoder.read_slice(36)?))),
            4 => Ok(Self::KeyValueStore(copy_u8_array(decoder.read_slice(36)?))),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }
}

impl scrypto_abi::LegacyDescribe for Own {
    fn describe() -> scrypto_abi::Type {
        Type::Own
    }
}
