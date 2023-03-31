use super::*;
use crate::schema::*;

pub fn validate_type_validation_with_type_kind<'a, E: CustomTypeExtension>(
    context: &TypeValidationContext,
    type_kind: &SchemaTypeKind<E>,
    type_validation: &SchemaTypeValidation<E>,
) -> Result<(), SchemaValidationError> {
    // It's always possible to opt into no additional validation.
    if matches!(type_validation, TypeValidation::None) {
        return Ok(());
    }
    match type_kind {
        TypeKind::Any | TypeKind::Tuple { .. } | TypeKind::Enum { .. } | TypeKind::Bool => {
            // Only None is supported - which is handled above
            return Err(SchemaValidationError::TypeValidationMismatch);
        }
        TypeKind::I8 => {
            let TypeValidation::I8(numeric_validation) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_numeric_validation(numeric_validation)?;
        }
        TypeKind::I16 => {
            let TypeValidation::I16(numeric_validation) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_numeric_validation(numeric_validation)?;
        }
        TypeKind::I32 => {
            let TypeValidation::I32(numeric_validation) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_numeric_validation(numeric_validation)?;
        }
        TypeKind::I64 => {
            let TypeValidation::I64(numeric_validation) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_numeric_validation(numeric_validation)?;
        }
        TypeKind::I128 => {
            let TypeValidation::I128(numeric_validation) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_numeric_validation(numeric_validation)?;
        }
        TypeKind::U8 => {
            let TypeValidation::U8(numeric_validation) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_numeric_validation(numeric_validation)?;
        }
        TypeKind::U16 => {
            let TypeValidation::U16(numeric_validation) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_numeric_validation(numeric_validation)?;
        }
        TypeKind::U32 => {
            let TypeValidation::U32(numeric_validation) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_numeric_validation(numeric_validation)?;
        }
        TypeKind::U64 => {
            let TypeValidation::U64(numeric_validation) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_numeric_validation(numeric_validation)?;
        }
        TypeKind::U128 => {
            let TypeValidation::U128(numeric_validation) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_numeric_validation(numeric_validation)?;
        }
        TypeKind::String => {
            let TypeValidation::String (length_validation ) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_length_validation(length_validation)?;
        }
        TypeKind::Array { .. } => {
            let TypeValidation::Array (length_validation ) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_length_validation(length_validation)?;
        }
        TypeKind::Map { .. } => {
            let TypeValidation::Map (length_validation ) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            validate_length_validation(length_validation)?;
        }
        TypeKind::Custom(custom_type_kind) => {
            let TypeValidation::Custom(custom_type_validation) = type_validation else {
                return Err(SchemaValidationError::TypeValidationMismatch);
            };
            E::validate_type_validation_with_type_kind(
                context,
                custom_type_kind,
                custom_type_validation,
            )?;
        }
    }
    Ok(())
}

fn validate_numeric_validation<T: Ord + Copy>(
    numeric_validation: &NumericValidation<T>,
) -> Result<(), SchemaValidationError> {
    if let Some(min) = numeric_validation.min {
        if let Some(max) = numeric_validation.max {
            if max < min {
                return Err(SchemaValidationError::TypeValidationNumericValidationInvalid);
            }
        }
    }
    Ok(())
}

fn validate_length_validation(
    length_validation: &LengthValidation,
) -> Result<(), SchemaValidationError> {
    if let Some(min) = length_validation.min {
        if let Some(max) = length_validation.max {
            if max < min {
                return Err(SchemaValidationError::TypeValidationLengthValidationInvalid);
            }
        }
    }
    Ok(())
}
