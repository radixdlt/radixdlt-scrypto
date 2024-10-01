use crate::internal_prelude::*;

pub type ScryptoTypeKind<L> = TypeKind<ScryptoCustomTypeKind, L>;
pub type ScryptoLocalTypeKind = LocalTypeKind<ScryptoCustomSchema>;
pub type ScryptoAggregatorTypeKind = AggregatorTypeKind<ScryptoCustomSchema>;
pub type VersionedScryptoSchema = VersionedSchema<ScryptoCustomSchema>;
pub type ScryptoSingleTypeSchema = SingleTypeSchema<ScryptoCustomSchema>;
pub type ScryptoTypeCollectionSchema = TypeCollectionSchema<ScryptoCustomSchema>;
pub type ScryptoSchema = Schema<ScryptoCustomSchema>;
pub type ScryptoTypeData<L> = TypeData<ScryptoCustomTypeKind, L>;
pub type ScryptoLocalTypeData = LocalTypeData<ScryptoCustomSchema>;
pub type ScryptoAggregatorTypeData = AggregatorTypeData<ScryptoCustomSchema>;
pub type ScryptoTypeValidation = TypeValidation<ScryptoCustomTypeValidation>;
pub type ScryptoTypeAggregator = TypeAggregator<ScryptoCustomTypeKind>;

pub trait ScryptoCheckedFixedSchema: CheckedFixedSchema<ScryptoCustomSchema> {}
impl<T: CheckedFixedSchema<ScryptoCustomSchema>> ScryptoCheckedFixedSchema for T {}

pub trait ScryptoCheckedBackwardsCompatibleSchema:
    CheckedBackwardsCompatibleSchema<ScryptoCustomSchema>
{
}
impl<T: CheckedBackwardsCompatibleSchema<ScryptoCustomSchema>>
    ScryptoCheckedBackwardsCompatibleSchema for T
{
}

/// A schema for the values that a codec can decode / views as valid
#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoSbor)]
pub enum ScryptoCustomTypeKind {
    Reference,
    Own,
    Decimal,
    PreciseDecimal,
    NonFungibleLocalId,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, ManifestSbor, ScryptoSbor)]
pub enum ScryptoCustomTypeKindLabel {
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

impl ReferenceValidation {
    fn compare(base: &Self, compared: &Self) -> ValidationChange {
        match (base, compared) {
            (base, compared) if base == compared => ValidationChange::Unchanged,
            (ReferenceValidation::IsGlobal, compared) if compared.requires_global() => {
                ValidationChange::Strengthened
            }
            (base, ReferenceValidation::IsGlobal) if base.requires_global() => {
                ValidationChange::Weakened
            }
            (ReferenceValidation::IsInternal, compared) if compared.requires_internal() => {
                ValidationChange::Strengthened
            }
            (base, ReferenceValidation::IsInternal) if base.requires_internal() => {
                ValidationChange::Weakened
            }
            (_, _) => ValidationChange::Incomparable,
        }
    }

    fn requires_global(&self) -> bool {
        match self {
            ReferenceValidation::IsGlobal => true,
            ReferenceValidation::IsGlobalPackage => true,
            ReferenceValidation::IsGlobalComponent => true,
            ReferenceValidation::IsGlobalResourceManager => true,
            ReferenceValidation::IsGlobalTyped(_, _) => true,
            ReferenceValidation::IsInternal => false,
            ReferenceValidation::IsInternalTyped(_, _) => false,
        }
    }

    fn requires_internal(&self) -> bool {
        match self {
            ReferenceValidation::IsGlobal => false,
            ReferenceValidation::IsGlobalPackage => false,
            ReferenceValidation::IsGlobalComponent => false,
            ReferenceValidation::IsGlobalResourceManager => false,
            ReferenceValidation::IsGlobalTyped(_, _) => false,
            ReferenceValidation::IsInternal => true,
            ReferenceValidation::IsInternalTyped(_, _) => true,
        }
    }
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
    fn compare(base: &Self, compared: &Self) -> ValidationChange {
        // This is strictly a little hard - if we get issues, we may wish to match
        // IsTypedObject(ResourcePackage, "FungibleBucket") as a strengthening of IsBucket and so on.
        if base == compared {
            ValidationChange::Unchanged
        } else {
            ValidationChange::Incomparable
        }
    }

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
    type CustomTypeKindLabel = ScryptoCustomTypeKindLabel;

