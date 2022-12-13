use crate::v2::*;
use sbor::CustomTypeId;
use sbor::rust::borrow::Cow;
use sbor::rust::string::String;
use sbor::rust::vec::Vec;
pub use well_known::*;

pub trait Schema<X: CustomTypeId> {
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
    ///     const SCHEMA_TYPE_REF: TypeRef = TypeRef::complex(stringify!(MyType), &[], &[]);
    /// #   fn get_local_type_data() { todo!() }
    /// }
    /// ```
    const SCHEMA_TYPE_REF: TypeRef;

    /// Returns the local schema for the given type, if the TypeRef is Custom
    fn get_local_type_data() -> Option<LocalTypeData<TypeRef>> { None }

    /// Should add all the dependent schemas, if the type depends on any.
    ///
    /// The algorithm should be:
    ///
    /// - For each (POSSIBLY MUTATED) type dependency needed by this type
    ///   - Get its type id and local schema, and mutate (both!) if needed
    ///   - Do aggregator.attempt_add_schema() to the (mutated) hash and (mutated) local schema
    ///
    /// - For each (BASE/UNMUTATED) type dependency `D`:
    ///   - If aggregator.should_read_descendents() then call `D::add_all_dependencies`
    fn add_all_dependencies(_aggregator: &mut SchemaAggregator<X>) {}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalTypeData<T> {
    pub schema: TypeSchema<T>,
    pub naming: TypeNaming,
}

impl<T> LocalTypeData<T> {
    pub const fn named(name: &'static str, schema: TypeSchema<T>) -> Self {
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

    pub const fn named_tuple(name: &'static str, element_types: Vec<T>) -> Self {
        Self {
            schema: TypeSchema::Tuple { element_types },
            naming: TypeNaming {
                type_name: Cow::Borrowed(name),
                field_names: None,
            },
        }
    }

    pub fn named_tuple_named_fields(name: &'static str, element_types: Vec<T>, field_names: &[&'static str]) -> Self {
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
///
/// When it comes to referencing other types in the schema:
/// - Non-negative numbers refer to custom types.
/// - Negative numbers map to well-known types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FullTypeSchema {
    pub custom_types: Vec<TypeSchema<isize>>,
    pub naming: Vec<TypeNaming>,
}

impl FullTypeSchema {
    pub fn get_schema(&self, index: isize) -> Option<&TypeSchema<isize>> {
        if index < 0 {
            let well_known_index = isize_to_well_known_index(index)?;
            well_known::look_up_type(well_known_index).map(|t| &t.schema)
        } else {
            self.custom_types.get(index as usize)
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

    pub use indices::*;
    pub use type_data::*;

    mod indices {
        use super::*;
        use sbor::*;

        pub const UNIT_INDEX: u8 = 0xff; // Can't use 0
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

        pub const ANY_INDEX: u8 = 0xf0;
        pub const BYTES_INDEX: u8 = 0xf1;
    }

    mod type_data {
        use super::*;
        use sbor::*;

        // BASIC TYPES
        pub static ANY_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("Any", TypeSchema::Any);

        pub static UNIT_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("-", TypeSchema::Unit);
        pub static BOOL_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("Bool", TypeSchema::Bool);

        pub static I8_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "I8",
            TypeSchema::I8 {
                validation: NumericValidation::none(),
            },
        );
        pub static I16_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "I16",
            TypeSchema::I16 {
                validation: NumericValidation::none(),
            },
        );
        pub static I32_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "I32",
            TypeSchema::I32 {
                validation: NumericValidation::none(),
            },
        );
        pub static I64_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "I64",
            TypeSchema::I64 {
                validation: NumericValidation::none(),
            },
        );
        pub static I128_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "I128",
            TypeSchema::I128 {
                validation: NumericValidation::none(),
            },
        );

