use super::*;
use crate::*;
use sbor::rust::collections::*;
use sbor::*;

pub type ScryptoTypeKind<L> = TypeKind<ScryptoCustomValueKind, ScryptoCustomTypeKind, L>;
pub type ScryptoSchema = Schema<ScryptoCustomTypeExtension>;
pub type ScryptoTypeData<L> = TypeData<ScryptoCustomTypeKind, L>;

/// A schema for the values that a codec can decode / views as valid
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum ScryptoCustomTypeKind {
    Reference, /* any */
    GlobalAddress,
    LocalAddress,
    PackageAddress,
    ComponentAddress,
    ResourceAddress,

    Own, /* any */
    Bucket,
    Proof,
    Vault,
    KeyValueStore,

    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
}

impl<L: SchemaTypeLink> CustomTypeKind<L> for ScryptoCustomTypeKind {
    type CustomValueKind = ScryptoCustomValueKind;
    type CustomTypeExtension = ScryptoCustomTypeExtension;
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum ScryptoCustomTypeValidation {}

impl CustomTypeValidation for ScryptoCustomTypeValidation {}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum ScryptoCustomTypeExtension {}

impl CustomTypeExtension for ScryptoCustomTypeExtension {
    const MAX_DEPTH: usize = SCRYPTO_SBOR_V1_MAX_DEPTH;
    const PAYLOAD_PREFIX: u8 = SCRYPTO_SBOR_V1_PAYLOAD_PREFIX;

    type CustomValueKind = ScryptoCustomValueKind;
    type CustomTypeKind<L: SchemaTypeLink> = ScryptoCustomTypeKind;
    type CustomTypeValidation = ScryptoCustomTypeValidation;
    type CustomTraversal = ScryptoCustomTraversal;

    fn linearize_type_kind(
        type_kind: Self::CustomTypeKind<GlobalTypeId>,
        _type_indices: &IndexSet<TypeHash>,
    ) -> Self::CustomTypeKind<LocalTypeIndex> {
        match type_kind {
            ScryptoCustomTypeKind::Reference => ScryptoCustomTypeKind::Reference,
            ScryptoCustomTypeKind::GlobalAddress => ScryptoCustomTypeKind::GlobalAddress,
            ScryptoCustomTypeKind::LocalAddress => ScryptoCustomTypeKind::LocalAddress,
            ScryptoCustomTypeKind::PackageAddress => ScryptoCustomTypeKind::PackageAddress,
            ScryptoCustomTypeKind::ComponentAddress => ScryptoCustomTypeKind::ComponentAddress,
            ScryptoCustomTypeKind::ResourceAddress => ScryptoCustomTypeKind::ResourceAddress,

            ScryptoCustomTypeKind::Own => ScryptoCustomTypeKind::Own,
            ScryptoCustomTypeKind::Bucket => ScryptoCustomTypeKind::Bucket,
            ScryptoCustomTypeKind::Proof => ScryptoCustomTypeKind::Proof,
            ScryptoCustomTypeKind::Vault => ScryptoCustomTypeKind::Vault,
            ScryptoCustomTypeKind::KeyValueStore => ScryptoCustomTypeKind::KeyValueStore,

            ScryptoCustomTypeKind::Decimal => ScryptoCustomTypeKind::Decimal,
            ScryptoCustomTypeKind::PreciseDecimal => ScryptoCustomTypeKind::PreciseDecimal,
            ScryptoCustomTypeKind::NonFungibleLocalId => ScryptoCustomTypeKind::NonFungibleLocalId,
        }
    }

    fn resolve_well_known_type(
        well_known_index: u8,
    ) -> Option<&'static TypeData<Self::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>> {
        resolve_scrypto_well_known_type(well_known_index)
    }

