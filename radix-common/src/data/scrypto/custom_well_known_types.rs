use super::*;
use crate::constants::RESOURCE_PACKAGE;
use crate::internal_prelude::*;
use basic_well_known_types::*;
use sbor::rust::prelude::*;
use sbor::*;

fn unnamed_custom_type_kind<L: SchemaTypeLink>(
    custom_type_kind: ScryptoCustomTypeKind,
    custom_type_validation: Option<ScryptoCustomTypeValidation>,
) -> ScryptoTypeData<L> {
    TypeData {
        kind: TypeKind::Custom(custom_type_kind),
        metadata: TypeMetadata::unnamed(),
        validation: match custom_type_validation {
            Some(v) => TypeValidation::Custom(v),
            None => TypeValidation::None,
        },
    }
}

fn named_custom_type_kind<L: SchemaTypeLink>(
    name: &'static str,
    custom_type_kind: ScryptoCustomTypeKind,
    custom_type_validation: Option<ScryptoCustomTypeValidation>,
) -> ScryptoTypeData<L> {
    TypeData {
        kind: TypeKind::Custom(custom_type_kind),
        metadata: TypeMetadata::no_child_names(name),
        validation: match custom_type_validation {
            Some(v) => TypeValidation::Custom(v),
            None => TypeValidation::None,
        },
    }
}

