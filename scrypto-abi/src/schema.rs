use crate::v2::*;
use sbor::rust::borrow::Cow;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
pub use well_known::*;

/// The `Schema` trait allows a type to describe how to interpret and validate a corresponding SBOR payload.
///
/// Each unique interpretation/validation of a type should have its own distinct type in the schema.
/// Uniqueness of a type in the schema is defined by its TypeRef.
#[allow(unused_variables)]
pub trait Schema<C: CustomTypeSchema> {
    /// The `TYPE_REF` should denote a unique identifier for this type (once turned into a payload)
    ///
    /// In particular, it should capture the uniqueness of anything relevant to the codec/payload, for example:
    /// * The payloads the codec can decode
    /// * The uniqueness of display instructions applied to the payload. EG if a wrapper type is intended to give
    ///   the value a different display interpretation, this should create a unique identifier.
    ///
    /// Note however that entirely "transparent" types such as pointers/smart pointers/etc are intended to be
    /// transparent to the schema, so should inherit their wrapped type id.
    ///
    /// If needing to generate a new type id, this can be generated via something like:
    /// ```
    /// impl Schema for MyType {
    ///     const SCHEMA_TYPE_REF: GlobalTypeRef = GlobalTypeRef::complex(stringify!(MyType), &[], &[]);
    /// #   fn get_local_type_data() { todo!() }
    /// }
    /// ```
    const SCHEMA_TYPE_REF: GlobalTypeRef;

    /// Returns the local schema for the given type, if the TypeRef is Custom
    fn get_local_type_data() -> Option<LocalTypeData<C, GlobalTypeRef>> {
        None
    }

    /// Should add all the dependent schemas, if the type depends on any.
    ///
    /// For direct/simple type dependencies, simply call `aggregator.add_child_type_and_descendents::<D>()`
    /// for each dependency.
    ///
    /// For more complicated type dependencies, where new types are being created (EG enum variants, or
    /// where a dependent type ie being customised/mutated via annotations), then the algorithm should be:
    ///
    /// - For each (possibly customised) type dependency needed directly by this type
    ///   - Ensure that if it's customised, then its `type_ref` is mutated from its underlying type
    ///   - Do `aggregator.add_child_type(type_ref, local_type_data)`
    ///
    /// - For each (base/unmutated) type dependency `D`:
    ///   - Call `aggregator.add_schema_descendents::<D>()`
    fn add_all_dependencies(aggregator: &mut SchemaAggregator<C>) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTypeData<C: CustomTypeSchema, L: TypeLink> {
    pub schema: TypeSchema<C, L>,
    pub naming: TypeNaming,
}

impl<C: CustomTypeSchema, L: TypeLink> LocalTypeData<C, L> {
    pub const fn named(name: &'static str, schema: TypeSchema<C, L>) -> Self {
        Self {
            schema,
            naming: TypeNaming {
                type_name: Cow::Borrowed(name),
                field_names: None,
            },
        }
    }

    pub const fn named_unit(name: &'static str) -> Self {
        Self {
            schema: TypeSchema::Unit,
            naming: TypeNaming {
                type_name: Cow::Borrowed(name),
                field_names: None,
            },
        }
    }

    pub const fn named_tuple(name: &'static str, element_types: Vec<L>) -> Self {
        Self {
            schema: TypeSchema::Tuple { element_types },
            naming: TypeNaming {
                type_name: Cow::Borrowed(name),
                field_names: None,
            },
        }
    }

    pub fn named_tuple_named_fields(
        name: &'static str,
        element_types: Vec<L>,
        field_names: &[&'static str],
    ) -> Self {
        Self {
            schema: TypeSchema::Tuple { element_types },
            naming: TypeNaming {
                type_name: Cow::Borrowed(name),
                field_names: Some(field_names.iter().map(|x| x.to_string()).collect()),
            },
        }
    }
}

/// This enables the type to be represented as eg JSON
/// Also used to facilitate type reconstruction
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeNaming {
    pub type_name: Cow<'static, str>,
    pub field_names: Option<Vec<String>>,
}

impl TypeNaming {
    pub const fn named(name: &'static str) -> Self {
        Self {
            type_name: Cow::Borrowed(name),
            field_names: None,
        }
    }
}

/// An array of custom types, and associated extra information.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullTypeSchema<C: CustomTypeSchema> {
    pub custom_types: Vec<TypeSchema<C, SchemaLocalTypeRef>>,
    pub naming: Vec<TypeNaming>,
}

pub struct ResolvedLocalTypeData<'a, C: CustomTypeSchema> {
    pub schema: Cow<'a, TypeSchema<C, SchemaLocalTypeRef>>,
    pub naming: Cow<'a, TypeNaming>,
}

