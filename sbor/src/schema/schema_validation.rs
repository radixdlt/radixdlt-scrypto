use crate::rust::collections::BTreeMap;
use crate::rust::collections::BTreeSet;
use crate::rust::string::String;
use crate::rust::vec::Vec;
use crate::*;

pub enum SchemaValidationError {
    MetadataLengthMismatch,
    DuplicateTypeHash,
    TypeKindInvalidSchemaLocalIndex,
    TypeKindInvalidWellKnownIndex,
    TypeMetadataContainedUnexpectedChildNames,
    TypeMetadataContainedWrongNumberOfChildNames,
    TypeMetadataForFieldsContainedEnumVariantChildNames,
    TypeMetadataForEnumIsNotEnumVariantChildNames,
    TypeMetadataHasMismatchingEnumDiscriminator,
    InvalidTypeName { message: String },
    InvalidFieldName { message: String },
    InvalidEnumVariantName { message: String },
}

pub fn validate_schema<E: CustomTypeExtension>(
    schema: &Schema<E>,
) -> Result<(), SchemaValidationError> {
    let Schema {
        type_kinds,
        type_metadata,
    } = schema;

    let types_len = type_kinds.len();
    if type_metadata.len() != types_len {
        return Err(SchemaValidationError::MetadataLengthMismatch);
    }

    let unique_type_hashes = type_metadata
        .iter()
        .map(|m| m.type_hash)
        .collect::<BTreeSet<_>>()
        .len();

    if unique_type_hashes != types_len {
        return Err(SchemaValidationError::DuplicateTypeHash);
    }

    let context = TypeValidationContext {
        local_types_len: types_len,
    };

    for (type_kind, type_metadata) in type_kinds.iter().zip(type_metadata.iter()) {
        validate_schema_type(
            &context,
            SchemaTypeValidationRequest::<E> {
                type_kind,
                novel_type_metadata: type_metadata,
            },
        )?;
    }
    Ok(())
}

struct SchemaTypeValidationRequest<'a, E: CustomTypeExtension> {
    type_kind: &'a SchemaTypeKind<E>,
    novel_type_metadata: &'a NovelTypeMetadata,
}

pub struct TypeValidationContext {
    pub local_types_len: usize,
}

pub struct CustomSchemaTypeValidationRequest<'a, E: CustomTypeExtension> {
    pub custom_type_kind: &'a SchemaCustomTypeKind<E>,
    pub type_metadata: &'a TypeMetadata,
}

fn validate_schema_type<'a, E: CustomTypeExtension>(
    context: &TypeValidationContext,
    request: SchemaTypeValidationRequest<'a, E>,
) -> Result<(), SchemaValidationError> {
    let SchemaTypeValidationRequest {
        type_kind,
        novel_type_metadata,
    } = request;

    let NovelTypeMetadata {
        type_hash: _, // We have already validated that the type hashes are distinct at the parent level
        type_metadata,
    } = novel_type_metadata;

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
            validate_childless_metadata(type_metadata)?;
        }
        TypeKind::Array { element_type } => {
            validate_childless_metadata(type_metadata)?;
            validate_index::<E>(context, element_type)?;
        }
        TypeKind::Tuple { field_types } => {
            validate_fields_metadata(type_metadata, field_types.len())?;
            for field_type in field_types.iter() {
                validate_index::<E>(context, field_type)?;
            }
        }
        TypeKind::Enum { variants } => {
            validate_enum_metadata(type_metadata, variants)?;
            for (_, field_types) in variants.iter() {
                for field_type in field_types.iter() {
                    validate_index::<E>(context, field_type)?;
                }
            }
        }
        TypeKind::Map {
            key_type,
            value_type,
        } => {
            validate_childless_metadata(type_metadata)?;
            validate_index::<E>(context, key_type)?;
            validate_index::<E>(context, value_type)?;
        }
        TypeKind::Custom(custom_type_kind) => {
            E::validate_custom_schema_type(
                context,
                CustomSchemaTypeValidationRequest {
                    custom_type_kind,
                    type_metadata,
                },
            )?;
        }
    }

    Ok(())
}

pub fn validate_index<E: CustomTypeExtension>(
    context: &TypeValidationContext,
    type_index: &LocalTypeIndex,
) -> Result<(), SchemaValidationError> {
    match type_index {
        LocalTypeIndex::WellKnown(well_known_index) => {
            if resolve_well_known_type::<E>(*well_known_index).is_none() {
                return Err(SchemaValidationError::TypeKindInvalidWellKnownIndex);
            }
        }
        LocalTypeIndex::SchemaLocalIndex(schema_local_index) => {
            if *schema_local_index >= context.local_types_len {
                return Err(SchemaValidationError::TypeKindInvalidSchemaLocalIndex);
            }
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
            // None can apply to any field count
        }
        Children::Fields(fields_matadata) => {
            if fields_matadata.len() != field_count {
                return Err(SchemaValidationError::TypeMetadataContainedWrongNumberOfChildNames);
            }
            for field_metadata in fields_matadata.iter() {
                let FieldMetadata { field_name } = field_metadata;
                validate_field_name(field_name)?;
            }
        }
        Children::Variants(_) => {
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
        Children::None | Children::Fields(_) => {
            return Err(SchemaValidationError::TypeMetadataForEnumIsNotEnumVariantChildNames)
        }
        Children::Variants(variants_metadata) => {
            if variants_metadata.len() != variants.len() {
                return Err(SchemaValidationError::TypeMetadataContainedWrongNumberOfChildNames);
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
    if name.len() == 0 {
        return Err(SchemaValidationError::InvalidTypeName {
            message: "Type name cannot be empty".to_owned(),
        });
    }
    if name.len() > 100 {
        return Err(SchemaValidationError::InvalidTypeName {
            message: "Type name cannot be more than 100 characters".to_owned(),
        });
    }
    for char in name.chars() {
        if !matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_') {
            return Err(SchemaValidationError::InvalidTypeName {
                // We need to validate this because we need to generate types from these names
                // Rust is much less prescriptive for identifier names (see https://doc.rust-lang.org/reference/identifiers.html and https://unicode.org/reports/tr31/)
                // But we can always loosen this later
                message: "At present, types names must match [0-9A-Za-z_]+".to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_enum_variant_name(name: &str) -> Result<(), SchemaValidationError> {
    if name.len() == 0 {
        return Err(SchemaValidationError::InvalidEnumVariantName {
            message: "Enum variant name cannot be empty".to_owned(),
        });
    }
    if name.len() > 100 {
        return Err(SchemaValidationError::InvalidEnumVariantName {
            message: "Enum variant name cannot be more than 100 characters".to_owned(),
        });
    }
    for char in name.chars() {
        if !matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_') {
            return Err(SchemaValidationError::InvalidEnumVariantName {
                message: "At present, enum variant names must match [0-9A-Za-z_]+".to_owned(),
            });
        }
    }
    Ok(())
}

fn validate_field_name(name: &str) -> Result<(), SchemaValidationError> {
    if name.len() == 0 {
        return Err(SchemaValidationError::InvalidFieldName {
            message: "Field name cannot be empty".to_owned(),
        });
    }
    if name.len() > 100 {
        return Err(SchemaValidationError::InvalidFieldName {
            message: "Field name cannot be more than 100 characters".to_owned(),
        });
    }
    for char in name.chars() {
        if !matches!(char, '0'..='9' | 'A'..='Z' | 'a'..='z' | '_') {
            return Err(SchemaValidationError::InvalidFieldName {
                message: "At present, field names must match [0-9A-Za-z_]+".to_owned(),
            });
        }
    }
    Ok(())
}
