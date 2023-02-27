use crate::abi::*;
use crate::api::types::*;
use crate::blueprints::resource::*;
use crate::data::ScryptoCustomTypeKind;
use crate::data::{ScryptoCustomValueKind, ScryptoEncoder};
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
    Account(AccountId), // TODO: Clean this out but required for now to be able to convert to the typed RENodeId
    KeyValueStore(KeyValueStoreId),
}

impl Own {
    pub fn component_id(&self) -> ComponentId {
        match self {
            Own::Component(v) => *v,
            _ => panic!("Not a component ownership"),
        }
    }
    pub fn vault_id(&self) -> VaultId {
        match self {
            Own::Vault(v) => *v,
            _ => panic!("Not a vault ownership"),
        }
    }
    pub fn key_value_store_id(&self) -> VaultId {
        match self {
            Own::KeyValueStore(v) => *v,
            _ => panic!("Not a kv-store ownership"),
        }
    }
    pub fn bucket_id(&self) -> BucketId {
        match self {
            Own::Bucket(v) => *v,
            _ => panic!("Not a bucket ownership"),
        }
    }
    pub fn proof_id(&self) -> ProofId {
        match self {
            Own::Proof(v) => *v,
            _ => panic!("Not a proof ownership"),
        }
    }
    pub fn kv_store_id(&self) -> KeyValueStoreId {
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
            Own::Component(v) => {
                encoder.write_byte(3)?;
                encoder.write_slice(v)?;
            }
            Own::KeyValueStore(v) => {
                encoder.write_byte(4)?;
                encoder.write_slice(v)?;
            }
            Own::Account(v) => {
                encoder.write_byte(5)?;
                encoder.write_slice(v)?;
            }
        }
        Ok(())
    }

    pub fn decode_body_common<X: CustomValueKind, D: Decoder<X>>(
        decoder: &mut D,
    ) -> Result<Self, DecodeError> {
        match decoder.read_byte()? {
            0 => Ok(Self::Bucket(copy_u8_array(decoder.read_slice(36)?))),
            1 => Ok(Self::Proof(copy_u8_array(decoder.read_slice(36)?))),
            2 => Ok(Self::Vault(copy_u8_array(decoder.read_slice(36)?))),
            3 => Ok(Self::Component(copy_u8_array(decoder.read_slice(36)?))),
            4 => Ok(Self::KeyValueStore(copy_u8_array(decoder.read_slice(36)?))),
            5 => Ok(Self::Account(copy_u8_array(decoder.read_slice(36)?))),
            _ => Err(DecodeError::InvalidCustomValue),
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut buffer = Vec::new();
        let mut encoder = ScryptoEncoder::new(&mut buffer);
        self.encode_body_common(&mut encoder).unwrap();
        buffer
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

impl scrypto_abi::LegacyDescribe for Own {
    fn describe() -> scrypto_abi::Type {
        Type::Own
    }
}

impl Describe<ScryptoCustomTypeKind<GlobalTypeId>> for Own {
    const TYPE_ID: GlobalTypeId =
        GlobalTypeId::well_known(crate::data::well_known_scrypto_custom_types::OWN_ID);
}
