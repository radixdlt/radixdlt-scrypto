use sbor::type_id::*;
use sbor::*;

use crate::component::*;
use crate::core::*;
use crate::crypto::*;
use crate::math::*;
use crate::resource::*;

use super::ScryptoCustomTypeId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomValue {
    // Global address types
    PackageAddress(PackageAddress),
    ComponentAddress(ComponentAddress),
    ResourceAddress(ResourceAddress),
    SystemAddress(SystemAddress),

    // RE nodes types
    Component(Component),
    KeyValueStore(KeyValueStore<(), ()>),
    Bucket(Bucket),
    Proof(Proof),
    Vault(Vault),

    // Other interpreted types
    Expression(Expression),
    Blob(Blob),
    NonFungibleAddress(NonFungibleAddress), // for resource address contained

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

impl CustomValue<ScryptoCustomTypeId> for ScryptoCustomValue {
    fn encode_type_id(&self, encoder: &mut Encoder<ScryptoCustomTypeId>) {
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
            ScryptoCustomValue::Vault(_) => {
                encoder.write_type_id(SborTypeId::Custom(ScryptoCustomTypeId::Vault))
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

    fn encode_value(&self, encoder: &mut Encoder<ScryptoCustomTypeId>) {
        match self {
            // TODO: vector free
            ScryptoCustomValue::PackageAddress(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::ComponentAddress(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::ResourceAddress(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::SystemAddress(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Component(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::KeyValueStore(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Bucket(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Proof(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Vault(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Expression(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Blob(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::NonFungibleAddress(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Hash(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::EcdsaSecp256k1PublicKey(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::EcdsaSecp256k1Signature(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::EddsaEd25519PublicKey(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::EddsaEd25519Signature(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::Decimal(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::PreciseDecimal(v) => encoder.write_slice(&v.to_vec()),
            ScryptoCustomValue::NonFungibleId(v) => encoder.write_slice(&v.to_vec()),
        }
    }

    fn decode(
        decoder: &mut Decoder<ScryptoCustomTypeId>,
        type_id: ScryptoCustomTypeId,
    ) -> Result<Self, DecodeError> {
        match type_id {
            ScryptoCustomTypeId::PackageAddress => {
                let n = 27;
                let slice = decoder.read_slice(n)?;
                PackageAddress::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::ComponentAddress => {
                let n = 27;
                let slice = decoder.read_slice(n)?;
                ComponentAddress::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::ResourceAddress => {
                let n = 27;
                let slice = decoder.read_slice(n)?;
                ResourceAddress::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::SystemAddress => {
                let n = 27;
                let slice = decoder.read_slice(n)?;
                SystemAddress::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::Component => {
                let n = 36;
                let slice = decoder.read_slice(n)?;
                Component::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::KeyValueStore => {
                let n = 36;
                let slice = decoder.read_slice(n)?;
                KeyValueStore::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::Bucket => {
                let n = 4;
                let slice = decoder.read_slice(n)?;
                Bucket::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::Proof => {
                let n = 4;
                let slice = decoder.read_slice(n)?;
                Proof::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::Vault => {
                let n = 36;
                let slice = decoder.read_slice(n)?;
                Vault::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::Expression => {
                let n = decoder.read_size()?;
                let slice = decoder.read_slice(n)?;
                Expression::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::Blob => {
                let n = 32;
                let slice = decoder.read_slice(n)?;
                Blob::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::NonFungibleAddress => {
                let n = decoder.read_size();
                let slice = decoder.read_slice(n)?;
                NonFungibleAddress::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::Hash => {
                let n = 32;
                let slice = decoder.read_slice(n)?;
                Hash::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::EcdsaSecp256k1PublicKey => {
                let n = EcdsaSecp256k1PublicKey::LENGTH;
                let slice = decoder.read_slice(n)?;
                EcdsaSecp256k1PublicKey::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::EcdsaSecp256k1Signature => {
                let n = EcdsaSecp256k1Signature::LENGTH;
                let slice = decoder.read_slice(n)?;
                EcdsaSecp256k1Signature::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::EddsaEd25519PublicKey => {
                let n = EddsaEd25519PublicKey::LENGTH;
                let slice = decoder.read_slice(n)?;
                EddsaEd25519PublicKey::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::EddsaEd25519Signature => {
                let n = EddsaEd25519Signature::LENGTH;
                let slice = decoder.read_slice(n)?;
                EddsaEd25519Signature::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::Decimal => {
                let n = Decimal::BITS / 8;
                let slice = decoder.read_slice(n)?;
                Decimal::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::PreciseDecimal => {
                let n = PreciseDecimal::BITS / 8;
                let slice = decoder.read_slice(n)?;
                PreciseDecimal::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
            ScryptoCustomTypeId::NonFungibleId => {
                let n = decoder.read_size();
                let slice = decoder.read_slice(n)?;
                NonFungibleId::try_from(slice).map_err(DecodeError::InvalidCustomValue)
            }
        }
    }
}
