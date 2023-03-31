use crate::rust::collections::BTreeMap;
use crate::rust::vec::Vec;
use crate::*;

mod type_kind;
mod type_metadata;
mod type_validation;

pub use type_kind::*;
pub use type_metadata::*;
pub use type_validation::*;

/// Combines all data about a Type:
/// * `kind` - The type's [`TypeKind`] - this is essentially the definition of the structure of the type,
///   and includes the type's `ValueKind` as well as the [`TypeKind`] of any child types.
/// * `metadata` - The type's [`TypeMetadata`] which includes the name of the type and any of its fields or variants.
/// * `validation` - The type's [`TypeValidation`] which includes extra validation instructions for the type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeData<C: CustomTypeKind<L>, L: SchemaTypeLink> {
    pub kind: TypeKind<C::CustomValueKind, C, L>,
    pub metadata: TypeMetadata,
    pub validation:
        TypeValidation<<C::CustomTypeExtension as CustomTypeExtension>::CustomTypeValidation>,
}

impl<C: CustomTypeKind<L>, L: SchemaTypeLink + Categorize<C::CustomValueKind>> TypeData<C, L> {
    pub fn new(kind: TypeKind<C::CustomValueKind, C, L>, metadata: TypeMetadata) -> Self {
        Self {
            kind,
            metadata,
            validation: TypeValidation::None,
        }
    }

    pub fn unnamed(kind: TypeKind<C::CustomValueKind, C, L>) -> Self {
        Self {
            kind,
            metadata: TypeMetadata::unnamed(),
            validation: TypeValidation::None,
        }
    }

    pub fn no_child_names(kind: TypeKind<C::CustomValueKind, C, L>, name: &'static str) -> Self {
        Self {
            kind,
            metadata: TypeMetadata::no_child_names(name),
            validation: TypeValidation::None,
        }
    }

    pub fn struct_with_unit_fields(name: &'static str) -> Self {
        Self::new(
            TypeKind::Tuple {
                field_types: crate::rust::vec![],
            },
            TypeMetadata::no_child_names(name),
        )
    }

    pub fn struct_with_unnamed_fields(name: &'static str, field_types: Vec<L>) -> Self {
        Self::new(
            TypeKind::Tuple { field_types },
            TypeMetadata::no_child_names(name),
        )
    }

    pub fn struct_with_named_fields(name: &'static str, fields: Vec<(&'static str, L)>) -> Self {
        let (field_names, field_types): (Vec<_>, _) = fields.into_iter().unzip();
        Self::new(
            TypeKind::Tuple { field_types },
            TypeMetadata::struct_fields(name, &field_names),
        )
    }

    pub fn enum_variants(name: &'static str, variants: BTreeMap<u8, TypeData<C, L>>) -> Self {
        let (variant_naming, variant_tuple_schemas) = variants
            .into_iter()
            .map(|(k, variant_type_data)| {
                let variant_fields_schema = match variant_type_data.kind {
                    TypeKind::Tuple { field_types } => field_types,
                    _ => panic!("Only Tuple is allowed in Enum variant TypeData"),
                };
                ((k, variant_type_data.metadata), (k, variant_fields_schema))
            })
            .unzip();
        Self::new(
            TypeKind::Enum {
                variants: variant_tuple_schemas,
            },
            TypeMetadata::enum_variants(name, variant_naming),
        )
    }

    pub fn with_validation(
        self,
        type_validation: TypeValidation<
            <C::CustomTypeExtension as CustomTypeExtension>::CustomTypeValidation,
        >,
    ) -> Self {
        Self {
            kind: self.kind,
            metadata: self.metadata,
            validation: type_validation,
        }
    }
}
