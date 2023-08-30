use super::*;
use crate::constants::RESOURCE_PACKAGE;
use sbor::rust::prelude::*;
use sbor::*;

fn unnamed_type_kind<L: SchemaTypeLink>(
    custom_type_kind: ScryptoCustomTypeKind,
    custom_type_validation: Option<ScryptoCustomTypeValidation>,
) -> TypeData<ScryptoCustomTypeKind, L> {
    TypeData {
        kind: TypeKind::Custom(custom_type_kind),
        metadata: TypeMetadata::unnamed(),
        validation: match custom_type_validation {
            Some(v) => TypeValidation::Custom(v),
            None => TypeValidation::None,
        },
    }
}

fn named_type_kind<L: SchemaTypeLink>(
    name: &'static str,
    custom_type_kind: ScryptoCustomTypeKind,
    custom_type_validation: Option<ScryptoCustomTypeValidation>,
) -> TypeData<ScryptoCustomTypeKind, L> {
    TypeData {
        kind: TypeKind::Custom(custom_type_kind),
        metadata: TypeMetadata::no_child_names(name),
        validation: match custom_type_validation {
            Some(v) => TypeValidation::Custom(v),
            None => TypeValidation::None,
        },
    }
}

create_well_known_lookup!(
    WELL_KNOWN_LOOKUP,
    well_known_scrypto_custom_types,
    ScryptoCustomTypeKind,
    [
        // References
        (
            REFERENCE,
            VALUE_KIND_REFERENCE,
            unnamed_type_kind(ScryptoCustomTypeKind::Reference, None)
        ),
        (
            GLOBAL_ADDRESS,
            VALUE_KIND_REFERENCE + 1,
            named_type_kind(
                "GlobalAddress",
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobal,
                ))
            )
        ),
        (
            INTERNAL_ADDRESS,
            VALUE_KIND_REFERENCE + 2,
            named_type_kind(
                "InternalAddress",
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsInternal,
                )),
            )
        ),
        (
            PACKAGE_ADDRESS,
            VALUE_KIND_REFERENCE + 3,
            named_type_kind(
                "PackageAddress",
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalPackage,
                )),
            )
        ),
        (
            COMPONENT_ADDRESS,
            VALUE_KIND_REFERENCE + 4,
            named_type_kind(
                "ComponentAddress",
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalComponent,
                )),
            )
        ),
        (
            RESOURCE_ADDRESS,
            VALUE_KIND_REFERENCE + 5,
            named_type_kind(
                "ResourceAddress",
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalResourceManager,
                )),
            )
        ),
        // Own
        (
            OWN,
            VALUE_KIND_OWN,
            unnamed_type_kind(ScryptoCustomTypeKind::Own, None)
        ),
        (
            OWN_BUCKET,
            VALUE_KIND_OWN + 1,
            named_type_kind(
                "Bucket",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(OwnValidation::IsBucket)),
            )
        ),
        (
            OWN_FUNGIBLE_BUCKET,
            VALUE_KIND_OWN + 2,
            named_type_kind(
                "FungibleBucket",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsTypedObject(
                        Some(RESOURCE_PACKAGE),
                        "FungibleBucket".to_string()
                    ),
                )),
            )
        ),
        (
            OWN_NON_FUNGIBLE_BUCKET,
            VALUE_KIND_OWN + 3,
            named_type_kind(
                "NonFungibleBucket",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsTypedObject(
                        Some(RESOURCE_PACKAGE),
                        "NonFungibleBucket".to_string(),
                    ),
                )),
            )
        ),
        (
            OWN_PROOF,
            VALUE_KIND_OWN + 4,
            named_type_kind(
                "Proof",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(OwnValidation::IsProof)),
            )
        ),
        (
            OWN_FUNGIBLE_PROOF,
            VALUE_KIND_OWN + 5,
            named_type_kind(
                "FungibleProof",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsTypedObject(
                        Some(RESOURCE_PACKAGE),
                        "FungibleProof".to_string()
                    ),
                )),
            )
        ),
        (
            OWN_NON_FUNGIBLE_PROOF,
            VALUE_KIND_OWN + 6,
            named_type_kind(
                "NonFungibleProof",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsTypedObject(
                        Some(RESOURCE_PACKAGE),
                        "NonFungibleProof".to_string(),
                    ),
                )),
            )
        ),
        (
            OWN_VAULT,
            VALUE_KIND_OWN + 7,
            named_type_kind(
                "Vault",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(OwnValidation::IsVault)),
            )
        ),
        (
            OWN_FUNGIBLE_VAULT,
            VALUE_KIND_OWN + 8,
            named_type_kind(
                "FungibleVault",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsTypedObject(
                        Some(RESOURCE_PACKAGE),
                        "FungibleVault".to_string()
                    ),
                )),
            )
        ),
        (
            OWN_NON_FUNGIBLE_VAULT,
            VALUE_KIND_OWN + 9,
            named_type_kind(
                "NonFungibleVault",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsTypedObject(
                        Some(RESOURCE_PACKAGE),
                        "NonFungibleVault".to_string(),
                    ),
                )),
            )
        ),
        (
            OWN_KEY_VALUE_STORE,
            VALUE_KIND_OWN + 10,
            named_type_kind(
                "KeyValueStore",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsKeyValueStore,
                )),
            )
        ),
        (
            OWN_GLOBAL_ADDRESS_RESERVATION,
            VALUE_KIND_OWN + 11,
            named_type_kind(
                "GlobalAddressReservation",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsGlobalAddressReservation,
                )),
            )
        ),
        // Others
        (
            DECIMAL,
            VALUE_KIND_DECIMAL,
            unnamed_type_kind(ScryptoCustomTypeKind::Decimal, None)
        ),
        (
            PRECISE_DECIMAL,
            VALUE_KIND_PRECISE_DECIMAL,
            unnamed_type_kind(ScryptoCustomTypeKind::PreciseDecimal, None)
        ),
        (
            NON_FUNGIBLE_LOCAL_ID,
            VALUE_KIND_NON_FUNGIBLE_LOCAL_ID,
            unnamed_type_kind(ScryptoCustomTypeKind::NonFungibleLocalId, None)
        ),
    ]
);

pub(crate) fn resolve_scrypto_well_known_type(
    well_known_index: WellKnownTypeIndex,
) -> Option<&'static TypeData<ScryptoCustomTypeKind, LocalTypeIndex>> {
    WELL_KNOWN_LOOKUP
        .get(well_known_index.as_index())
        .and_then(|x| x.as_ref())
}