    fn label(&self) -> Self::CustomTypeKindLabel {
        match self {
            ScryptoCustomTypeKind::Reference => ScryptoCustomTypeKindLabel::Reference,
            ScryptoCustomTypeKind::Own => ScryptoCustomTypeKindLabel::Own,
            ScryptoCustomTypeKind::Decimal => ScryptoCustomTypeKindLabel::Decimal,
            ScryptoCustomTypeKind::PreciseDecimal => ScryptoCustomTypeKindLabel::PreciseDecimal,
            ScryptoCustomTypeKind::NonFungibleLocalId => {
                ScryptoCustomTypeKindLabel::NonFungibleLocalId
            }
        }
    }
}

impl CustomTypeKindLabel for ScryptoCustomTypeKindLabel {
    fn name(&self) -> &'static str {
        match self {
            ScryptoCustomTypeKindLabel::Reference => "Reference",
            ScryptoCustomTypeKindLabel::Own => "Own",
            ScryptoCustomTypeKindLabel::Decimal => "Decimal",
            ScryptoCustomTypeKindLabel::PreciseDecimal => "PreciseDecimal",
            ScryptoCustomTypeKindLabel::NonFungibleLocalId => "NonFungibleLocalId",
        }
    }
}

impl CustomTypeValidation for ScryptoCustomTypeValidation {
    fn compare(base: &Self, compared: &Self) -> ValidationChange {
        match (base, compared) {
            (
                ScryptoCustomTypeValidation::Reference(base),
                ScryptoCustomTypeValidation::Reference(compared),
            ) => ReferenceValidation::compare(base, compared),
            (ScryptoCustomTypeValidation::Reference(_), ScryptoCustomTypeValidation::Own(_)) => {
                ValidationChange::Incomparable
            }
            (ScryptoCustomTypeValidation::Own(_), ScryptoCustomTypeValidation::Reference(_)) => {
                ValidationChange::Incomparable
            }
            (
                ScryptoCustomTypeValidation::Own(base),
                ScryptoCustomTypeValidation::Own(compared),
            ) => OwnValidation::compare(base, compared),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub struct ScryptoCustomSchema {}

lazy_static::lazy_static! {
    static ref EMPTY_SCHEMA: Schema<ScryptoCustomSchema> = {
        Schema::empty()
    };
}

impl CustomSchema for ScryptoCustomSchema {
    type CustomLocalTypeKind = ScryptoCustomTypeKind;
    type CustomAggregatorTypeKind = ScryptoCustomTypeKind;
    type CustomTypeKindLabel = ScryptoCustomTypeKindLabel;
    type CustomTypeValidation = ScryptoCustomTypeValidation;
    type DefaultCustomExtension = ScryptoCustomExtension;

    fn linearize_type_kind(
        type_kind: Self::CustomLocalTypeKind,
        _type_indices: &IndexSet<TypeHash>,
    ) -> Self::CustomAggregatorTypeKind {
        type_kind
    }

    fn resolve_well_known_type(
        well_known_id: WellKnownTypeId,
    ) -> Option<&'static LocalTypeData<Self>> {
        resolve_scrypto_well_known_type(well_known_id)
    }

    fn validate_custom_type_kind(
        _context: &SchemaContext,
        type_kind: &Self::CustomLocalTypeKind,
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
        type_kind: &Self::CustomLocalTypeKind,
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
        custom_type_kind: &Self::CustomLocalTypeKind,
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

pub trait HasSchemaHash {
    fn generate_schema_hash(&self) -> SchemaHash;
}

impl HasSchemaHash for VersionedScryptoSchema {
    fn generate_schema_hash(&self) -> SchemaHash {
        SchemaHash::from(hash(scrypto_encode(self).unwrap()))
    }
}

pub fn replace_self_package_address(
    schema: &mut VersionedScryptoSchema,
    package_address: PackageAddress,
) {
    for type_validation in &mut schema.v1_mut().type_validations {
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