fn named_struct<L: SchemaTypeLink>(
    name: &'static str,
    fields: impl IntoIterator<Item = (&'static str, WellKnownTypeId)>,
) -> ScryptoTypeData<L> {
    TypeData::struct_with_named_fields(
        name,
        fields
            .into_iter()
            .map(|(name, field_type)| (name, field_type.into()))
            .collect(),
    )
}

fn named_tuple<L: SchemaTypeLink>(
    name: &'static str,
    fields: impl IntoIterator<Item = WellKnownTypeId>,
) -> ScryptoTypeData<L> {
    let field_types = fields
        .into_iter()
        .map(|field_type| field_type.into())
        .collect();
    TypeData::struct_with_unnamed_fields(name, field_types)
}

fn named_transparent<L: SchemaTypeLink>(
    name: &'static str,
    inner: ScryptoTypeData<L>,
) -> ScryptoTypeData<L> {
    inner.with_name(Some(name.into()))
}

fn array_of<L: SchemaTypeLink>(inner: WellKnownTypeId) -> ScryptoTypeData<L> {
    TypeData {
        kind: TypeKind::Array {
            element_type: inner.into(),
        },
        metadata: TypeMetadata::unnamed(),
        validation: TypeValidation::None,
    }
}

fn named_enum<L: SchemaTypeLink>(
    name: &'static str,
    variants: impl IntoIterator<Item = (u8, ScryptoTypeData<L>)>,
) -> ScryptoTypeData<L> {
    TypeData::enum_variants(name, variants.into_iter().collect())
}

fn bytes_fixed_length_type_data<L: SchemaTypeLink>(length: usize) -> ScryptoTypeData<L> {
    bytes_type_data().with_validation(TypeValidation::Array(LengthValidation {
        min: Some(length.try_into().unwrap()),
        max: Some(length.try_into().unwrap()),
    }))
}

const REFERENCES_START: u8 = 0x80;
const OWNED_ENTITIES_START: u8 = 0xa0;
const MISC_TYPES_START: u8 = 0xc0;
const CRYPTO_TYPES_START: u8 = 0xd0;
const ROLE_ASSIGNMENT_TYPES_START: u8 = 0xe0;
const OTHER_MODULE_TYPES_START: u8 = 0xf0;

//===============================================================================================================
// SCRYPTO WELL KNOWN TYPES
//===============================================================================================================
// These correspond to either actual Scrypto value kinds, or other data types used in Scrypto / likely encountered
// by builders writing blueprints, in their interfaces.
//
// There are a number of reasons for doing this:
// 1. It is an optimization: Shared types mean smaller schemas.
// 2. Having well known types for these so that these types are all the "same type" across all scrypto schemas.
//    This means that semantic information can be interpreted for these values outside of the ledger - for example,
//    it can be used to (eg display URL / Instants in certain ways in the dashboard or wallet).
// 3. In future, if/when we allow attaching extra data such as translations or docs against global types,
//    these types will be defined in a place where we can attach this data across everyones' schemas, instead of
//    being owned by the owner of each schema.
//    In other words, the type's GlobalTypeAddress is well known, instead of being into a schema under a node.
//
// NOTE:
// - Once we create new types, they will need to be separate indices, but can have the same names
//===============================================================================================================
create_well_known_lookup!(
    WELL_KNOWN_LOOKUP,
    well_known_scrypto_custom_types,
    ScryptoCustomTypeKind,
    [
        // References
        (
            REFERENCE,
            REFERENCES_START,
            unnamed_custom_type_kind(ScryptoCustomTypeKind::Reference, None)
        ),
        (
            GLOBAL_ADDRESS,
            REFERENCES_START + 1,
            named_custom_type_kind(
                "GlobalAddress",
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobal,
                ))
            )
        ),
        (
            INTERNAL_ADDRESS,
            REFERENCES_START + 2,
            named_custom_type_kind(
                "InternalAddress",
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsInternal,
                )),
            )
        ),
        (
            PACKAGE_ADDRESS,
            REFERENCES_START + 3,
            named_custom_type_kind(
                "PackageAddress",
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalPackage,
                )),
            )
        ),
        (
            COMPONENT_ADDRESS,
            REFERENCES_START + 4,
            named_custom_type_kind(
                "ComponentAddress",
                ScryptoCustomTypeKind::Reference,
                Some(ScryptoCustomTypeValidation::Reference(
                    ReferenceValidation::IsGlobalComponent,
                )),
            )
        ),
        (
            RESOURCE_ADDRESS,
            REFERENCES_START + 5,
            named_custom_type_kind(
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
            OWNED_ENTITIES_START,
            unnamed_custom_type_kind(ScryptoCustomTypeKind::Own, None)
        ),
        (
            OWN_BUCKET,
            OWNED_ENTITIES_START + 1,
            named_custom_type_kind(
                "Bucket",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(OwnValidation::IsBucket)),
            )
        ),
        (
            OWN_FUNGIBLE_BUCKET,
            OWNED_ENTITIES_START + 2,
            named_custom_type_kind(
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
            OWNED_ENTITIES_START + 3,
            named_custom_type_kind(
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
            OWNED_ENTITIES_START + 4,
            named_custom_type_kind(
                "Proof",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(OwnValidation::IsProof)),
            )
        ),
        (
            OWN_FUNGIBLE_PROOF,
            OWNED_ENTITIES_START + 5,
            named_custom_type_kind(
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
            OWNED_ENTITIES_START + 6,
            named_custom_type_kind(
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
            OWNED_ENTITIES_START + 7,
            named_custom_type_kind(
                "Vault",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(OwnValidation::IsVault)),
            )
        ),
        (
            OWN_FUNGIBLE_VAULT,
            OWNED_ENTITIES_START + 8,
            named_custom_type_kind(
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
            OWNED_ENTITIES_START + 9,
            named_custom_type_kind(
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
            OWNED_ENTITIES_START + 10,
            named_custom_type_kind(
                "KeyValueStore",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsKeyValueStore,
                )),
            )
        ),
        (
            OWN_GLOBAL_ADDRESS_RESERVATION,
            OWNED_ENTITIES_START + 11,
            named_custom_type_kind(
                "GlobalAddressReservation",
                ScryptoCustomTypeKind::Own,
                Some(ScryptoCustomTypeValidation::Own(
                    OwnValidation::IsGlobalAddressReservation,
                )),
            )
        ),
        // Other well known scrypto types.
        // A combination of custom value kinds, composite types, or semantic transparent types
        //
        (
            DECIMAL,
            MISC_TYPES_START + 0,
            unnamed_custom_type_kind(ScryptoCustomTypeKind::Decimal, None)
        ),
        (
            PRECISE_DECIMAL,
            MISC_TYPES_START + 1,
            unnamed_custom_type_kind(ScryptoCustomTypeKind::PreciseDecimal, None)
        ),
        (
            NON_FUNGIBLE_LOCAL_ID,
            MISC_TYPES_START + 2,
            unnamed_custom_type_kind(ScryptoCustomTypeKind::NonFungibleLocalId, None)
        ),
        (
            NON_FUNGIBLE_GLOBAL_ID,
            MISC_TYPES_START + 3,
            named_struct(
                "NonFungibleGlobalId",
                [
                    ("resource_address", RESOURCE_ADDRESS_TYPE),
                    ("local_id", NON_FUNGIBLE_LOCAL_ID_TYPE),
                ]
            )
        ),
        (
            INSTANT,
            MISC_TYPES_START + 4,
            named_transparent("Instant", i64_type_data(),)
        ),
        (
            UTC_DATE_TIME,
            MISC_TYPES_START + 5,
            named_struct(
                "UtcDateTime",
                [
                    ("year", U32_TYPE),
                    ("month", U8_TYPE),
                    ("day_of_month", U8_TYPE),
                    ("hour", U8_TYPE),
                    ("minute", U8_TYPE),
                    ("second", U8_TYPE),
                ]
            )
        ),
        (
            URL,
            MISC_TYPES_START + 6,
            named_transparent("Url", string_type_data(),)
        ),
        (
            ORIGIN,
            MISC_TYPES_START + 7,
            named_transparent("Origin", string_type_data(),)
        ),
        // Crypto-related types from CRYPTO_TYPES_START
        (
            PUBLIC_KEY,
            CRYPTO_TYPES_START + 0,
            named_enum(
                "PublicKey",
                [
                    (0u8, named_tuple("Secp256k1", [SECP256K1_PUBLIC_KEY_TYPE])),
                    (1u8, named_tuple("Ed25519", [ED25519_PUBLIC_KEY_TYPE])),
                ]
            )
        ),
        (
            SECP256K1_PUBLIC_KEY,
            CRYPTO_TYPES_START + 1,
            named_transparent(
                "Secp256k1PublicKey",
                bytes_fixed_length_type_data(Secp256k1PublicKey::LENGTH),
            )
        ),
        (
            ED25519_PUBLIC_KEY,
            CRYPTO_TYPES_START + 2,
            named_transparent(
                "Ed25519PublicKey",
                bytes_fixed_length_type_data(Ed25519PublicKey::LENGTH),
            )
        ),
        (
            PUBLIC_KEY_HASH,
            CRYPTO_TYPES_START + 8,
            named_enum(
                "PublicKeyHash",
                [
                    (
                        0u8,
                        named_tuple("Secp256k1", [SECP256K1_PUBLIC_KEY_HASH_TYPE])
                    ),
                    (1u8, named_tuple("Ed25519", [ED25519_PUBLIC_KEY_HASH_TYPE])),
                ]
            )
        ),
        (
            SECP256K1_PUBLIC_KEY_HASH,
            CRYPTO_TYPES_START + 9,
            named_transparent(
                "Secp256k1PublicKeyHash",
                bytes_fixed_length_type_data(Secp256k1PublicKeyHash::LENGTH),
            )
        ),
        (
            ED25519_PUBLIC_KEY_HASH,
            CRYPTO_TYPES_START + 10,
            named_transparent(
                "Ed25519PublicKeyHash",
                bytes_fixed_length_type_data(Ed25519PublicKeyHash::LENGTH),
            )
        ),
        // ROLE ASSIGNMENT TYPES
        (
            ACCESS_RULE,
            ROLE_ASSIGNMENT_TYPES_START + 0,
            named_enum(
                "AccessRule",
                [
                    (0u8, named_tuple("AllowAll", [])),
                    (1u8, named_tuple("DenyAll", [])),
                    (2u8, named_tuple("Protected", [COMPOSITE_REQUIREMENT_TYPE])),
                ],
            )
        ),
        (
            ///Name of the schema type AccessRuleNode is not changed to CompositeRequirement to preserve backward compatibility.
            COMPOSITE_REQUIREMENT,
            ROLE_ASSIGNMENT_TYPES_START + 1,
            named_enum(
                "AccessRuleNode",
                [
                    (0u8, named_tuple("ProofRule", [BASIC_REQUIREMENT_TYPE])),
                    (1u8, named_tuple("AnyOf", [COMPOSITE_REQUIREMENT_LIST_TYPE])),
                    (2u8, named_tuple("AllOf", [COMPOSITE_REQUIREMENT_LIST_TYPE])),
                ],
            )
        ),
        (
            COMPOSITE_REQUIREMENT_LIST,
            ROLE_ASSIGNMENT_TYPES_START + 2,
            array_of(COMPOSITE_REQUIREMENT_TYPE)
        ),
        (
            ///Name of the schema type ProofRule is not changed to BasicRequirement to preserve backward compatibility.
            BASIC_REQUIREMENT,
            ROLE_ASSIGNMENT_TYPES_START + 3,
            named_enum(
                "ProofRule",
                [
                    (0u8, named_tuple("Require", [RESOURCE_OR_NON_FUNGIBLE_TYPE])),
                    (
                        1u8,
                        named_tuple("AmountOf", [DECIMAL_TYPE, RESOURCE_ADDRESS_TYPE])
                    ),
                    (
                        2u8,
                        named_tuple("CountOf", [U8_TYPE, RESOURCE_OR_NON_FUNGIBLE_LIST_TYPE])
                    ),
                    (
                        3u8,
                        named_tuple("AllOf", [RESOURCE_OR_NON_FUNGIBLE_LIST_TYPE])
                    ),
                    (
                        4u8,
                        named_tuple("AnyOf", [RESOURCE_OR_NON_FUNGIBLE_LIST_TYPE])
                    ),
                ],
            )
        ),
        (
            RESOURCE_OR_NON_FUNGIBLE,
            ROLE_ASSIGNMENT_TYPES_START + 4,
            named_enum(
                "ResourceOrNonFungible",
                [
                    (
                        0u8,
                        named_tuple("NonFungible", [NON_FUNGIBLE_GLOBAL_ID_TYPE])
                    ),
                    (1u8, named_tuple("Resource", [RESOURCE_ADDRESS_TYPE])),
                ],
            )
        ),
        (
            RESOURCE_OR_NON_FUNGIBLE_LIST,
            ROLE_ASSIGNMENT_TYPES_START + 5,
            array_of(RESOURCE_OR_NON_FUNGIBLE_TYPE)
        ),
        (
            OWNER_ROLE,
            ROLE_ASSIGNMENT_TYPES_START + 6,
            named_enum(
                "OwnerRole",
                [
                    (0u8, named_tuple("None", [])),
                    (1u8, named_tuple("Fixed", [ACCESS_RULE_TYPE])),
                    (2u8, named_tuple("Updatable", [ACCESS_RULE_TYPE])),
                ],
            )
        ),
        (
            ROLE_KEY,
            ROLE_ASSIGNMENT_TYPES_START + 7,
            named_transparent("RoleKey", string_type_data(),)
        ),
        // OTHER MODULE TYPES
        (
            MODULE_ID,
            OTHER_MODULE_TYPES_START + 0,
            named_enum(
                "ModuleId",
                [
                    (0u8, named_tuple("Main", [])),
                    (1u8, named_tuple("Metadata", [])),
                    (2u8, named_tuple("Royalty", [])),
                    (3u8, named_tuple("RoleAssignment", [])),
                ],
            )
        ),
        (
            ATTACHED_MODULE_ID,
            OTHER_MODULE_TYPES_START + 1,
            named_enum(
                "AttachedModuleId",
                [
                    (1u8, named_tuple("Metadata", [])),
                    (2u8, named_tuple("Royalty", [])),
                    (3u8, named_tuple("RoleAssignment", [])),
                ],
            )
        ),
        (
            ROYALTY_AMOUNT,
            OTHER_MODULE_TYPES_START + 2,
            named_enum(
                "RoyaltyAmount",
                [
                    (0u8, named_tuple("Free", [])),
                    (1u8, named_tuple("Xrd", [DECIMAL_TYPE])),
                    (2u8, named_tuple("Usd", [DECIMAL_TYPE])),
                ],
            )
        ),
    ]
);

