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
    TypeMetadataContainedDuplicateFieldNames,
    TypeMetadataFieldNameCountDoesNotMatchFieldCount,
    TypeMetadataContainedUnexpectedEnumVariants,
    TypeMetadataContainedUnexpectedNamedFields,
    TypeMetadataContainedWrongNumberOfVariants,
    TypeMetadataEnumNameIsRequired,
    TypeMetadataEnumVariantNameIsRequired,
    TypeMetadataForEnumIsNotEnumVariantChildNames,
    TypeMetadataHasMismatchingEnumDiscriminator,
    TypeMetadataContainedDuplicateEnumVariantNames,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rust::prelude::*;

    fn create_schema(type_data: Vec<TypeData<NoCustomTypeKind, LocalTypeIndex>>) -> BasicSchema {
        let mut type_kinds = vec![];
        let mut type_metadata = vec![];
        let mut type_validations = vec![];
        for type_data in type_data {
            let TypeData {
                kind,
                metadata,
                validation,
            } = type_data;
            type_kinds.push(kind);
            type_metadata.push(metadata);
            type_validations.push(validation);
        }
        BasicSchema {
            type_kinds,
            type_metadata,
            type_validations,
        }
    }

    #[test]
    pub fn duplicate_enum_variant_names_not_allowed() {
        let schema = create_schema(vec![TypeData::enum_variants(
            "TestEnum",
            btreemap![
                0 => TypeData::struct_with_unit_fields("VariantA"),
                1 => TypeData::struct_with_unit_fields("VariantB"),
                2 => TypeData::struct_with_unit_fields("VariantA"), // Repeat!
            ],
        )]);
        assert_eq!(
            validate_schema(&schema),
            Err(SchemaValidationError::TypeMetadataContainedDuplicateEnumVariantNames)
        );
    }
}
