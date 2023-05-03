use crate::rust::prelude::*;
use crate::traversal::*;
use crate::*;

pub trait CustomTypeKind<L: SchemaTypeLink>: Debug + Clone + PartialEq + Eq {
    type CustomTypeValidation: CustomTypeValidation;
}

pub trait CustomTypeValidation: Debug + Clone + PartialEq + Eq {}

pub trait CustomTypeExtension: Debug + Clone + PartialEq + Eq + 'static {
    const MAX_DEPTH: usize;
    const PAYLOAD_PREFIX: u8;
    type CustomValueKind: CustomValueKind;
    type CustomTypeValidation: CustomTypeValidation;
    type CustomTypeKind<L: SchemaTypeLink>: CustomTypeKind<
        L,
        CustomTypeValidation = Self::CustomTypeValidation,
    >;
    type CustomTraversal: CustomTraversal<CustomValueKind = Self::CustomValueKind>;

    fn linearize_type_kind(
        type_kind: Self::CustomTypeKind<GlobalTypeId>,
        type_indices: &IndexSet<TypeHash>,
    ) -> Self::CustomTypeKind<LocalTypeIndex>;

    // Note - each custom type extension should have its own cache
    fn resolve_well_known_type(
        well_known_index: u8,
    ) -> Option<&'static TypeData<Self::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>>;

    /// Verifies if the custom type kind is valid within the schema context,
    /// e.g. to check if an offset is out of bounds.
    fn validate_custom_type_kind(
        context: &SchemaContext,
        custom_type_kind: &Self::CustomTypeKind<LocalTypeIndex>,
    ) -> Result<(), SchemaValidationError>;

    /// Verifies if the custom type validation is appropriate for the custom type kind.
    /// Note that custom type validation can only be associated with custom type kind.
    fn validate_custom_type_validation(
        context: &SchemaContext,
        custom_type_kind: &Self::CustomTypeKind<LocalTypeIndex>,
        custom_type_validation: &Self::CustomTypeValidation,
    ) -> Result<(), SchemaValidationError>;

    /// Verifies if the metadata is appropriate for the custom type kind.
    fn validate_type_metadata_with_custom_type_kind(
        context: &SchemaContext,
        custom_type_kind: &Self::CustomTypeKind<LocalTypeIndex>,
        type_metadata: &TypeMetadata,
    ) -> Result<(), SchemaValidationError>;

    fn custom_type_kind_matches_value_kind<L: SchemaTypeLink>(
        custom_type_kind: &Self::CustomTypeKind<L>,
        value_kind: ValueKind<Self::CustomValueKind>,
    ) -> bool;

    fn empty_schema() -> &'static Schema<Self>;
}
