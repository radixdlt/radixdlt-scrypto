use super::*;
use sbor::*;

use well_known_scrypto_custom_types::*;

pub mod well_known_scrypto_custom_types {
    use super::*;

    pub const ADDRESS_ID: u8 = VALUE_KIND_ADDRESS;
    pub const PACKAGE_ADDRESS_ID: u8 = 0xf0;
    pub const COMPONENT_ADDRESS_ID: u8 = 0xf1;
    pub const RESOURCE_ADDRESS_ID: u8 = 0xf2;

    pub const OWN_ID: u8 = VALUE_KIND_OWN;
    // We skip KeyValueStore because it has generic parameters

    pub const DECIMAL_ID: u8 = VALUE_KIND_DECIMAL;
    pub const PRECISE_DECIMAL_ID: u8 = VALUE_KIND_PRECISE_DECIMAL;
    pub const NON_FUNGIBLE_LOCAL_ID_ID: u8 = VALUE_KIND_NON_FUNGIBLE_LOCAL_ID;
}

pub(crate) fn resolve_scrypto_custom_well_known_type(
    well_known_index: u8,
) -> Option<TypeData<ScryptoCustomTypeKind<LocalTypeIndex>, LocalTypeIndex>> {
    let (name, custom_type_kind) = match well_known_index {
        VALUE_KIND_ADDRESS => ("Address", ScryptoCustomTypeKind::Address),
        PACKAGE_ADDRESS_ID => ("PackageAddress", ScryptoCustomTypeKind::PackageAddress),
        COMPONENT_ADDRESS_ID => ("ComponentAddress", ScryptoCustomTypeKind::ComponentAddress),
        RESOURCE_ADDRESS_ID => ("ResourceAddress", ScryptoCustomTypeKind::ResourceAddress),

        OWN_ID => ("Own", ScryptoCustomTypeKind::Own),

        DECIMAL_ID => ("Decimal", ScryptoCustomTypeKind::Decimal),
        PRECISE_DECIMAL_ID => ("PreciseDecimal", ScryptoCustomTypeKind::PreciseDecimal),
        NON_FUNGIBLE_LOCAL_ID_ID => (
            "NonFungibleLocalId",
            ScryptoCustomTypeKind::NonFungibleLocalId,
        ),
        _ => return None,
    };

    Some(TypeData::named_no_child_names(
        name,
        TypeKind::Custom(custom_type_kind),
    ))
}
