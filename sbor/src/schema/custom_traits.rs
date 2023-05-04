use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

pub trait CustomTypeKind<L: SchemaTypeLink>: Debug + Clone + PartialEq + Eq {
    type CustomTypeValidation: CustomTypeValidation;
}

pub trait CustomTypeValidation: Debug + Clone + PartialEq + Eq {}

pub trait CustomSchema: Debug + Clone + Copy + PartialEq + Eq + 'static {
    type CustomTypeValidation: CustomTypeValidation;
    type CustomTypeKind<L: SchemaTypeLink>: CustomTypeKind<
        L,
        CustomTypeValidation = Self::CustomTypeValidation,
    >;

    fn linearize_type_kind(
        type_kind: Self::CustomTypeKind<GlobalTypeId>,
        type_indices: &IndexSet<TypeHash>,
    ) -> Self::CustomTypeKind<LocalTypeIndex>;

    // Note - each custom type extension should have its own cache
    fn resolve_well_known_type(
        well_known_index: u8,
    ) -> Option<&'static TypeData<Self::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>>;

    /// Used when validating schemas are self-consistent.
    ///
    /// Verifies if the custom type kind is valid within the schema context,
    /// e.g. to check if an offset is out of bounds.
    fn validate_custom_type_kind(
        context: &SchemaContext,
        custom_type_kind: &Self::CustomTypeKind<LocalTypeIndex>,
    ) -> Result<(), SchemaValidationError>;

    /// Used when validating schemas are self-consistent.
    ///
    /// Verifies if the custom type validation is appropriate for the custom type kind.
    /// Note that custom type validation can only be associated with custom type kind.
    fn validate_custom_type_validation(
        context: &SchemaContext,
        custom_type_kind: &Self::CustomTypeKind<LocalTypeIndex>,
        custom_type_validation: &Self::CustomTypeValidation,
    ) -> Result<(), SchemaValidationError>;

    /// Used when validating schemas are self-consistent.
    ///
    /// Verifies if the metadata is appropriate for the custom type kind.
    fn validate_type_metadata_with_custom_type_kind(
        context: &SchemaContext,
        custom_type_kind: &Self::CustomTypeKind<LocalTypeIndex>,
        type_metadata: &TypeMetadata,
    ) -> Result<(), SchemaValidationError>;

    fn empty_schema() -> &'static Schema<Self>;
}

pub trait CustomExtension: Debug + Clone + PartialEq + Eq + 'static {
    const MAX_DEPTH: usize;
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
        type_kind: &TypeKind<
            <Self::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeIndex>,
            LocalTypeIndex,
        >,
    ) -> bool;

    /// Used in the typed traverser
    ///
    /// This method is only called if custom_value_kind_matches_type_kind didn't apply.
    /// It's a fallback for any custom type kinds which should match against non-custom
    /// value kinds (in most cases there won't be any such cases).
    fn custom_type_kind_matches_non_custom_value_kind(
        schema: &Schema<Self::CustomSchema>,
        custom_type_kind: &<Self::CustomSchema as CustomSchema>::CustomTypeKind<LocalTypeIndex>,
        non_custom_value_kind: ValueKind<Self::CustomValueKind>,
    ) -> bool;
}
