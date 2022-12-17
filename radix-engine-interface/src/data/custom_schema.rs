use super::*;
use sbor::rust::collections::IndexSet;
use sbor::*;

#[allow(dead_code)]
type ScryptoTypeSchema<TypeLink> =
    TypeSchema<ScryptoCustomTypeId, ScryptoCustomTypeSchema<TypeLink>, TypeLink>;

/// A schema for the values that a codec can decode / views as valid
#[derive(Debug, Clone, PartialEq, Eq, TypeId, Encode, Decode)]
pub enum ScryptoCustomTypeSchema<L: TypeLink> {
    // Global address types
    PackageAddress,
    ComponentAddress,
    ResourceAddress,
    SystemAddress,

    // RE nodes types
    Component,
    KeyValueStore { key_type: L, value_type: L },
    Bucket,
    Proof,
    Vault,

    // Other interpreted types
    Expression,
    Blob,
    NonFungibleAddress,

    // Uninterpreted
    Hash,
    EcdsaSecp256k1PublicKey,
    EcdsaSecp256k1Signature,
    EddsaEd25519PublicKey,
    EddsaEd25519Signature,
    Decimal,
    PreciseDecimal,
    NonFungibleId,
}

impl<L: TypeLink> CustomTypeSchema for ScryptoCustomTypeSchema<L> {
    type CustomTypeId = ScryptoCustomTypeId;
}

impl LinearizableCustomTypeSchema for ScryptoCustomTypeSchema<GlobalTypeRef> {
    type Linearized = ScryptoCustomTypeSchema<SchemaLocalTypeRef>;

    fn linearize(self, schemas: &IndexSet<ComplexTypeHash>) -> Self::Linearized {
        match self {
            Self::PackageAddress => ScryptoCustomTypeSchema::PackageAddress,
            Self::ComponentAddress => ScryptoCustomTypeSchema::ComponentAddress,
            Self::ResourceAddress => ScryptoCustomTypeSchema::ResourceAddress,
            Self::SystemAddress => ScryptoCustomTypeSchema::SystemAddress,
            Self::Component => ScryptoCustomTypeSchema::Component,
            Self::KeyValueStore {
                key_type,
                value_type,
            } => ScryptoCustomTypeSchema::KeyValueStore {
                key_type: resolve_local_type_ref(schemas, &key_type),
                value_type: resolve_local_type_ref(schemas, &value_type),
            },
            Self::Bucket => ScryptoCustomTypeSchema::Bucket,
            Self::Proof => ScryptoCustomTypeSchema::Proof,
            Self::Vault => ScryptoCustomTypeSchema::Vault,
            Self::Expression => ScryptoCustomTypeSchema::Expression,
            Self::Blob => ScryptoCustomTypeSchema::Blob,
            Self::NonFungibleAddress => ScryptoCustomTypeSchema::NonFungibleAddress,
            Self::Hash => ScryptoCustomTypeSchema::Hash,
            Self::EcdsaSecp256k1PublicKey => ScryptoCustomTypeSchema::EcdsaSecp256k1PublicKey,
            Self::EcdsaSecp256k1Signature => ScryptoCustomTypeSchema::EcdsaSecp256k1Signature,
            Self::EddsaEd25519PublicKey => ScryptoCustomTypeSchema::EddsaEd25519PublicKey,
            Self::EddsaEd25519Signature => ScryptoCustomTypeSchema::EddsaEd25519Signature,
            Self::Decimal => ScryptoCustomTypeSchema::Decimal,
            Self::PreciseDecimal => ScryptoCustomTypeSchema::PreciseDecimal,
            Self::NonFungibleId => ScryptoCustomTypeSchema::NonFungibleId,
        }
    }
}

use well_known_scrypto_schemas::*;

mod well_known_scrypto_schemas {
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
            PACKAGE_ADDRESS_INDEX => ("PackageAddress", ScryptoCustomTypeSchema::PackageAddress),
            COMPONENT_ADDRESS_INDEX => (
                "ComponentAddress",
                ScryptoCustomTypeSchema::ComponentAddress,
            ),
            RESOURCE_ADDRESS_INDEX => ("ResourceAddress", ScryptoCustomTypeSchema::ResourceAddress),
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
            PRECISE_DECIMAL_INDEX => ("PreciseDecimal", ScryptoCustomTypeSchema::PreciseDecimal),
            NON_FUNGIBLE_ID_INDEX => ("NonFungibleId", ScryptoCustomTypeSchema::NonFungibleId),
            _ => return None,
        };

        Some(LocalTypeData::named_no_child_names(
            name,
            TypeSchema::Custom(custom_type_schema),
        ))
    }
}
