use sbor::type_id::*;
use sbor::*;

use crate::api::types::*;
use crate::crypto::*;
use crate::data::types::*;
use crate::data::*;
use crate::math::{Decimal, PreciseDecimal};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomValue {
    // RE global address types
    PackageAddress(PackageAddress),
    ComponentAddress(ComponentAddress),
    ResourceAddress(ResourceAddress),
    SystemAddress(SystemAddress),

    // RE interpreted types
    Own(Own),
    Component(ComponentId),
    KeyValueStore(KeyValueStoreId),
    NonFungibleAddress(NonFungibleAddress),
    Blob(Blob),

    // TX interpreted types
    Bucket(ManifestBucket),
    Proof(ManifestProof),
    Expression(ManifestExpression),

    // Uninterpreted
    Hash(Hash),
    EcdsaSecp256k1PublicKey(EcdsaSecp256k1PublicKey),
    EcdsaSecp256k1Signature(EcdsaSecp256k1Signature),
    EddsaEd25519PublicKey(EddsaEd25519PublicKey),
    EddsaEd25519Signature(EddsaEd25519Signature),
    Decimal(Decimal),
    PreciseDecimal(PreciseDecimal),
    NonFungibleId(NonFungibleId),
}

impl<E: Encoder<ScryptoCustomTypeId>> Encode<ScryptoCustomTypeId, E> for ScryptoCustomValue {
    fn encode_type_id(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            ScryptoCustomValue::PackageAddress(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::PackageAddress))
            }
            ScryptoCustomValue::ComponentAddress(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::ComponentAddress))
            }
            ScryptoCustomValue::ResourceAddress(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::ResourceAddress))
            }
            ScryptoCustomValue::SystemAddress(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::SystemAddress))
            }
            ScryptoCustomValue::Own(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Own))
            }
            ScryptoCustomValue::Component(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Component))
            }
            ScryptoCustomValue::KeyValueStore(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::KeyValueStore))
            }
            ScryptoCustomValue::Bucket(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Bucket))
            }
            ScryptoCustomValue::Proof(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Proof))
            }
            ScryptoCustomValue::Expression(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Expression))
            }
            ScryptoCustomValue::Blob(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Blob))
            }
            ScryptoCustomValue::NonFungibleAddress(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::NonFungibleAddress))
            }
            ScryptoCustomValue::Hash(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Hash))
            }
            ScryptoCustomValue::EcdsaSecp256k1PublicKey(_) => encoder.write_type_id(
                SborTypeId::Custom(ScryptoCustomTypeId::EcdsaSecp256k1PublicKey),
            ),
            ScryptoCustomValue::EcdsaSecp256k1Signature(_) => encoder.write_type_id(
                SborTypeId::Custom(ScryptoCustomTypeId::EcdsaSecp256k1Signature),
            ),
            ScryptoCustomValue::EddsaEd25519PublicKey(_) => encoder.write_type_id(
                SborTypeId::Custom(ScryptoCustomTypeId::EddsaEd25519PublicKey),
            ),
            ScryptoCustomValue::EddsaEd25519Signature(_) => encoder.write_type_id(
                SborTypeId::Custom(ScryptoCustomTypeId::EddsaEd25519Signature),
            ),
            ScryptoCustomValue::Decimal(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Decimal))
            }
            ScryptoCustomValue::PreciseDecimal(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::PreciseDecimal))
            }
            ScryptoCustomValue::NonFungibleId(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::NonFungibleId))
            }
        }
    }

    fn encode_body(&self, encoder: &mut E) -> Result<(), EncodeError> {
        match self {
            // TODO: vector free
            ScryptoCustomValue::PackageAddress(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::ComponentAddress(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::ResourceAddress(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::SystemAddress(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Own(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Component(v) => encoder.write_slice(v.as_slice()),
            ScryptoCustomValue::KeyValueStore(v) => encoder.write_slice(v.as_slice()),
            ScryptoCustomValue::Bucket(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Proof(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Expression(v) => {
                let buf = v.to_vec();
                encoder.write_size(buf.len())?;
                encoder.write_slice(&buf)
            }
            ScryptoCustomValue::Blob(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::NonFungibleAddress(v) => {
                let buf = v.to_vec();
                encoder.write_size(buf.len())?;
                encoder.write_slice(&buf)
            }
            ScryptoCustomValue::Hash(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::EcdsaSecp256k1PublicKey(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::EcdsaSecp256k1Signature(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::EddsaEd25519PublicKey(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::EddsaEd25519Signature(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Decimal(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::PreciseDecimal(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::NonFungibleId(v) => {
                let buf = v.to_vec();
                encoder.write_size(buf.len())?;
                encoder.write_slice(&buf)
            }
        }
    }
}

impl<D: Decoder<ScryptoCustomTypeId>> Decode<ScryptoCustomTypeId, D> for ScryptoCustomValue {
    fn decode_body_with_type_id(
        decoder: &mut D,
        type_id: SborTypeId<ScryptoCustomTypeId>,
    ) -> Result<Self, DecodeError> {
        let SborTypeId::Custom(type_id) = type_id else {
            return Err(DecodeError::UnexpectedCustomTypeId { actual: type_id.as_u8() });
        };
        match type_id {
            ScryptoCustomTypeId::PackageAddress => {
                let n = 27;
                let slice = decoder.read_slice(n)?;
                PackageAddress::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::PackageAddress)
            }
            ScryptoCustomTypeId::ComponentAddress => {
                let n = 27;
                let slice = decoder.read_slice(n)?;
                ComponentAddress::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::ComponentAddress)
            }
            ScryptoCustomTypeId::ResourceAddress => {
                let n = 27;
                let slice = decoder.read_slice(n)?;
                ResourceAddress::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::ResourceAddress)
            }
            ScryptoCustomTypeId::SystemAddress => {
                let n = 27;
                let slice = decoder.read_slice(n)?;
                SystemAddress::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::SystemAddress)
            }
            ScryptoCustomTypeId::Own => {
                let n = 36;
                let slice = decoder.read_slice(n)?;
                Ok(Self::Own(
                    slice
                        .try_into()
                        .map_err(|_| DecodeError::InvalidCustomValue)?,
                ))
            }
            ScryptoCustomTypeId::Component => {
                let n = 36;
                let slice = decoder.read_slice(n)?;
                Ok(Self::Component(
                    slice
                        .try_into()
                        .map_err(|_| DecodeError::InvalidCustomValue)?,
                ))
            }
            ScryptoCustomTypeId::KeyValueStore => {
                let n = 36;
                let slice = decoder.read_slice(n)?;
                Ok(Self::KeyValueStore(
                    slice
                        .try_into()
                        .map_err(|_| DecodeError::InvalidCustomValue)?,
                ))
            }
            ScryptoCustomTypeId::Bucket => {
                let n = 4;
                let slice = decoder.read_slice(n)?;
                Ok(Self::Bucket(
                    slice
                        .try_into()
                        .map_err(|_| DecodeError::InvalidCustomValue)?,
                ))
            }
            ScryptoCustomTypeId::Proof => {
                let n = 4;
                let slice = decoder.read_slice(n)?;
                Ok(Self::Proof(
                    slice
                        .try_into()
                        .map_err(|_| DecodeError::InvalidCustomValue)?,
                ))
            }
            ScryptoCustomTypeId::Expression => {
                let n = 1;
                let slice = decoder.read_slice(n)?;
                Ok(Self::Expression(
                    slice
                        .try_into()
                        .map_err(|_| DecodeError::InvalidCustomValue)?,
                ))
            }
            ScryptoCustomTypeId::Blob => {
                let n = 32;
                let slice = decoder.read_slice(n)?;
                Blob::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::Blob)
            }
            ScryptoCustomTypeId::NonFungibleAddress => {
                let n = decoder.read_size()?;
                let slice = decoder.read_slice(n)?;
                NonFungibleAddress::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::NonFungibleAddress)
            }
            ScryptoCustomTypeId::Hash => {
                let n = 32;
                let slice = decoder.read_slice(n)?;
                Hash::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::Hash)
            }
            ScryptoCustomTypeId::EcdsaSecp256k1PublicKey => {
                let n = EcdsaSecp256k1PublicKey::LENGTH;
                let slice = decoder.read_slice(n)?;
                EcdsaSecp256k1PublicKey::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::EcdsaSecp256k1PublicKey)
            }
            ScryptoCustomTypeId::EcdsaSecp256k1Signature => {
                let n = EcdsaSecp256k1Signature::LENGTH;
                let slice = decoder.read_slice(n)?;
                EcdsaSecp256k1Signature::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::EcdsaSecp256k1Signature)
            }
            ScryptoCustomTypeId::EddsaEd25519PublicKey => {
                let n = EddsaEd25519PublicKey::LENGTH;
                let slice = decoder.read_slice(n)?;
                EddsaEd25519PublicKey::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::EddsaEd25519PublicKey)
            }
            ScryptoCustomTypeId::EddsaEd25519Signature => {
                let n = EddsaEd25519Signature::LENGTH;
                let slice = decoder.read_slice(n)?;
                EddsaEd25519Signature::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::EddsaEd25519Signature)
            }
            ScryptoCustomTypeId::Decimal => {
                let n = Decimal::BITS / 8;
                let slice = decoder.read_slice(n)?;
                Decimal::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::Decimal)
            }
            ScryptoCustomTypeId::PreciseDecimal => {
                let n = PreciseDecimal::BITS / 8;
                let slice = decoder.read_slice(n)?;
                PreciseDecimal::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::PreciseDecimal)
            }
            ScryptoCustomTypeId::NonFungibleId => {
                let n = decoder.read_size()?;
                let slice = decoder.read_slice(n)?;
                NonFungibleId::try_from(slice)
                    .map_err(|_| DecodeError::InvalidCustomValue)
                    .map(Self::NonFungibleId)
            }
        }
    }
}
