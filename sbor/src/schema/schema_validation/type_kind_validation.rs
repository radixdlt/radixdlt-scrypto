use super::*;
use crate::schema::*;

pub const MAX_NUMBER_OF_FIELDS: usize = 1024;

pub fn validate_type_kind<'a, S: CustomSchema>(
    context: &SchemaContext,
    type_kind: &LocalTypeKind<S>,
) -> Result<(), SchemaValidationError> {
    match type_kind {
        TypeKind::Any
        | TypeKind::Bool
        | TypeKind::I8
        | TypeKind::I16
        | TypeKind::I32
        | TypeKind::I64
        | TypeKind::I128
        | TypeKind::U8
        | TypeKind::U16
        | TypeKind::U32
        | TypeKind::U64
        | TypeKind::U128
        | TypeKind::String => {
            // Nothing to check
        }
        TypeKind::Array { element_type } => {
            validate_index::<S>(context, element_type)?;
        }
        TypeKind::Tuple { field_types } => {
            if field_types.len() > MAX_NUMBER_OF_FIELDS {
                return Err(SchemaValidationError::TypeKindTupleTooLong {
                    max_size: MAX_NUMBER_OF_FIELDS,
                });
            }
            for field_type in field_types.iter() {
                validate_index::<S>(context, field_type)?;
            }
        }
        TypeKind::Enum { variants } => {
            for (_, field_types) in variants.iter() {
                if field_types.len() > MAX_NUMBER_OF_FIELDS {
                    return Err(SchemaValidationError::TypeKindEnumVariantTooLong {
                        max_size: MAX_NUMBER_OF_FIELDS,
                    });
                }
                for field_type in field_types.iter() {
                    validate_index::<S>(context, field_type)?;
                }
            }
        }
        TypeKind::Map {
            key_type,
            value_type,
        } => {
            validate_index::<S>(context, key_type)?;
            validate_index::<S>(context, value_type)?;
        }
        TypeKind::Custom(custom_type_kind) => {
            S::validate_custom_type_kind(context, custom_type_kind)?;
        }
    }

    Ok(())
}

pub fn validate_index<S: CustomSchema>(
    context: &SchemaContext,
    type_id: &LocalTypeId,
) -> Result<(), SchemaValidationError> {
    match type_id {
        LocalTypeId::WellKnown(well_known_index) => {
            if S::resolve_well_known_type(*well_known_index).is_none() {
                return Err(SchemaValidationError::TypeKindInvalidWellKnownIndex);
            }
        }
        LocalTypeId::SchemaLocalIndex(schema_local_index) => {
            if *schema_local_index >= context.local_types_len {
                return Err(SchemaValidationError::TypeKindInvalidSchemaLocalIndex);
            }
        }
    }
    Ok(())
}
