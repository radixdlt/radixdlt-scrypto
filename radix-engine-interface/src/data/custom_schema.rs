use super::*;
use sbor::rust::collections::*;
use sbor::*;

pub type ScryptoTypeKind<L> = TypeKind<ScryptoCustomValueKind, ScryptoCustomTypeKind<L>, L>;
pub type ScryptoSchema = Schema<ScryptoCustomTypeExtension>;

/// A schema for the values that a codec can decode / views as valid
#[derive(Debug, Clone, PartialEq, Eq, Categorize, Encode, Decode)]
pub enum ScryptoCustomTypeKind<L: SchemaTypeLink> {
    Reference, /* any */
    PackageAddress,
    ComponentAddress,
    ResourceAddress,

    Own, /* any */
    KeyValueStore { key_type: L, value_type: L },

    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,

    PublicKey, /* any */
    EcdsaSecp256k1PublicKey,
    EddsaEd25519PublicKey,
}

impl<L: SchemaTypeLink> CustomTypeKind<L> for ScryptoCustomTypeKind<L> {
    type CustomValueKind = ScryptoCustomValueKind;
    type CustomTypeExtension = ScryptoCustomTypeExtension;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomTypeValidation {}

impl CustomTypeValidation for ScryptoCustomTypeValidation {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScryptoCustomTypeExtension {}

impl CustomTypeExtension for ScryptoCustomTypeExtension {
    type CustomValueKind = ScryptoCustomValueKind;
    type CustomTypeKind<L: SchemaTypeLink> = ScryptoCustomTypeKind<L>;
    type CustomTypeValidation = ScryptoCustomTypeValidation;

    fn linearize_type_kind(
        type_kind: Self::CustomTypeKind<GlobalTypeId>,
        type_indices: &BTreeMap<TypeHash, usize>,
    ) -> Self::CustomTypeKind<LocalTypeIndex> {
        match type_kind {
            ScryptoCustomTypeKind::Reference => ScryptoCustomTypeKind::Reference,
            ScryptoCustomTypeKind::PackageAddress => ScryptoCustomTypeKind::PackageAddress,
            ScryptoCustomTypeKind::ComponentAddress => ScryptoCustomTypeKind::ComponentAddress,
            ScryptoCustomTypeKind::ResourceAddress => ScryptoCustomTypeKind::ResourceAddress,

            ScryptoCustomTypeKind::Own => ScryptoCustomTypeKind::Own,
            ScryptoCustomTypeKind::KeyValueStore {
                key_type,
                value_type,
            } => ScryptoCustomTypeKind::KeyValueStore {
                key_type: resolve_local_type_ref(type_indices, &key_type),
                value_type: resolve_local_type_ref(type_indices, &value_type),
            },

            ScryptoCustomTypeKind::Decimal => ScryptoCustomTypeKind::Decimal,
            ScryptoCustomTypeKind::PreciseDecimal => ScryptoCustomTypeKind::PreciseDecimal,
            ScryptoCustomTypeKind::NonFungibleLocalId => ScryptoCustomTypeKind::NonFungibleLocalId,
            ScryptoCustomTypeKind::PublicKey => ScryptoCustomTypeKind::PublicKey,
            ScryptoCustomTypeKind::EcdsaSecp256k1PublicKey => {
                ScryptoCustomTypeKind::EcdsaSecp256k1PublicKey
            }
            ScryptoCustomTypeKind::EddsaEd25519PublicKey => {
                ScryptoCustomTypeKind::EddsaEd25519PublicKey
            }
        }
    }

    fn resolve_custom_well_known_type(
        well_known_index: u8,
    ) -> Option<TypeData<Self::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>> {
        let (name, custom_type_kind) = match well_known_index {
            REFERENCE_ID => ("Reference", ScryptoCustomTypeKind::Reference),
            OWN_ID => ("Own", ScryptoCustomTypeKind::Own),

            DECIMAL_ID => ("Decimal", ScryptoCustomTypeKind::Decimal),
            PRECISE_DECIMAL_ID => ("PreciseDecimal", ScryptoCustomTypeKind::PreciseDecimal),
            NON_FUNGIBLE_LOCAL_ID_ID => (
                "NonFungibleLocalId",
                ScryptoCustomTypeKind::NonFungibleLocalId,
            ),
            PUBLIC_KEY_ID => ("PublicKey", ScryptoCustomTypeKind::PublicKey),
            _ => return None,
        };

        Some(TypeData::named_no_child_names(
            name,
            TypeKind::Custom(custom_type_kind),
        ))
    }
}

use well_known_scrypto_types::*;

mod well_known_scrypto_types {
    use super::*;

    pub const REFERENCE_ID: u8 = VALUE_KIND_REFERENCE;
    // TODO: add support for specific variants

    pub const OWN_ID: u8 = VALUE_KIND_OWN;
    // TODO: add support for specific variants
    // We skip KeyValueStore because it has generic parameters

    pub const DECIMAL_ID: u8 = VALUE_KIND_DECIMAL;
    pub const PRECISE_DECIMAL_ID: u8 = VALUE_KIND_PRECISE_DECIMAL;
    pub const NON_FUNGIBLE_LOCAL_ID_ID: u8 = VALUE_KIND_NON_FUNGIBLE_LOCAL_ID;
    pub const PUBLIC_KEY_ID: u8 = VALUE_KIND_PUBLIC_KEY;
}
