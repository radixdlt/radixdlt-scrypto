use vec_traits::VecSbor;

use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

pub trait CustomTypeKind<L: SchemaTypeLink>: Debug + Clone + PartialEq + Eq {
    type CustomTypeValidation: CustomTypeValidation;
    type CustomTypeKindLabel: CustomTypeKindLabel;

    fn label(&self) -> Self::CustomTypeKindLabel;
}

pub trait CustomTypeKindLabel: Debug + Copy + Clone + PartialEq + Eq {
    fn name(&self) -> &'static str;
}

pub trait CustomTypeValidation: Debug + Clone + PartialEq + Eq {
    fn compare(base: &Self, compared: &Self) -> ValidationChange;
}

pub trait CustomSchema: Debug + Clone + Copy + PartialEq + Eq + 'static {
    type CustomTypeValidation: CustomTypeValidation + VecSbor<Self::DefaultCustomExtension>;
    type CustomLocalTypeKind: CustomTypeKind<
            LocalTypeId,
            CustomTypeValidation = Self::CustomTypeValidation,
            CustomTypeKindLabel = Self::CustomTypeKindLabel,
        > + VecSbor<Self::DefaultCustomExtension>;
    type CustomAggregatorTypeKind: CustomTypeKind<
        RustTypeId,
        CustomTypeValidation = Self::CustomTypeValidation,
        CustomTypeKindLabel = Self::CustomTypeKindLabel,
    >;
    type CustomTypeKindLabel: CustomTypeKindLabel;
    /// Should only be used for default encoding of a schema, where it's required.
    /// Typically you should start from a CustomExtension and not use this.
    type DefaultCustomExtension: ValidatableCustomExtension<(), CustomSchema = Self>;

    fn linearize_type_kind(
        type_kind: Self::CustomAggregatorTypeKind,
        type_indices: &IndexSet<TypeHash>,
    ) -> Self::CustomLocalTypeKind;

    // Note - each custom type extension should have its own cache
    fn resolve_well_known_type(
        well_known_id: WellKnownTypeId,
    ) -> Option<&'static LocalTypeData<Self>>;

    /// Used when validating schemas are self-consistent.
    ///
    /// Verifies if the custom type kind is valid within the schema context,
    /// e.g. to check if an offset is out of bounds.
    fn validate_custom_type_kind(
        context: &SchemaContext,
        custom_type_kind: &Self::CustomLocalTypeKind,
    ) -> Result<(), SchemaValidationError>;

    /// Used when validating schemas are self-consistent.
    ///
    /// Verifies if the custom type validation is appropriate for the custom type kind.
    /// Note that custom type validation can only be associated with custom type kind.
    fn validate_custom_type_validation(
        context: &SchemaContext,
        custom_type_kind: &Self::CustomLocalTypeKind,
        custom_type_validation: &Self::CustomTypeValidation,
    ) -> Result<(), SchemaValidationError>;

    /// Used when validating schemas are self-consistent.
    ///
    /// Verifies if the metadata is appropriate for the custom type kind.
    fn validate_type_metadata_with_custom_type_kind(
        context: &SchemaContext,
        custom_type_kind: &Self::CustomLocalTypeKind,
        type_metadata: &TypeMetadata,
    ) -> Result<(), SchemaValidationError>;

    fn empty_schema() -> &'static Schema<Self>;
}

pub trait CustomExtension: Debug + Clone + PartialEq + Eq + 'static {
    const PAYLOAD_PREFIX: u8;

    type CustomValueKind: CustomValueKind;

    type CustomTraversal: CustomTraversal<CustomValueKind = Self::CustomValueKind>;

    type CustomSchema: CustomSchema;

    /// Used in the typed traverser
    ///
    /// This method is only called if the type_kind is not "Any"
    fn custom_value_kind_matches_type_kind(
        schema: &Schema<Self::CustomSchema>,
        custom_value_kind: Self::CustomValueKind,
        type_kind: &LocalTypeKind<Self::CustomSchema>,
    ) -> bool;

    /// Used in the typed traverser
    ///
    /// This method is only called if custom_value_kind_matches_type_kind didn't apply.
    /// It's a fallback for any custom type kinds which should match against non-custom
    /// value kinds (in most cases there won't be any such cases).
    fn custom_type_kind_matches_non_custom_value_kind(
        schema: &Schema<Self::CustomSchema>,
        custom_type_kind: &<Self::CustomSchema as CustomSchema>::CustomLocalTypeKind,
        non_custom_value_kind: ValueKind<Self::CustomValueKind>,
    ) -> bool;
}
