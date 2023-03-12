use crate::rust::string::String;
use crate::*;

mod type_kind_validation;
mod type_metadata_validation;
mod type_validation_validation;

pub use type_kind_validation::*;
pub use type_metadata_validation::*;
pub use type_validation_validation::*;

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum SchemaValidationError {
    MetadataLengthMismatch,
    ValidationsLengthMismatch,
    TypeKindTupleTooLong { max_size: usize },
    TypeKindEnumVariantTooLong { max_size: usize },
    TypeKindInvalidSchemaLocalIndex,
    TypeKindInvalidWellKnownIndex,
    TypeMetadataContainedUnexpectedChildNames,
    TypeMetadataFieldNameCountDoesNotMatchFieldCount,
    TypeMetadataContainedUnexpectedEnumVariants,
    TypeMetadataContainedUnexpectedNamedFields,
    TypeMetadataContainedWrongNumberOfVariants,
    TypeMetadataForEnumIsNotEnumVariantChildNames,
    TypeMetadataHasMismatchingEnumDiscriminator,
    InvalidIdentName { message: String },
    TypeValidationMismatch,
    TypeValidationNumericValidationInvalid,
    TypeValidationLengthValidationInvalid,
}

pub fn validate_schema<E: CustomTypeExtension>(
    schema: &Schema<E>,
) -> Result<(), SchemaValidationError> {
    let Schema {
        type_kinds,
        type_metadata,
        type_validations,
    } = schema;

    let types_len = type_kinds.len();
    if type_metadata.len() != types_len {
        return Err(SchemaValidationError::MetadataLengthMismatch);
    }
    if type_validations.len() != types_len {
        return Err(SchemaValidationError::ValidationsLengthMismatch);
    }
    let context = TypeValidationContext {
        local_types_len: types_len,
    };

    for i in 0..types_len {
        validate_type_kind::<E>(&context, &type_kinds[i])?;
        validate_type_metadata_with_type_kind::<E>(&context, &type_kinds[i], &type_metadata[i])?;
        validate_type_validation_with_type_kind::<E>(
            &context,
            &type_kinds[i],
            &type_validations[i],
        )?;
    }
    Ok(())
}

pub struct TypeValidationContext {
    pub local_types_len: usize,
}