pub fn resolve_scrypto_well_known_type(
    well_known_index: WellKnownTypeId,
) -> Option<&'static ScryptoLocalTypeData> {
    WELL_KNOWN_LOOKUP
        .get(well_known_index.as_index())
        .and_then(|x| x.as_ref())
}

#[cfg(test)]
mod tests {
    use super::well_known_scrypto_custom_types::*;
    use super::*;
    use crate::math::{Decimal, PreciseDecimal};

    #[test]
    fn test_custom_type_values_are_valid() {
        // NOTE: Some of these types are actually defined in the `radix-engine-interface` - so for those,
        // I've instead added tests for them in `interface_well_known_types.rs` in the `radix-engine-interface` crate.
        // But I've kept them in the list below for completeness, in order with the types above - as a comment.

        // MISC TYPES
        test_equivalence(DECIMAL_TYPE, Decimal::from(1));
        test_equivalence(PRECISE_DECIMAL_TYPE, PreciseDecimal::from(1));
        test_equivalence(NON_FUNGIBLE_LOCAL_ID_TYPE, NonFungibleLocalId::integer(2));
        // NonFungibleGlobalId - tested in interface crate
        test_equivalence(INSTANT_TYPE, Instant::new(0));
        test_equivalence(
            UTC_DATE_TIME_TYPE,
            UtcDateTime::from_instant(&Instant::new(0)).unwrap(),
        );
        // URL - tested in interface crate
        // Origin - tested in interface crate

        // CRYPTO-RELATED
        test_equivalence(
            PUBLIC_KEY_TYPE,
            PublicKey::Ed25519(Ed25519PublicKey([0; Ed25519PublicKey::LENGTH])),
        );
        test_equivalence(
            PUBLIC_KEY_TYPE,
            PublicKey::Secp256k1(Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH])),
        );
        test_equivalence(
            ED25519_PUBLIC_KEY_TYPE,
            Ed25519PublicKey([0; Ed25519PublicKey::LENGTH]),
        );
        test_equivalence(
            SECP256K1_PUBLIC_KEY_TYPE,
            Secp256k1PublicKey([0; Secp256k1PublicKey::LENGTH]),
        );
        test_equivalence(
            PUBLIC_KEY_HASH_TYPE,
            PublicKeyHash::Ed25519(Ed25519PublicKeyHash([0; Ed25519PublicKeyHash::LENGTH])),
        );
        test_equivalence(
            PUBLIC_KEY_HASH_TYPE,
            PublicKeyHash::Secp256k1(Secp256k1PublicKeyHash([0; Secp256k1PublicKeyHash::LENGTH])),
        );
        test_equivalence(
            ED25519_PUBLIC_KEY_HASH_TYPE,
            Ed25519PublicKeyHash([0; Ed25519PublicKeyHash::LENGTH]),
        );
        test_equivalence(
            SECP256K1_PUBLIC_KEY_HASH_TYPE,
            Secp256k1PublicKeyHash([0; Secp256k1PublicKeyHash::LENGTH]),
        );
    }

    fn test_equivalence<T: ScryptoEncode + ScryptoDescribe>(id: WellKnownTypeId, value: T) {
        test_type_data_equivalent::<T>(id);
        test_statically_valid(id, value);
    }

    fn test_statically_valid<T: ScryptoEncode>(id: WellKnownTypeId, value: T) {
        let type_name = core::any::type_name::<T>();

        validate_payload_against_schema::<ScryptoCustomExtension, _>(
            &scrypto_encode(&value).unwrap(),
            &ScryptoCustomSchema::empty_schema(),
            id.into(),
            &(),
            10,
        )
        .unwrap_or_else(|err| {
            panic!("Expected value for {type_name} to match well known type but got: {err:?}")
        });
    }

    fn test_type_data_equivalent<T: ScryptoDescribe>(id: WellKnownTypeId) {
        let type_name = core::any::type_name::<T>();

        assert_eq!(T::TYPE_ID, RustTypeId::from(id), "The ScryptoDescribe impl for {type_name} has a TYPE_ID which does not equal its well known type id");
        let localized_type_data =
            localize_well_known_type_data::<ScryptoCustomSchema>(T::type_data());
        let resolved = resolve_scrypto_well_known_type(id)
            .unwrap_or_else(|| panic!("Well known index for {type_name} not found in lookup"));
        assert_eq!(&localized_type_data, resolved, "The ScryptoDescribe impl for {type_name} has type data which does not equal its well known type data");
    }
}
