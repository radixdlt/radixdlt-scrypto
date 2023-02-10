use super::*;
use crate::rust::collections::*;
use crate::rust::fmt::Debug;
use crate::CustomValueKind;

pub trait CustomTypeKind<L: SchemaTypeLink>: Debug + Clone + PartialEq + Eq {
    type CustomValueKind: CustomValueKind;
    type CustomTypeExtension: CustomTypeExtension<
        CustomValueKind = Self::CustomValueKind,
        CustomTypeKind<L> = Self,
    >;
}

pub trait CustomTypeValidation: Debug + Clone + PartialEq + Eq {}

pub trait CustomTypeExtension: Debug + Clone + PartialEq + Eq {
    type CustomValueKind: CustomValueKind;
    type CustomTypeKind<L: SchemaTypeLink>: CustomTypeKind<
        L,
        CustomValueKind = Self::CustomValueKind,
        CustomTypeExtension = Self,
    >;
    type CustomTypeValidation: CustomTypeValidation;

    fn linearize_type_kind(
        type_kind: Self::CustomTypeKind<GlobalTypeId>,
        type_indices: &IndexSet<TypeHash>,
    ) -> Self::CustomTypeKind<LocalTypeIndex>;

    fn resolve_custom_well_known_type(
        well_known_index: u8,
    ) -> Option<TypeData<Self::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>>;

    fn validate_type_kind(
        context: &TypeValidationContext,
        type_kind: &SchemaCustomTypeKind<Self>,
    ) -> Result<(), SchemaValidationError>;

    fn validate_type_metadata_with_type_kind(
        context: &TypeValidationContext,
        type_kind: &SchemaCustomTypeKind<Self>,
        type_metadata: &TypeMetadata,
    ) -> Result<(), SchemaValidationError>;

    fn validate_type_validation_with_type_kind(
        context: &TypeValidationContext,
        type_kind: &SchemaCustomTypeKind<Self>,
        type_validation: &SchemaCustomTypeValidation<Self>,
    ) -> Result<(), SchemaValidationError>;
}
