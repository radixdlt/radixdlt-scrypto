use super::*;
use sbor::*;

use well_known_scrypto_custom_types::*;

pub mod well_known_scrypto_custom_types {
    use super::*;

    pub const REFERENCE_ID: u8 = VALUE_KIND_REFERENCE;
    pub const GLOBAL_ADDRESS_ID: u8 = VALUE_KIND_REFERENCE + 1;
    pub const INTERNAL_ADDRESS_ID: u8 = VALUE_KIND_REFERENCE + 2;
    pub const PACKAGE_ADDRESS_ID: u8 = VALUE_KIND_REFERENCE + 3;
    pub const COMPONENT_ADDRESS_ID: u8 = VALUE_KIND_REFERENCE + 4;
    pub const RESOURCE_ADDRESS_ID: u8 = VALUE_KIND_REFERENCE + 5;

    pub const OWN_ID: u8 = VALUE_KIND_OWN;
    pub const OWN_BUCKET_ID: u8 = VALUE_KIND_OWN + 1;
    pub const OWN_PROOF_ID: u8 = VALUE_KIND_OWN + 2;
    pub const OWN_VAULT_ID: u8 = VALUE_KIND_OWN + 3;
    pub const OWN_KEY_VALUE_STORE_ID: u8 = VALUE_KIND_OWN + 4;

    pub const DECIMAL_ID: u8 = VALUE_KIND_DECIMAL;
    pub const PRECISE_DECIMAL_ID: u8 = VALUE_KIND_PRECISE_DECIMAL;
    pub const NON_FUNGIBLE_LOCAL_ID_ID: u8 = VALUE_KIND_NON_FUNGIBLE_LOCAL_ID;
}

fn unnamed_type_kind(
    custom_type_kind: ScryptoCustomTypeKind,
    custom_type_validation: Option<ScryptoCustomTypeValidation>,
) -> TypeData<ScryptoCustomTypeKind, LocalTypeIndex> {
    TypeData {
        kind: TypeKind::Custom(custom_type_kind),
        metadata: TypeMetadata::unnamed(),
        validation: match custom_type_validation {
            Some(v) => TypeValidation::Custom(v),
            None => TypeValidation::None,
        },
    }
}

create_well_known_lookup!(
    WELL_KNOWN_LOOKUP,
    ScryptoCustomTypeKind,
    [
        // Addresses
        (
            REFERENCE_ID,
            unnamed_type_kind(ScryptoCustomTypeKind::Reference, None)
        ),
        (
            GLOBAL_ADDRESS_ID,
            unnamed_type_kind(
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobal
                ))
            )
        ),
        (
            INTERNAL_ADDRESS_ID,
            unnamed_type_kind(
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsInternal
                ))
            )
        ),
        (
            PACKAGE_ADDRESS_ID,
            unnamed_type_kind(
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalPackage
                ))
            )
        ),
        (
            COMPONENT_ADDRESS_ID,
            unnamed_type_kind(
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalComponent
                ))
            )
        ),
        (
            RESOURCE_ADDRESS_ID,
            unnamed_type_kind(
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalResource
                ))
            )
        ),
        // Owned entities
        (OWN_ID, unnamed_type_kind(ScryptoCustomTypeKind::Own, None)),
        (
            OWN_BUCKET_ID,
            unnamed_type_kind(
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(OwnValidation::IsBucket))
            )
        ),
        (
            OWN_PROOF_ID,
            unnamed_type_kind(
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(OwnValidation::IsProof))
            )
        ),
        (
            OWN_VAULT_ID,
            unnamed_type_kind(
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(OwnValidation::IsVault))
            )
        ),
        (
            OWN_KEY_VALUE_STORE_ID,
            unnamed_type_kind(
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsKeyValueStore
                ))
            )
        ),
        // Others
        (
            DECIMAL_ID,
            unnamed_type_kind(ScryptoCustomTypeKind::Decimal, None)
        ),
        (
            PRECISE_DECIMAL_ID,
            unnamed_type_kind(ScryptoCustomTypeKind::PreciseDecimal, None)
        ),
        (
            NON_FUNGIBLE_LOCAL_ID_ID,
            unnamed_type_kind(ScryptoCustomTypeKind::NonFungibleLocalId, None)
        ),
    ]
);

pub(crate) fn resolve_scrypto_well_known_type(
    well_known_index: u8,
) -> Option<&'static TypeData<ScryptoCustomTypeKind, LocalTypeIndex>> {
    // We know that WELL_KNOWN_LOOKUP has 255 elements, so can use `get_unchecked` for fast look-ups
    unsafe {
        WELL_KNOWN_LOOKUP
            .get_unchecked(well_known_index as usize)
            .as_ref()
    }
}