impl<C: CustomTypeSchema> FullTypeSchema<C> {
    pub fn resolve<'a, W: CustomWellKnownType<CustomTypeSchema = C>>(
        &'a self,
        type_ref: SchemaLocalTypeRef,
    ) -> Option<ResolvedLocalTypeData<'a, C>> {
        match type_ref {
            SchemaLocalTypeRef::WellKnown(index) => {
                resolve_well_known_type_data::<W>(index).map(|local_type_data| {
                    ResolvedLocalTypeData {
                        schema: Cow::Owned(local_type_data.schema),
                        naming: Cow::Owned(local_type_data.naming),
                    }
                })
            }
            SchemaLocalTypeRef::SchemaLocal(index) => {
                match (self.custom_types.get(index), self.naming.get(index)) {
                    (Some(schema), Some(naming)) => Some(ResolvedLocalTypeData {
                        schema: Cow::Borrowed(schema),
                        naming: Cow::Borrowed(naming),
                    }),
                    (None, None) => None,
                    _ => panic!("Index existed in exactly one of schema and naming"),
                }
            }
        }
    }
}

// COPIED FROM radix-engine-interfaces
// TODO - merge these crates to avoid copying this!!
mod scrypto_custom_type_ids {
    pub const TYPE_PACKAGE_ADDRESS: u8 = 0x80;
    pub const TYPE_COMPONENT_ADDRESS: u8 = 0x81;
    pub const TYPE_RESOURCE_ADDRESS: u8 = 0x82;
    pub const TYPE_SYSTEM_ADDRESS: u8 = 0x83;
    pub const TYPE_COMPONENT: u8 = 0x90;
    pub const TYPE_BUCKET: u8 = 0x92;
    pub const TYPE_PROOF: u8 = 0x93;
    pub const TYPE_VAULT: u8 = 0x94;
    pub const TYPE_EXPRESSION: u8 = 0xa0;
    pub const TYPE_BLOB: u8 = 0xa1;
    pub const TYPE_NON_FUNGIBLE_ADDRESS: u8 = 0xa2;
    pub const TYPE_HASH: u8 = 0xb0;
    pub const TYPE_ECDSA_SECP256K1_PUBLIC_KEY: u8 = 0xb1;
    pub const TYPE_ECDSA_SECP256K1_SIGNATURE: u8 = 0xb2;
    pub const TYPE_EDDSA_ED25519_PUBLIC_KEY: u8 = 0xb3;
    pub const TYPE_EDDSA_ED25519_SIGNATURE: u8 = 0xb4;
    pub const TYPE_DECIMAL: u8 = 0xb5;
    pub const TYPE_PRECISE_DECIMAL: u8 = 0xb6;
    pub const TYPE_NON_FUNGIBLE_ID: u8 = 0xb7;
}

pub mod well_known {
    use super::scrypto_custom_type_ids::*;
    use super::*;

    pub use basic_indices::*;
    pub use scrypto_indices::*;

    pub const CUSTOM_WELL_KNOWN_TYPE_START: u8 = 0x80;

    mod basic_indices {
        use sbor::*;

        // These must be usable in a const context
        pub const ANY_INDEX: u8 = 0x40;

        pub const UNIT_INDEX: u8 = TYPE_UNIT;
        pub const BOOL_INDEX: u8 = TYPE_BOOL;

        pub const I8_INDEX: u8 = TYPE_I8;
        pub const I16_INDEX: u8 = TYPE_I16;
        pub const I32_INDEX: u8 = TYPE_I32;
        pub const I64_INDEX: u8 = TYPE_I64;
        pub const I128_INDEX: u8 = TYPE_I128;

        pub const U8_INDEX: u8 = TYPE_U8;
        pub const U16_INDEX: u8 = TYPE_U16;
        pub const U32_INDEX: u8 = TYPE_U32;
        pub const U64_INDEX: u8 = TYPE_U64;
        pub const U128_INDEX: u8 = TYPE_U128;

        pub const STRING_INDEX: u8 = TYPE_STRING;

        pub const BYTES_INDEX: u8 = 0x41;
    }

