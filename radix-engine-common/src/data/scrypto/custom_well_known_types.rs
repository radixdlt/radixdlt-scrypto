use super::*;
use crate::native_addresses::RESOURCE_PACKAGE;
use sbor::rust::prelude::*;
use sbor::*;
use well_known_scrypto_custom_types::*;

pub mod well_known_scrypto_custom_types {
    use super::*;

    pub const REFERENCE_ID: u8 = VALUE_KIND_REFERENCE;
    pub fn reference_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        unnamed_type_kind(ScryptoCustomTypeKind::Reference, None)
    }
    pub const GLOBAL_ADDRESS_ID: u8 = VALUE_KIND_REFERENCE + 1;
    pub fn global_address_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        // Record the type name, despite information available from node id.
        named_type_kind(
            "GlobalAddress",
            ScryptoCustomTypeKind::Reference,
            Some(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsGlobal,
            )),
        )
    }
    pub const INTERNAL_ADDRESS_ID: u8 = VALUE_KIND_REFERENCE + 2;
    pub fn internal_address_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        // Record the type name, despite information available from node id.
        named_type_kind(
            "InternalAddress",
            ScryptoCustomTypeKind::Reference,
            Some(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsInternal,
            )),
        )
    }
    pub const PACKAGE_ADDRESS_ID: u8 = VALUE_KIND_REFERENCE + 3;
    pub fn package_address_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        // Record the type name, despite information available from node id.
        named_type_kind(
            "PackageAddress",
            ScryptoCustomTypeKind::Reference,
            Some(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsGlobalPackage,
            )),
        )
    }
    pub const COMPONENT_ADDRESS_ID: u8 = VALUE_KIND_REFERENCE + 4;
    pub fn component_address_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        // Record the type name, despite information available from node id.
        named_type_kind(
            "ComponentAddress",
            ScryptoCustomTypeKind::Reference,
            Some(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsGlobalComponent,
            )),
        )
    }
    pub const RESOURCE_ADDRESS_ID: u8 = VALUE_KIND_REFERENCE + 5;
    pub fn resource_address_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        // Record the type name, despite information available from node id.
        named_type_kind(
            "ResourceAddress",
            ScryptoCustomTypeKind::Reference,
            Some(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsGlobalResourceManager,
            )),
        )
    }

    pub const OWN_ID: u8 = VALUE_KIND_OWN;
    pub fn own_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        unnamed_type_kind(ScryptoCustomTypeKind::Own, None)
    }
    pub const OWN_BUCKET_ID: u8 = VALUE_KIND_OWN + 1;
    pub fn own_bucket_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        // Bucket is not clear from the address in the value, so add it as a type name
        named_type_kind(
            "Bucket",
            ScryptoCustomTypeKind::Own,
            Some(ScryptoCustomTypeValidation::Own(OwnValidation::IsBucket)),
        )
    }
    pub const OWN_FUNGIBLE_BUCKET_ID: u8 = VALUE_KIND_OWN + 2;
    pub fn own_fungible_bucket_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L>
    {
        // Bucket is not clear from the address in the value, so add it as a type name
        named_type_kind(
            "FungibleBucket",
            ScryptoCustomTypeKind::Own,
            Some(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(Some(RESOURCE_PACKAGE), "FungibleBucket".to_string()),
            )),
        )
    }
    pub const OWN_NON_FUNGIBLE_BUCKET_ID: u8 = VALUE_KIND_OWN + 3;
    pub fn own_non_fungible_bucket_type_data<L: SchemaTypeLink>(
    ) -> TypeData<ScryptoCustomTypeKind, L> {
        // Bucket is not clear from the address in the value, so add it as a type name
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
    }
    pub const OWN_PROOF_ID: u8 = VALUE_KIND_OWN + 4;
    pub fn own_proof_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        // Proof is not clear from the address in the value, so add it as a type name
        named_type_kind(
            "Proof",
            ScryptoCustomTypeKind::Own,
            Some(ScryptoCustomTypeValidation::Own(OwnValidation::IsProof)),
        )
    }
    pub const OWN_FUNGIBLE_PROOF_ID: u8 = VALUE_KIND_OWN + 5;
    pub fn own_fungible_proof_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        // Proof is not clear from the address in the value, so add it as a type name
        named_type_kind(
            "FungibleProof",
            ScryptoCustomTypeKind::Own,
            Some(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(Some(RESOURCE_PACKAGE), "FungibleProof".to_string()),
            )),
        )
    }
    pub const OWN_NON_FUNGIBLE_PROOF_ID: u8 = VALUE_KIND_OWN + 6;
    pub fn own_non_fungible_proof_type_data<L: SchemaTypeLink>(
    ) -> TypeData<ScryptoCustomTypeKind, L> {
        // Proof is not clear from the address in the value, so add it as a type name
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
    }
    pub const OWN_VAULT_ID: u8 = VALUE_KIND_OWN + 7;
    pub fn own_vault_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        // Record the type name, despite information available from node id.
        named_type_kind(
            "Vault",
            ScryptoCustomTypeKind::Own,
            Some(ScryptoCustomTypeValidation::Own(OwnValidation::IsVault)),
        )
    }
    pub const OWN_FUNGIBLE_VAULT_ID: u8 = VALUE_KIND_OWN + 8;
    pub fn own_fungible_vault_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        // Record the type name, despite information available from node id.
        named_type_kind(
            "FungibleVault",
            ScryptoCustomTypeKind::Own,
            Some(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(Some(RESOURCE_PACKAGE), "FungibleVault".to_string()),
            )),
        )
    }
    pub const OWN_NON_FUNGIBLE_VAULT_ID: u8 = VALUE_KIND_OWN + 9;
    pub fn own_non_fungible_vault_type_data<L: SchemaTypeLink>(
    ) -> TypeData<ScryptoCustomTypeKind, L> {
        // Record the type name, despite information available from node id.
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
    }
    pub const OWN_KEY_VALUE_STORE_ID: u8 = VALUE_KIND_OWN + 10;
    pub fn own_key_value_store_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L>
    {
        // Record the type name, despite information available from node id.
        named_type_kind(
            "KeyValueStore",
            ScryptoCustomTypeKind::Own,
            Some(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsKeyValueStore,
            )),
        )
    }
    pub const OWN_GLOBAL_ADDRESS_RESERVATION_ID: u8 = VALUE_KIND_OWN + 11;
    pub fn own_global_address_reservation_type_data<L: SchemaTypeLink>(
    ) -> TypeData<ScryptoCustomTypeKind, L> {
        // Record the type name, despite information available from node id.
        named_type_kind(
            "GlobalAddressReservation",
            ScryptoCustomTypeKind::Own,
            Some(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsGlobalAddressReservation,
            )),
        )
    }

    pub const DECIMAL_ID: u8 = VALUE_KIND_DECIMAL;
    pub fn decimal_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        unnamed_type_kind(ScryptoCustomTypeKind::Decimal, None)
    }
    pub const PRECISE_DECIMAL_ID: u8 = VALUE_KIND_PRECISE_DECIMAL;
    pub fn precise_decimal_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L> {
        unnamed_type_kind(ScryptoCustomTypeKind::PreciseDecimal, None)
    }
    pub const NON_FUNGIBLE_LOCAL_ID_ID: u8 = VALUE_KIND_NON_FUNGIBLE_LOCAL_ID;
    pub fn non_fungible_local_id_type_data<L: SchemaTypeLink>() -> TypeData<ScryptoCustomTypeKind, L>
    {
        unnamed_type_kind(ScryptoCustomTypeKind::NonFungibleLocalId, None)
    }
}

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
    ScryptoCustomTypeKind,
    [
        // Addresses
        (REFERENCE_ID, reference_type_data()),
        (GLOBAL_ADDRESS_ID, global_address_type_data()),
        (INTERNAL_ADDRESS_ID, internal_address_type_data()),
        (PACKAGE_ADDRESS_ID, package_address_type_data()),
        (COMPONENT_ADDRESS_ID, component_address_type_data()),
        (RESOURCE_ADDRESS_ID, resource_address_type_data()),
        // Owned entities
        (OWN_ID, own_type_data()),
        (OWN_BUCKET_ID, own_bucket_type_data()),
        (OWN_PROOF_ID, own_proof_type_data()),
        (OWN_VAULT_ID, own_vault_type_data()),
        (OWN_KEY_VALUE_STORE_ID, own_key_value_store_type_data()),
        (
            OWN_GLOBAL_ADDRESS_RESERVATION_ID,
            own_global_address_reservation_type_data()
        ),
        // Others
        (DECIMAL_ID, decimal_type_data()),
        (PRECISE_DECIMAL_ID, precise_decimal_type_data()),
        (NON_FUNGIBLE_LOCAL_ID_ID, non_fungible_local_id_type_data()),
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
