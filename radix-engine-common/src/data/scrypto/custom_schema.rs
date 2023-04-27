use super::*;
use crate::types::PackageAddress;
use crate::*;
use sbor::rust::collections::*;
use sbor::*;

pub type ScryptoTypeKind<L> = TypeKind<ScryptoCustomValueKind, ScryptoCustomTypeKind, L>;
pub type ScryptoSchema = Schema<ScryptoCustomTypeExtension>;
pub type ScryptoTypeData<L> = TypeData<ScryptoCustomTypeKind, L>;

/// A schema for the values that a codec can decode / views as valid
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoSbor)]
pub enum ScryptoCustomTypeKind {
    Reference,
    Own,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoSbor)]
pub enum ScryptoCustomTypeValidation {
    Reference(ReferenceValidation),
    Own(OwnValidation),
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoSbor)]
pub enum ReferenceValidation {
    IsGlobal,
    IsGlobalPackage,
    IsGlobalComponent,
    IsGlobalResource,
    IsLocal,
    IsTypedObject(PackageAddress, String),
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoSbor)]
pub enum OwnValidation {
    IsBucket,
    IsProof,
    IsVault,
    IsKeyValueStore,
    IsTypedObject(PackageAddress, String),
}

impl<L: SchemaTypeLink> CustomTypeKind<L> for ScryptoCustomTypeKind {
    type CustomValueKind = ScryptoCustomValueKind;
    type CustomTypeValidation = ScryptoCustomTypeValidation;
}

impl CustomTypeValidation for ScryptoCustomTypeValidation {}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ScryptoCustomTypeExtension {}

lazy_static::lazy_static! {
    static ref EMPTY_SCHEMA: Schema<ScryptoCustomTypeExtension> = {
        Schema::empty()
    };
}

impl CustomTypeExtension for ScryptoCustomTypeExtension {
    const MAX_DEPTH: usize = SCRYPTO_SBOR_V1_MAX_DEPTH;
    const PAYLOAD_PREFIX: u8 = SCRYPTO_SBOR_V1_PAYLOAD_PREFIX;

    type CustomValueKind = ScryptoCustomValueKind;
    type CustomTypeKind<L: SchemaTypeLink> = ScryptoCustomTypeKind;
    type CustomTraversal = ScryptoCustomTraversal;
    type CustomTypeValidation = ScryptoCustomTypeValidation;

    fn linearize_type_kind(
        type_kind: Self::CustomTypeKind<GlobalTypeId>,
        _type_indices: &IndexSet<TypeHash>,
    ) -> Self::CustomTypeKind<LocalTypeIndex> {
        match type_kind {
            ScryptoCustomTypeKind::Reference => ScryptoCustomTypeKind::Reference,
            ScryptoCustomTypeKind::Own => ScryptoCustomTypeKind::Own,
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

    fn validate_custom_type_kind(
        _context: &SchemaContext,
        type_kind: &Self::CustomTypeKind<LocalTypeIndex>,
    ) -> Result<(), SchemaValidationError> {
        match type_kind {
            ScryptoCustomTypeKind::Reference
            | ScryptoCustomTypeKind::Own
            | ScryptoCustomTypeKind::Decimal
            | ScryptoCustomTypeKind::PreciseDecimal
            | ScryptoCustomTypeKind::NonFungibleLocalId => {
                // No validations
            }
        }
        Ok(())
    }

    fn validate_type_metadata_with_custom_type_kind(
        _: &SchemaContext,
        type_kind: &Self::CustomTypeKind<LocalTypeIndex>,
        type_metadata: &TypeMetadata,
    ) -> Result<(), SchemaValidationError> {
        // Even though they all map to the same thing, we keep the explicit match statement so that
        // we will have to explicitly check this when we add a new `ScryptoCustomTypeKind`
        match type_kind {
            ScryptoCustomTypeKind::Reference
            | ScryptoCustomTypeKind::Own
            | ScryptoCustomTypeKind::Decimal
            | ScryptoCustomTypeKind::PreciseDecimal
            | ScryptoCustomTypeKind::NonFungibleLocalId => {
                validate_childless_metadata(type_metadata)?;
            }
        }
        Ok(())
    }

    fn validate_custom_type_validation(
        _context: &SchemaContext,
        custom_type_kind: &Self::CustomTypeKind<LocalTypeIndex>,
        custom_type_validation: &Self::CustomTypeValidation,
    ) -> Result<(), SchemaValidationError> {
        match custom_type_kind {
            ScryptoCustomTypeKind::Reference => {
                if let ScryptoCustomTypeValidation::Reference(_) = custom_type_validation {
                    Ok(())
                } else {
                    return Err(SchemaValidationError::TypeValidationMismatch);
                }
            }
            ScryptoCustomTypeKind::Own => {
                if let ScryptoCustomTypeValidation::Own(_) = custom_type_validation {
                    Ok(())
                } else {
                    return Err(SchemaValidationError::TypeValidationMismatch);
                }
            }
            ScryptoCustomTypeKind::Decimal
            | ScryptoCustomTypeKind::PreciseDecimal
            | ScryptoCustomTypeKind::NonFungibleLocalId => {
                // All these custom type kinds only support `SchemaTypeValidation::None`.
                // If they get to this point, they have been paired with some ScryptoCustomTypeValidation
                // - which isn't valid.
                return Err(SchemaValidationError::TypeValidationMismatch);
            }
        }
    }

    fn custom_type_kind_matches_value_kind<L: SchemaTypeLink>(
        custom_type_kind: &Self::CustomTypeKind<L>,
        value_kind: ValueKind<Self::CustomValueKind>,
    ) -> bool {
        match custom_type_kind {
            ScryptoCustomTypeKind::Reference => matches!(
                value_kind,
                ValueKind::Custom(ScryptoCustomValueKind::Reference)
            ),
            ScryptoCustomTypeKind::Own => {
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

    fn empty_schema() -> &'static Schema<Self> {
        &EMPTY_SCHEMA
    }
}
