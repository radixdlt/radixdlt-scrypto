use crate::internal_prelude::*;

pub type ScryptoTypeKind<L> = TypeKind<ScryptoCustomTypeKind, L>;
pub type ScryptoSchema = Schema<ScryptoCustomSchema>;
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
    IsGlobalResourceManager,
    IsGlobalTyped(Option<PackageAddress>, String),
    IsInternal,
    IsInternalTyped(Option<PackageAddress>, String),
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoSbor)]
pub enum OwnValidation {
    IsBucket,
    IsProof,
    IsVault,
    IsKeyValueStore,
    IsGlobalAddressReservation,
    IsTypedObject(Option<PackageAddress>, String),
}

impl OwnValidation {
    pub fn could_match_manifest_bucket(&self) -> bool {
        match self {
            OwnValidation::IsBucket => true,
            OwnValidation::IsProof => false,
            OwnValidation::IsVault => false,
            OwnValidation::IsKeyValueStore => false,
            OwnValidation::IsGlobalAddressReservation => false,
            // Hard to validate without knowing package addresses from engine, assume fine
            OwnValidation::IsTypedObject(_, _) => true,
        }
    }

    pub fn could_match_manifest_proof(&self) -> bool {
        match self {
            OwnValidation::IsBucket => false,
            OwnValidation::IsProof => true,
            OwnValidation::IsVault => false,
            OwnValidation::IsKeyValueStore => false,
            OwnValidation::IsGlobalAddressReservation => false,
            // Hard to validate without knowing package addresses from engine, assume fine
            OwnValidation::IsTypedObject(_, _) => true,
        }
    }

    pub fn could_match_manifest_address_reservation(&self) -> bool {
        match self {
            OwnValidation::IsBucket => false,
            OwnValidation::IsProof => false,
            OwnValidation::IsVault => false,
            OwnValidation::IsKeyValueStore => false,
            OwnValidation::IsGlobalAddressReservation => true,
            OwnValidation::IsTypedObject(_, _) => false,
        }
    }
}

impl ReferenceValidation {
    pub fn could_match_manifest_address(&self) -> bool {
        match self {
            ReferenceValidation::IsGlobal => true,
            ReferenceValidation::IsGlobalPackage => true,
            ReferenceValidation::IsGlobalComponent => true,
            ReferenceValidation::IsGlobalResourceManager => true,
            ReferenceValidation::IsGlobalTyped(_, _) => true,
            ReferenceValidation::IsInternal => true,
            ReferenceValidation::IsInternalTyped(_, _) => true,
        }
    }
}

impl<L: SchemaTypeLink> CustomTypeKind<L> for ScryptoCustomTypeKind {
    type CustomTypeValidation = ScryptoCustomTypeValidation;
}

impl CustomTypeValidation for ScryptoCustomTypeValidation {}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ScryptoCustomSchema {}

lazy_static::lazy_static! {
    static ref EMPTY_SCHEMA: Schema<ScryptoCustomSchema> = {
        Schema::empty()
    };
}

impl CustomSchema for ScryptoCustomSchema {
    type CustomTypeKind<L: SchemaTypeLink> = ScryptoCustomTypeKind;
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

    fn empty_schema() -> &'static Schema<Self> {
        &EMPTY_SCHEMA
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ScryptoCustomExtension {}

impl CustomExtension for ScryptoCustomExtension {
    const MAX_DEPTH: usize = SCRYPTO_SBOR_V1_MAX_DEPTH;
    const PAYLOAD_PREFIX: u8 = SCRYPTO_SBOR_V1_PAYLOAD_PREFIX;

    type CustomValueKind = ScryptoCustomValueKind;
    type CustomTraversal = ScryptoCustomTraversal;
    type CustomSchema = ScryptoCustomSchema;

    fn custom_value_kind_matches_type_kind(
        _: &Schema<Self::CustomSchema>,
        custom_value_kind: Self::CustomValueKind,
        type_kind: &TypeKind<
            <Self::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeIndex>,
            LocalTypeIndex,
        >,
    ) -> bool {
        match custom_value_kind {
            ScryptoCustomValueKind::Reference => matches!(
                type_kind,
                TypeKind::Custom(ScryptoCustomTypeKind::Reference)
            ),
            ScryptoCustomValueKind::Own => {
                matches!(type_kind, TypeKind::Custom(ScryptoCustomTypeKind::Own))
            }
            ScryptoCustomValueKind::Decimal => {
                matches!(type_kind, TypeKind::Custom(ScryptoCustomTypeKind::Decimal))
            }
            ScryptoCustomValueKind::PreciseDecimal => matches!(
                type_kind,
                TypeKind::Custom(ScryptoCustomTypeKind::PreciseDecimal)
            ),
            ScryptoCustomValueKind::NonFungibleLocalId => matches!(
                type_kind,
                TypeKind::Custom(ScryptoCustomTypeKind::NonFungibleLocalId)
            ),
        }
    }

    fn custom_type_kind_matches_non_custom_value_kind(
        _: &Schema<Self::CustomSchema>,
        _: &<Self::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeIndex>,
        _: ValueKind<Self::CustomValueKind>,
    ) -> bool {
        // It's not possible for a custom type kind to match a non-custom value kind
        false
    }
}

pub fn replace_self_package_address(schema: &mut ScryptoSchema, package_address: PackageAddress) {
    for type_validation in &mut schema.type_validations {
        match type_validation {
            TypeValidation::Custom(ScryptoCustomTypeValidation::Own(
                OwnValidation::IsTypedObject(package, _),
            ))
            | TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsGlobalTyped(package, _),
            ))
            | TypeValidation::Custom(ScryptoCustomTypeValidation::Reference(
                ReferenceValidation::IsInternalTyped(package, _),
            )) => {
                if package.is_none() {
                    *package = Some(package_address)
                }
            }
            _ => {}
        }
    }
}
