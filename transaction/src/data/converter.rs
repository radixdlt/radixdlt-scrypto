use radix_engine_interface::data::manifest::model::{
    ManifestDecimal, ManifestNonFungibleLocalId, ManifestPreciseDecimal,
};
use radix_engine_interface::data::scrypto::model::*;
use radix_engine_interface::math::*;

/* Utils for conversion between "trusted" manifest value and rust value */

pub fn to_decimal(d: ManifestDecimal) -> Decimal {
    Decimal::try_from(d.0.as_slice()).unwrap()
}

pub fn to_precise_decimal(d: ManifestPreciseDecimal) -> PreciseDecimal {
    PreciseDecimal::try_from(d.0.as_slice()).unwrap()
}

pub fn to_non_fungible_local_id(id: ManifestNonFungibleLocalId) -> NonFungibleLocalId {
    match id {
        ManifestNonFungibleLocalId::String(i) => NonFungibleLocalId::string(i).unwrap(),
        ManifestNonFungibleLocalId::Integer(i) => NonFungibleLocalId::integer(i),
        ManifestNonFungibleLocalId::Bytes(i) => NonFungibleLocalId::bytes(i).unwrap(),
        ManifestNonFungibleLocalId::UUID(i) => NonFungibleLocalId::uuid(i).unwrap(),
    }
}

pub fn from_decimal(d: Decimal) -> ManifestDecimal {
    ManifestDecimal(d.to_vec().try_into().unwrap())
}

pub fn from_precise_decimal(d: PreciseDecimal) -> ManifestPreciseDecimal {
    ManifestPreciseDecimal(d.to_vec().try_into().unwrap())
}

pub fn from_non_fungible_local_id(id: NonFungibleLocalId) -> ManifestNonFungibleLocalId {
    match id {
        NonFungibleLocalId::String(i) => {
            ManifestNonFungibleLocalId::string(i.value().to_owned()).unwrap()
        }
        NonFungibleLocalId::Integer(i) => ManifestNonFungibleLocalId::integer(i.value()).unwrap(),
        NonFungibleLocalId::Bytes(i) => {
            ManifestNonFungibleLocalId::bytes(i.value().to_owned()).unwrap()
        }
        NonFungibleLocalId::UUID(i) => ManifestNonFungibleLocalId::uuid(i.value()).unwrap(),
    }
}