    fn validate_type_kind(
        _context: &TypeValidationContext,
        type_kind: &SchemaCustomTypeKind<Self>,
    ) -> Result<(), SchemaValidationError> {
        match type_kind {
            ScryptoCustomTypeKind::Reference
            | ScryptoCustomTypeKind::GlobalAddress
            | ScryptoCustomTypeKind::LocalAddress
            | ScryptoCustomTypeKind::PackageAddress
            | ScryptoCustomTypeKind::ComponentAddress
            | ScryptoCustomTypeKind::ResourceAddress
            | ScryptoCustomTypeKind::Own
            | ScryptoCustomTypeKind::Bucket
            | ScryptoCustomTypeKind::Proof
            | ScryptoCustomTypeKind::Vault
            | ScryptoCustomTypeKind::KeyValueStore
            | ScryptoCustomTypeKind::Decimal
            | ScryptoCustomTypeKind::PreciseDecimal
            | ScryptoCustomTypeKind::NonFungibleLocalId => {
                // No validations
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
            ScryptoCustomTypeKind::Reference
            | ScryptoCustomTypeKind::GlobalAddress
            | ScryptoCustomTypeKind::LocalAddress
            | ScryptoCustomTypeKind::PackageAddress
            | ScryptoCustomTypeKind::ComponentAddress
            | ScryptoCustomTypeKind::ResourceAddress
            | ScryptoCustomTypeKind::Own
            | ScryptoCustomTypeKind::Bucket
            | ScryptoCustomTypeKind::Proof
            | ScryptoCustomTypeKind::Vault
            | ScryptoCustomTypeKind::KeyValueStore { .. }
            | ScryptoCustomTypeKind::Decimal
            | ScryptoCustomTypeKind::PreciseDecimal
            | ScryptoCustomTypeKind::NonFungibleLocalId => {
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
            ScryptoCustomTypeKind::Reference
            | ScryptoCustomTypeKind::GlobalAddress
            | ScryptoCustomTypeKind::LocalAddress
            | ScryptoCustomTypeKind::PackageAddress
            | ScryptoCustomTypeKind::ComponentAddress
            | ScryptoCustomTypeKind::ResourceAddress
            | ScryptoCustomTypeKind::Own
            | ScryptoCustomTypeKind::Bucket
            | ScryptoCustomTypeKind::Proof
            | ScryptoCustomTypeKind::Vault
            | ScryptoCustomTypeKind::KeyValueStore { .. }
            | ScryptoCustomTypeKind::Decimal
            | ScryptoCustomTypeKind::PreciseDecimal
            | ScryptoCustomTypeKind::NonFungibleLocalId => {
                // All these custom type kinds only support `SchemaTypeValidation::None`.
                // If they get to this point, they have been paired with some ScryptoCustomTypeValidation
                // - which isn't valid.
                return Err(SchemaValidationError::TypeValidationMismatch);
            }
        }
    }

    // FIXME: schema - this is insufficient, especially for address validation.
    fn custom_type_kind_matches_value_kind<L: SchemaTypeLink>(
        custom_type_kind: &Self::CustomTypeKind<L>,
        value_kind: ValueKind<Self::CustomValueKind>,
    ) -> bool {
        match custom_type_kind {
            ScryptoCustomTypeKind::Reference => matches!(
                value_kind,
                ValueKind::Custom(ScryptoCustomValueKind::Reference)
            ),
            ScryptoCustomTypeKind::GlobalAddress => matches!(
                value_kind,
                ValueKind::Custom(ScryptoCustomValueKind::Reference)
            ),
            ScryptoCustomTypeKind::LocalAddress => matches!(
                value_kind,
                ValueKind::Custom(ScryptoCustomValueKind::Reference)
            ),
            ScryptoCustomTypeKind::PackageAddress => matches!(
                value_kind,
                ValueKind::Custom(ScryptoCustomValueKind::Reference)
            ),
            ScryptoCustomTypeKind::ComponentAddress => matches!(
                value_kind,
                ValueKind::Custom(ScryptoCustomValueKind::Reference)
            ),
            ScryptoCustomTypeKind::ResourceAddress => matches!(
                value_kind,
                ValueKind::Custom(ScryptoCustomValueKind::Reference)
            ),
            ScryptoCustomTypeKind::Own => {
                matches!(value_kind, ValueKind::Custom(ScryptoCustomValueKind::Own))
            }
            ScryptoCustomTypeKind::Bucket => {
                matches!(value_kind, ValueKind::Custom(ScryptoCustomValueKind::Own))
            }
            ScryptoCustomTypeKind::Proof => {
                matches!(value_kind, ValueKind::Custom(ScryptoCustomValueKind::Own))
            }
            ScryptoCustomTypeKind::Vault => {
                matches!(value_kind, ValueKind::Custom(ScryptoCustomValueKind::Own))
            }
            ScryptoCustomTypeKind::KeyValueStore { .. } => {
                matches!(value_kind, ValueKind::Custom(ScryptoCustomValueKind::Own))
            }
            ScryptoCustomTypeKind::Decimal => matches!(
                value_kind,
                ValueKind::Custom(ScryptoCustomValueKind::Decimal)
            ),
            ScryptoCustomTypeKind::PreciseDecimal => matches!(
                value_kind,
                ValueKind::Custom(ScryptoCustomValueKind::PreciseDecimal)
            ),
            ScryptoCustomTypeKind::NonFungibleLocalId => matches!(
                value_kind,
                ValueKind::Custom(ScryptoCustomValueKind::NonFungibleLocalId)
            ),
        }
    }
}