    pub enum WellKnownType<X: CustomWellKnownType> {
        // Any
        Any,
        // Basic, limitless
        Unit,
        Bool,
        I8,
        I16,
        I32,
        I64,
        I128,
        U8,
        U16,
        U32,
        U64,
        U128,
        String,
        // Common aliases
        Bytes,
        // Custom
        Custom(X),
    }

    pub fn resolve_well_known_type_data<W: CustomWellKnownType>(
        well_known_index: u8,
    ) -> Option<LocalTypeData<W::CustomTypeSchema, SchemaLocalTypeRef>> {
        let type_data = match well_known_index {
            ANY_INDEX => LocalTypeData::named("Any", TypeSchema::Any),

            UNIT_INDEX => LocalTypeData::named("-", TypeSchema::Unit),
            BOOL_INDEX => LocalTypeData::named("Bool", TypeSchema::Bool),

            I8_INDEX => LocalTypeData::named(
                "I8",
                TypeSchema::I8 {
                    validation: NumericValidation::none(),
                },
            ),
            I16_INDEX => LocalTypeData::named(
                "I16",
                TypeSchema::I16 {
                    validation: NumericValidation::none(),
                },
            ),
            I32_INDEX => LocalTypeData::named(
                "I32",
                TypeSchema::I32 {
                    validation: NumericValidation::none(),
                },
            ),
            I64_INDEX => LocalTypeData::named(
                "I64",
                TypeSchema::I64 {
                    validation: NumericValidation::none(),
                },
            ),
            I128_INDEX => LocalTypeData::named(
                "I128",
                TypeSchema::I128 {
                    validation: NumericValidation::none(),
                },
            ),

            U8_INDEX => LocalTypeData::named(
                "U8",
                TypeSchema::U8 {
                    validation: NumericValidation::none(),
                },
            ),
            U16_INDEX => LocalTypeData::named(
                "U16",
                TypeSchema::U16 {
                    validation: NumericValidation::none(),
                },
            ),
            U32_INDEX => LocalTypeData::named(
                "U32",
                TypeSchema::U32 {
                    validation: NumericValidation::none(),
                },
            ),
            U64_INDEX => LocalTypeData::named(
                "U64",
                TypeSchema::U64 {
                    validation: NumericValidation::none(),
                },
            ),
            U128_INDEX => LocalTypeData::named(
                "U128",
                TypeSchema::U128 {
                    validation: NumericValidation::none(),
                },
            ),

            STRING_INDEX => LocalTypeData::named(
                "String",
                TypeSchema::String {
                    length_validation: LengthValidation::none(),
                },
            ),

            BYTES_INDEX => LocalTypeData::named(
                "Bytes",
                TypeSchema::Array {
                    element_sbor_type_id: sbor::TYPE_U8,
                    element_type: SchemaLocalTypeRef::WellKnown(U8_INDEX),
                    length_validation: LengthValidation::none(),
                },
            ),
            index if index >= CUSTOM_WELL_KNOWN_TYPE_START => {
                return W::from_well_known_index(index)
            }
            _ => return None,
        };
        Some(type_data)
    }

    pub trait CustomWellKnownType {
        type CustomTypeSchema: CustomTypeSchema;

        fn from_well_known_index(
            well_known_index: u8,
        ) -> Option<LocalTypeData<Self::CustomTypeSchema, SchemaLocalTypeRef>>;
    }

    mod scrypto_indices {
        use super::*;

        pub const PACKAGE_ADDRESS_INDEX: u8 = TYPE_PACKAGE_ADDRESS;
        pub const COMPONENT_ADDRESS_INDEX: u8 = TYPE_COMPONENT_ADDRESS;
        pub const RESOURCE_ADDRESS_INDEX: u8 = TYPE_RESOURCE_ADDRESS;
        pub const SYSTEM_ADDRESS_INDEX: u8 = TYPE_SYSTEM_ADDRESS;

        pub const COMPONENT_INDEX: u8 = TYPE_COMPONENT;
        pub const BUCKET_INDEX: u8 = TYPE_BUCKET;
        pub const PROOF_INDEX: u8 = TYPE_PROOF;
        pub const VAULT_INDEX: u8 = TYPE_VAULT;

        pub const EXPRESSION_INDEX: u8 = TYPE_EXPRESSION;
        pub const BLOB_INDEX: u8 = TYPE_BLOB;
        pub const NON_FUNGIBLE_ADDRESS_INDEX: u8 = TYPE_NON_FUNGIBLE_ADDRESS;