        pub static U8_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "U8",
            TypeSchema::U8 {
                validation: NumericValidation::none(),
            },
        );
        pub static U16_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "U16",
            TypeSchema::U16 {
                validation: NumericValidation::none(),
            },
        );
        pub static U32_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "U32",
            TypeSchema::U32 {
                validation: NumericValidation::none(),
            },
        );
        pub static U64_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "U64",
            TypeSchema::U64 {
                validation: NumericValidation::none(),
            },
        );
        pub static U128_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "U128",
            TypeSchema::U128 {
                validation: NumericValidation::none(),
            },
        );

        pub static STRING_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "String",
            TypeSchema::String {
                length_validation: LengthValidation::none(),
            },
        );

        // RADIX ENGINE TYPES
        pub static PACKAGE_ADDRESS_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("PackageAddress", TypeSchema::PackageAddress);
        pub static COMPONENT_ADDRESS_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("ComponentAddress", TypeSchema::ComponentAddress);
        pub static RESOURCE_ADDRESS_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("ResourceAddress", TypeSchema::ResourceAddress);
        pub static SYSTEM_ADDRESS_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("SystemAddress", TypeSchema::SystemAddress);

        pub static COMPONENT_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("Component", TypeSchema::Component);
        pub static BUCKET_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("Bucket", TypeSchema::Bucket);
        pub static PROOF_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("Proof", TypeSchema::Proof);
        pub static VAULT_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("Vault", TypeSchema::Vault);

        pub static EXPRESSION_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("Expression", TypeSchema::Expression);
        pub static BLOB_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("Blob", TypeSchema::Blob);
        pub static NON_FUNGIBLE_ADDRESS_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("NonFungibleAddress", TypeSchema::NonFungibleAddress);

        pub static HASH_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("Hash", TypeSchema::Hash);
        pub static ECDSA_SECP256K1_PUBLIC_KEY_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named(
                "EcdsaSecp256k1PublicKey",
                TypeSchema::EcdsaSecp256k1PublicKey,
            );
        pub static ECDSA_SECP256K1_SIGNATURE_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "EcdsaSecp256k1Signature",
            TypeSchema::EcdsaSecp256k1Signature,
        );
        pub static EDDSA_ED25519_PUBLIC_KEY_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("EddsaEd25519PublicKey", TypeSchema::EddsaEd25519PublicKey);
        pub static EDDSA_ED25519_SIGNATURE_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("EddsaEd25519Signature", TypeSchema::EddsaEd25519Signature);
        pub static DECIMAL_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("Decimal", TypeSchema::Decimal);
        pub static PRECISE_DECIMAL_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("PreciseDecimal", TypeSchema::PreciseDecimal);
        pub static NON_FUNGIBLE_ID_TYPE_DATA: LocalTypeData<isize> =
            LocalTypeData::named("NonFungibleId", TypeSchema::NonFungibleId);

        pub static BYTES_TYPE_DATA: LocalTypeData<isize> = LocalTypeData::named(
            "Bytes",
            TypeSchema::Array {
                element_sbor_type_id: TYPE_U8,
                element_type: well_known_index_to_isize(U8_INDEX),
                length_validation: LengthValidation::none(),
            },
        );
    }

    pub const fn well_known_index_to_isize(index: u8) -> isize {
        -(index as isize)
    }

    pub const fn isize_to_well_known_index(numeric_type_ref: isize) -> Option<u8> {
        if numeric_type_ref < 0 && numeric_type_ref >= -255 {
            Some((-numeric_type_ref) as u8)
        } else {
            None
        }
    }

    pub fn look_up_type(index: u8) -> Option<&'static LocalTypeData<isize>> {
        match index {
            ANY_INDEX => Some(&ANY_TYPE_DATA),

            UNIT_INDEX => Some(&UNIT_TYPE_DATA),
            BOOL_INDEX => Some(&BOOL_TYPE_DATA),

            I8_INDEX => Some(&I8_TYPE_DATA),
            I16_INDEX => Some(&I16_TYPE_DATA),
            I32_INDEX => Some(&I32_TYPE_DATA),
            I64_INDEX => Some(&I64_TYPE_DATA),
            I128_INDEX => Some(&I128_TYPE_DATA),

            U8_INDEX => Some(&U8_TYPE_DATA),
            U16_INDEX => Some(&U16_TYPE_DATA),
            U32_INDEX => Some(&U32_TYPE_DATA),
            U64_INDEX => Some(&U64_TYPE_DATA),
            U128_INDEX => Some(&U128_TYPE_DATA),

            STRING_INDEX => Some(&STRING_TYPE_DATA),

            PACKAGE_ADDRESS_INDEX => Some(&PACKAGE_ADDRESS_TYPE_DATA),
            COMPONENT_ADDRESS_INDEX => Some(&COMPONENT_ADDRESS_TYPE_DATA),
            RESOURCE_ADDRESS_INDEX => Some(&RESOURCE_ADDRESS_TYPE_DATA),
            SYSTEM_ADDRESS_INDEX => Some(&SYSTEM_ADDRESS_TYPE_DATA),

            COMPONENT_INDEX => Some(&COMPONENT_TYPE_DATA),
            BUCKET_INDEX => Some(&BUCKET_TYPE_DATA),
            PROOF_INDEX => Some(&PROOF_TYPE_DATA),
            VAULT_INDEX => Some(&VAULT_TYPE_DATA),

            EXPRESSION_INDEX => Some(&EXPRESSION_TYPE_DATA),
            BLOB_INDEX => Some(&BLOB_TYPE_DATA),
            NON_FUNGIBLE_ADDRESS_INDEX => Some(&NON_FUNGIBLE_ADDRESS_TYPE_DATA),

            HASH_INDEX => Some(&HASH_TYPE_DATA),
            ECDSA_SECP256K1_PUBLIC_KEY_INDEX => Some(&ECDSA_SECP256K1_PUBLIC_KEY_TYPE_DATA),
            ECDSA_SECP256K1_SIGNATURE_INDEX => Some(&ECDSA_SECP256K1_SIGNATURE_TYPE_DATA),
            EDDSA_ED25519_PUBLIC_KEY_INDEX => Some(&EDDSA_ED25519_PUBLIC_KEY_TYPE_DATA),
            EDDSA_ED25519_SIGNATURE_INDEX => Some(&EDDSA_ED25519_SIGNATURE_TYPE_DATA),
            DECIMAL_INDEX => Some(&DECIMAL_TYPE_DATA),
            PRECISE_DECIMAL_INDEX => Some(&PRECISE_DECIMAL_TYPE_DATA),
            NON_FUNGIBLE_ID_INDEX => Some(&NON_FUNGIBLE_ID_TYPE_DATA),

            BYTES_INDEX => Some(&BYTES_TYPE_DATA),

            _ => None,
        }
    }
}
