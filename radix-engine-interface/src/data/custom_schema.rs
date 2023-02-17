use super::*;
use sbor::rust::collections::*;
use sbor::*;

pub type ScryptoTypeKind<L> = TypeKind<ScryptoCustomValueKind, ScryptoCustomTypeKind<L>, L>;
pub type ScryptoSchema = Schema<ScryptoCustomTypeExtension>;

/// A schema for the values that a codec can decode / views as valid
#[derive(Debug, Clone, PartialEq, Eq, Categorize, Encode, Decode)]
pub enum ScryptoCustomTypeKind<L: SchemaTypeLink> {
    // Global address types
    PackageAddress,
    ComponentAddress,
    ResourceAddress,

    // Other Engine types
    Own,
    NonFungibleGlobalId,
    KeyValueStore { key_type: L, value_type: L },

    // Manifest types
    Blob,
    Bucket,
    Proof,
    Expression,

    // Uninterpreted
    Hash,
    EcdsaSecp256k1PublicKey,
    EcdsaSecp256k1Signature,
    EddsaEd25519PublicKey,
    EddsaEd25519Signature,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
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
        type_indices: &IndexSet<TypeHash>,
    ) -> Self::CustomTypeKind<LocalTypeIndex> {
        match type_kind {
            ScryptoCustomTypeKind::PackageAddress => ScryptoCustomTypeKind::PackageAddress,
            ScryptoCustomTypeKind::ComponentAddress => ScryptoCustomTypeKind::ComponentAddress,
            ScryptoCustomTypeKind::ResourceAddress => ScryptoCustomTypeKind::ResourceAddress,
            ScryptoCustomTypeKind::KeyValueStore {
                key_type,
                value_type,
            } => ScryptoCustomTypeKind::KeyValueStore {
                key_type: resolve_local_type_ref(type_indices, &key_type),
                value_type: resolve_local_type_ref(type_indices, &value_type),
            },
            ScryptoCustomTypeKind::Bucket => ScryptoCustomTypeKind::Bucket,
            ScryptoCustomTypeKind::Proof => ScryptoCustomTypeKind::Proof,
            ScryptoCustomTypeKind::Own => ScryptoCustomTypeKind::Own,
            ScryptoCustomTypeKind::Expression => ScryptoCustomTypeKind::Expression,
            ScryptoCustomTypeKind::Blob => ScryptoCustomTypeKind::Blob,
            ScryptoCustomTypeKind::NonFungibleGlobalId => {
                ScryptoCustomTypeKind::NonFungibleGlobalId
            }
            ScryptoCustomTypeKind::Hash => ScryptoCustomTypeKind::Hash,
            ScryptoCustomTypeKind::EcdsaSecp256k1PublicKey => {
                ScryptoCustomTypeKind::EcdsaSecp256k1PublicKey
            }
            ScryptoCustomTypeKind::EcdsaSecp256k1Signature => {
                ScryptoCustomTypeKind::EcdsaSecp256k1Signature
            }
            ScryptoCustomTypeKind::EddsaEd25519PublicKey => {
                ScryptoCustomTypeKind::EddsaEd25519PublicKey
            }
            ScryptoCustomTypeKind::EddsaEd25519Signature => {
                ScryptoCustomTypeKind::EddsaEd25519Signature
            }
            ScryptoCustomTypeKind::Decimal => ScryptoCustomTypeKind::Decimal,
            ScryptoCustomTypeKind::PreciseDecimal => ScryptoCustomTypeKind::PreciseDecimal,
            ScryptoCustomTypeKind::NonFungibleLocalId => ScryptoCustomTypeKind::NonFungibleLocalId,
        }
    }

    fn resolve_custom_well_known_type(
        well_known_index: u8,
    ) -> Option<TypeData<Self::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>> {
        resolve_scrypto_custom_well_known_type(well_known_index)
    }

    fn validate_type_kind(
        context: &TypeValidationContext,
        type_kind: &SchemaCustomTypeKind<Self>,
    ) -> Result<(), SchemaValidationError> {
        match type_kind {
            ScryptoCustomTypeKind::PackageAddress
            | ScryptoCustomTypeKind::ComponentAddress
            | ScryptoCustomTypeKind::ResourceAddress
            | ScryptoCustomTypeKind::Own
            | ScryptoCustomTypeKind::NonFungibleGlobalId
            | ScryptoCustomTypeKind::Blob
            | ScryptoCustomTypeKind::Bucket
            | ScryptoCustomTypeKind::Proof
            | ScryptoCustomTypeKind::Expression
            | ScryptoCustomTypeKind::Hash
            | ScryptoCustomTypeKind::EcdsaSecp256k1PublicKey
            | ScryptoCustomTypeKind::EcdsaSecp256k1Signature
            | ScryptoCustomTypeKind::EddsaEd25519PublicKey
            | ScryptoCustomTypeKind::EddsaEd25519Signature
            | ScryptoCustomTypeKind::Decimal
            | ScryptoCustomTypeKind::PreciseDecimal
            | ScryptoCustomTypeKind::NonFungibleLocalId => {
                // No validations
            }
            ScryptoCustomTypeKind::KeyValueStore {
                key_type,
                value_type,
            } => {
                validate_index::<Self>(context, key_type)?;
                validate_index::<Self>(context, value_type)?;
            }
        }
        Ok(())
    }

    fn validate_type_metadata_with_type_kind(
        _: &TypeValidationContext,
        type_kind: &SchemaCustomTypeKind<Self>,
        type_metadata: &TypeMetadata,
    ) -> Result<(), SchemaValidationError> {
        // Even though they all map to the same thing, we keep the explicit match statement so that
        // we will have to explicitly check this when we add a new `ScryptoCustomTypeKind`
        match type_kind {
            ScryptoCustomTypeKind::PackageAddress
            | ScryptoCustomTypeKind::ComponentAddress
            | ScryptoCustomTypeKind::ResourceAddress
            | ScryptoCustomTypeKind::Own
            | ScryptoCustomTypeKind::NonFungibleGlobalId
            | ScryptoCustomTypeKind::Blob
            | ScryptoCustomTypeKind::Bucket
            | ScryptoCustomTypeKind::Proof
            | ScryptoCustomTypeKind::Expression
            | ScryptoCustomTypeKind::Hash
            | ScryptoCustomTypeKind::EcdsaSecp256k1PublicKey
            | ScryptoCustomTypeKind::EcdsaSecp256k1Signature
            | ScryptoCustomTypeKind::EddsaEd25519PublicKey
            | ScryptoCustomTypeKind::EddsaEd25519Signature
            | ScryptoCustomTypeKind::Decimal
            | ScryptoCustomTypeKind::PreciseDecimal
            | ScryptoCustomTypeKind::NonFungibleLocalId
            | ScryptoCustomTypeKind::KeyValueStore { .. } => {
                validate_childless_metadata(type_metadata)?;
            }
        }
        Ok(())
    }

    fn validate_type_validation_with_type_kind(
        _: &TypeValidationContext,
        type_kind: &SchemaCustomTypeKind<Self>,
        _: &SchemaCustomTypeValidation<Self>,
    ) -> Result<(), SchemaValidationError> {
        // NOTE:
        // Right now SchemaCustomTypeValidation is an empty enum, so it'd be reasonable to panic,
        // but soon this will contain custom validations (eg for Address), so the below code
        // is in preparation for when we add these in.

        match type_kind {
            // Even though they all map to the same thing, we keep the explicit match statement so that
            // we will have to explicitly check this when we add a new `ScryptoCustomTypeKind`
            ScryptoCustomTypeKind::PackageAddress
            | ScryptoCustomTypeKind::ComponentAddress
            | ScryptoCustomTypeKind::ResourceAddress
            | ScryptoCustomTypeKind::Own
            | ScryptoCustomTypeKind::NonFungibleGlobalId
            | ScryptoCustomTypeKind::Blob
            | ScryptoCustomTypeKind::Bucket
            | ScryptoCustomTypeKind::Proof
            | ScryptoCustomTypeKind::Expression
            | ScryptoCustomTypeKind::Hash
            | ScryptoCustomTypeKind::EcdsaSecp256k1PublicKey
            | ScryptoCustomTypeKind::EcdsaSecp256k1Signature
            | ScryptoCustomTypeKind::EddsaEd25519PublicKey
            | ScryptoCustomTypeKind::EddsaEd25519Signature
            | ScryptoCustomTypeKind::Decimal
            | ScryptoCustomTypeKind::PreciseDecimal
            | ScryptoCustomTypeKind::NonFungibleLocalId
            | ScryptoCustomTypeKind::KeyValueStore { .. } => {
                // All these custom type kinds only support `SchemaTypeValidation::None`.
                // If they get to this point, they have been paired with some ScryptoCustomTypeValidation
                // - which isn't valid.
                return Err(SchemaValidationError::TypeValidationMismatch);
            }
        }
    }
}
