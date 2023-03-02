use super::*;
use crate::rust::prelude::*;
use crate::schema::*;

pub fn validate_type_metadata_with_type_kind<'a, E: CustomTypeExtension>(
    context: &TypeValidationContext,
    type_kind: &SchemaTypeKind<E>,
    type_metadata: &TypeMetadata,
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
        | TypeKind::String
        | TypeKind::Array { .. }
        | TypeKind::Map { .. } => {
            validate_childless_metadata(type_metadata)?;
        }
        TypeKind::Tuple { field_types } => {
            validate_fields_metadata(type_metadata, field_types.len())?;
        }
        TypeKind::Enum { variants } => {
            validate_enum_metadata(type_metadata, variants)?;
        }
        TypeKind::Custom(custom_type_kind) => {
            E::validate_type_metadata_with_type_kind(context, custom_type_kind, type_metadata)?;
        }
    }

    Ok(())
}

pub fn validate_childless_metadata(
    type_metadata: &TypeMetadata,
) -> Result<(), SchemaValidationError> {
    validate_type_name(type_metadata.type_name.as_ref())?;

    if !matches!(type_metadata.children, Children::None) {
        return Err(SchemaValidationError::TypeMetadataContainedUnexpectedChildNames);
    }
    return Ok(());
}

pub fn validate_fields_metadata(
    type_metadata: &TypeMetadata,
    field_count: usize,
) -> Result<(), SchemaValidationError> {
    validate_type_name(type_metadata.type_name.as_ref())?;
    validate_fields_child_names(&type_metadata.children, field_count)?;
    Ok(())
}

pub fn validate_fields_child_names(
    child_names: &Children,
    field_count: usize,
) -> Result<(), SchemaValidationError> {
    match child_names {
        Children::None => {
            if field_count != 0 {
                return Err(SchemaValidationError::TypeMetadataContainedWrongNumberOfChildren);
            }
        }
        Children::UnnamedFields => {
            if field_count == 0 {
                return Err(SchemaValidationError::TypeMetadataContainedWrongNumberOfChildren);
            }
        }
        Children::NamedFields(fields_metadata) => {
            if field_count == 0 || fields_metadata.len() != field_count {
                return Err(SchemaValidationError::TypeMetadataContainedWrongNumberOfChildren);
            }
            for field_metadata in fields_metadata.iter() {
                let FieldMetadata { field_name } = field_metadata;
                validate_field_name(field_name)?;
            }
        }
        Children::EnumVariants(_) => {
            return Err(SchemaValidationError::TypeMetadataForFieldsContainedEnumVariantChildNames)
        }
    }
    Ok(())
}

pub fn validate_enum_metadata(
    type_metadata: &TypeMetadata,
    variants: &BTreeMap<u8, Vec<LocalTypeIndex>>,
) -> Result<(), SchemaValidationError> {
    let TypeMetadata {
        type_name,
        children,
    } = type_metadata;
    validate_type_name(type_name.as_ref())?;

    match &children {
        Children::None | Children::UnnamedFields | Children::NamedFields(_) => {
            return Err(SchemaValidationError::TypeMetadataForEnumIsNotEnumVariantChildNames)
        }
        Children::EnumVariants(variants_metadata) => {
            if variants_metadata.len() != variants.len() {
                return Err(SchemaValidationError::TypeMetadataContainedWrongNumberOfChildren);
            }
            for (discriminator, variant_metadata) in variants_metadata.iter() {
                let Some(child_types) = variants.get(discriminator) else {
                    return Err(SchemaValidationError::TypeMetadataHasMismatchingEnumDiscriminator)
                };

                let TypeMetadata {
                    type_name,
                    children,
                } = variant_metadata;
                validate_enum_variant_name(type_name.as_ref())?;
                validate_fields_child_names(children, child_types.len())?;
            }
        }
    }
    Ok(())
}

fn validate_type_name(name: &str) -> Result<(), SchemaValidationError> {
    validate_ident("type name", name)
}

fn validate_enum_variant_name(name: &str) -> Result<(), SchemaValidationError> {
    validate_ident("enum variant", name)
}

fn validate_field_name(name: &str) -> Result<(), SchemaValidationError> {
    validate_ident("field name", name)
}

fn validate_ident(ident_name: &str, name: &str) -> Result<(), SchemaValidationError> {
    if name.len() == 0 {
        return Err(SchemaValidationError::InvalidIdentName {
            message: format!("Ident {} cannot be empty", ident_name),
        });
    }

    if name.len() > 100 {
        return Err(SchemaValidationError::InvalidIdentName {
            message: format!("Ident {} cannot be more than 100 characters", ident_name),
        });
    }

    let first_char = name.chars().next().unwrap();
    if !matches!(first_char, 'A'..='Z' | 'a'..='z') {
        return Err(SchemaValidationError::InvalidIdentName {
            message: format!(
                "At present, {} idents must match [A-Za-z][0-9A-Za-z_]{{0,99}}",
                ident_name
            ),
        });
    }

    for char in name.chars() {
        if !matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_') {
            return Err(SchemaValidationError::InvalidIdentName {
                message: format!(
                    "At present, {} idents must match [A-Za-z][0-9A-Za-z_]{{0,99}}",
                    ident_name
                ),
            });
        }
    }
    Ok(())
}
