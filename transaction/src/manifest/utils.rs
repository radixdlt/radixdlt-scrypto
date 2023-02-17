use radix_engine_interface::{
    api::types::*,
    blueprints::resource::NonFungibleLocalId,
    data::model::Address,
    math::{Decimal, PreciseDecimal},
};
use transaction_data::model::{
    ManifestAddress, ManifestDecimal, ManifestNonFungibleLocalId, ManifestPreciseDecimal,
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