        pub const HASH_INDEX: u8 = TYPE_HASH;
        pub const ECDSA_SECP256K1_PUBLIC_KEY_INDEX: u8 = TYPE_ECDSA_SECP256K1_PUBLIC_KEY;
        pub const ECDSA_SECP256K1_SIGNATURE_INDEX: u8 = TYPE_ECDSA_SECP256K1_SIGNATURE;
        pub const EDDSA_ED25519_PUBLIC_KEY_INDEX: u8 = TYPE_EDDSA_ED25519_PUBLIC_KEY;
        pub const EDDSA_ED25519_SIGNATURE_INDEX: u8 = TYPE_EDDSA_ED25519_SIGNATURE;
        pub const DECIMAL_INDEX: u8 = TYPE_DECIMAL;
        pub const PRECISE_DECIMAL_INDEX: u8 = TYPE_PRECISE_DECIMAL;
        pub const NON_FUNGIBLE_ID_INDEX: u8 = TYPE_NON_FUNGIBLE_ID;
    }

    pub enum ScryptoCustomWellKnownType {}

    impl CustomWellKnownType for ScryptoCustomWellKnownType {
        type CustomTypeSchema = ScryptoCustomTypeSchema<SchemaLocalTypeRef>;

        fn from_well_known_index(
            well_known_index: u8,
        ) -> Option<LocalTypeData<Self::CustomTypeSchema, SchemaLocalTypeRef>> {
            let (name, custom_type_schema) = match well_known_index {
                PACKAGE_ADDRESS_INDEX => {
                    ("PackageAddress", ScryptoCustomTypeSchema::PackageAddress)
                }
                COMPONENT_ADDRESS_INDEX => (
                    "ComponentAddress",
                    ScryptoCustomTypeSchema::ComponentAddress,
                ),
                RESOURCE_ADDRESS_INDEX => {
                    ("ResourceAddress", ScryptoCustomTypeSchema::ResourceAddress)
                }
                SYSTEM_ADDRESS_INDEX => ("SystemAddress", ScryptoCustomTypeSchema::SystemAddress),

                COMPONENT_INDEX => ("Component", ScryptoCustomTypeSchema::Component),
                BUCKET_INDEX => ("Bucket", ScryptoCustomTypeSchema::Bucket),
                PROOF_INDEX => ("Proof", ScryptoCustomTypeSchema::Proof),
                VAULT_INDEX => ("Vault", ScryptoCustomTypeSchema::Vault),

                EXPRESSION_INDEX => ("Expression", ScryptoCustomTypeSchema::Expression),
                BLOB_INDEX => ("Blob", ScryptoCustomTypeSchema::Blob),
                NON_FUNGIBLE_ADDRESS_INDEX => (
                    "NonFungibleAddress",
                    ScryptoCustomTypeSchema::NonFungibleAddress,
                ),

                HASH_INDEX => ("Hash", ScryptoCustomTypeSchema::Hash),
                ECDSA_SECP256K1_PUBLIC_KEY_INDEX => (
                    "EcdsaSecp256k1PublicKey",
                    ScryptoCustomTypeSchema::EcdsaSecp256k1PublicKey,
                ),
                ECDSA_SECP256K1_SIGNATURE_INDEX => (
                    "EcdsaSecp256k1Signature",
                    ScryptoCustomTypeSchema::EcdsaSecp256k1Signature,
                ),
                EDDSA_ED25519_PUBLIC_KEY_INDEX => (
                    "EddsaEd25519PublicKey",
                    ScryptoCustomTypeSchema::EddsaEd25519PublicKey,
                ),
                EDDSA_ED25519_SIGNATURE_INDEX => (
                    "EddsaEd25519Signature",
                    ScryptoCustomTypeSchema::EddsaEd25519Signature,
                ),
                DECIMAL_INDEX => ("Decimal", ScryptoCustomTypeSchema::Decimal),
                PRECISE_DECIMAL_INDEX => {
                    ("PreciseDecimal", ScryptoCustomTypeSchema::PreciseDecimal)
                }
                NON_FUNGIBLE_ID_INDEX => ("NonFungibleId", ScryptoCustomTypeSchema::NonFungibleId),
                _ => return None,
            };

            Some(LocalTypeData::named(
                name,
                TypeSchema::Custom(custom_type_schema),
            ))
        }
    }
}
