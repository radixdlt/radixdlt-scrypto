use radix_engine_interface::{
    api::types::*,
    blueprints::resource::NonFungibleLocalId,
    crypto::{EcdsaSecp256k1PublicKey, EddsaEd25519PublicKey, PublicKey},
    data::model::Address,
    math::{Decimal, PreciseDecimal},
};
use transaction_data::model::{
    ManifestAddress, ManifestDecimal, ManifestNonFungibleLocalId, ManifestPreciseDecimal,
    ManifestPublicKey,
};

pub fn map_validated_address(address: ManifestAddress) -> Address {
    match address {
        ManifestAddress::Package(a) => {
            Address::Package(PackageAddress::try_from(a.as_slice()).unwrap())
        }
        ManifestAddress::Component(a) => {
            Address::Component(ComponentAddress::try_from(a.as_slice()).unwrap())
        }
        ManifestAddress::ResourceManager(a) => {
            Address::ResourceManager(ResourceAddress::try_from(a.as_slice()).unwrap())
        }
    }
}

pub fn map_validated_decimal(d: ManifestDecimal) -> Decimal {
    Decimal::try_from(d.0.as_slice()).unwrap()
}

pub fn map_validated_precise_decimal(d: ManifestPreciseDecimal) -> PreciseDecimal {
    PreciseDecimal::try_from(d.0.as_slice()).unwrap()
}

pub fn map_validated_non_fungible_local_id(id: ManifestNonFungibleLocalId) -> NonFungibleLocalId {
    match id {
        ManifestNonFungibleLocalId::String(i) => NonFungibleLocalId::string(i.clone()).unwrap(),
        ManifestNonFungibleLocalId::Integer(i) => NonFungibleLocalId::integer(i.clone()),
        ManifestNonFungibleLocalId::Bytes(i) => NonFungibleLocalId::bytes(i.clone()).unwrap(),
        ManifestNonFungibleLocalId::UUID(i) => NonFungibleLocalId::uuid(i.clone()).unwrap(),
    }
}

pub fn map_validated_public_key(pk: ManifestPublicKey) -> PublicKey {
    match pk {
        ManifestPublicKey::EcdsaSecp256k1(pk) => {
            PublicKey::EcdsaSecp256k1(EcdsaSecp256k1PublicKey(pk))
        }
        ManifestPublicKey::EddsaEd25519(pk) => PublicKey::EddsaEd25519(EddsaEd25519PublicKey(pk)),
    }
}
