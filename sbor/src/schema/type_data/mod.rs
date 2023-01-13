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
/// * `metadata` - The type's [`TypeMetadata`] including the name of the type and any of its fields or variants.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeData<C: CustomTypeKind<L>, L: SchemaTypeLink> {
    pub kind: TypeKind<C::CustomValueKind, C, L>,
    pub metadata: TypeMetadata,
    pub validation:
        TypeValidation<<C::CustomTypeExtension as CustomTypeExtension>::CustomTypeValidation>,
}

impl<C: CustomTypeKind<L>, L: SchemaTypeLink + Categorize<C::CustomValueKind>> TypeData<C, L> {
    pub fn new(metadata: TypeMetadata, kind: TypeKind<C::CustomValueKind, C, L>) -> Self {
        Self {
            kind,
            metadata,
            validation: TypeValidation::None,
        }
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

    pub fn named_no_child_names(
        name: &'static str,
        schema: TypeKind<C::CustomValueKind, C, L>,
    ) -> Self {
        Self::new(TypeMetadata::named_no_child_names(name), schema)
    }

    pub fn named_unit(name: &'static str) -> Self {
        Self::new(
            TypeMetadata::named_no_child_names(name),
            TypeKind::Tuple {
                field_types: crate::rust::vec![],
            },
        )
    }

    pub fn named_tuple(name: &'static str, field_types: Vec<L>) -> Self {
        Self::new(
            TypeMetadata::named_no_child_names(name),
            TypeKind::Tuple { field_types },
        )
    }

    pub fn named_fields_tuple(name: &'static str, fields: Vec<(&'static str, L)>) -> Self {
        let (field_names, field_types): (Vec<_>, _) = fields.into_iter().unzip();
        Self::new(
            TypeMetadata::named_with_fields(name, &field_names),
            TypeKind::Tuple { field_types },
        )
    }

    pub fn named_enum(name: &'static str, variants: BTreeMap<u8, TypeData<C, L>>) -> Self {
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
            TypeMetadata::named_with_variants(name, variant_naming),
            TypeKind::Enum {
                variants: variant_tuple_schemas,
            },
        )
    }
}
