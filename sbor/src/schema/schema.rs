use crate::rust::vec::Vec;
use crate::*;

/// An array of custom type kinds, and associated extra information which can attach to the type kinds
#[derive(Debug, Clone, PartialEq, Sbor)]
#[sbor(child_types = "E::CustomTypeKind<LocalTypeIndex>, E::CustomTypeValidation")]
pub struct Schema<E: CustomTypeExtension> {
    pub type_kinds: Vec<SchemaTypeKind<E>>,
    pub type_metadata: Vec<NovelTypeMetadata>,
    pub type_validations: Vec<SchemaTypeValidation<E>>,
}

pub type SchemaTypeKind<E> =
    TypeKind<<E as CustomTypeExtension>::CustomValueKind, SchemaCustomTypeKind<E>, LocalTypeIndex>;
pub type SchemaCustomTypeKind<E> = <E as CustomTypeExtension>::CustomTypeKind<LocalTypeIndex>;
pub type SchemaTypeValidation<E> = TypeValidation<<E as CustomTypeExtension>::CustomTypeValidation>;
pub type SchemaCustomTypeValidation<E> = <E as CustomTypeExtension>::CustomTypeValidation;

pub fn resolve_type_kind<'s: 't, 't, E: CustomTypeExtension>(
    type_kinds: &'s [SchemaTypeKind<E>],
    type_index: LocalTypeIndex,
) -> Option<&'t SchemaTypeKind<E>> {
    match type_index {
        LocalTypeIndex::WellKnown(index) => E::resolve_well_known_type(index)
            .map(|local_type_data| &local_type_data.kind),
        LocalTypeIndex::SchemaLocalIndex(index) => type_kinds.get(index),
    }
}

pub struct ResolvedTypeData<'a, E: CustomTypeExtension> {
    pub kind: &'a TypeKind<E::CustomValueKind, E::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>,
    pub metadata: &'a TypeMetadata,
    pub validation: &'a TypeValidation<E::CustomTypeValidation>,
}

impl<E: CustomTypeExtension> Schema<E> {
    pub fn resolve<'a>(&'a self, type_ref: LocalTypeIndex) -> Option<ResolvedTypeData<'a, E>> {
        match type_ref {
            LocalTypeIndex::WellKnown(index) => {
                match E::resolve_well_known_type(index) {
                    Some(TypeData { kind, metadata, validation }) => {
                        Some(ResolvedTypeData {
                            kind,
                            metadata,
                            validation,
                        })
                    },
                    None => None,
                }
            },
            LocalTypeIndex::SchemaLocalIndex(index) => {
                match (self.type_kinds.get(index), self.type_metadata.get(index), self.type_validations.get(index)) {
                    (Some(type_kind), Some(novel_metadata), Some(validation)) => Some(ResolvedTypeData {
                        kind: type_kind,
                        metadata: &novel_metadata.type_metadata,
                        validation,
                    }),
                    _ => None,
                }
            }
        }
    }

    pub fn validate(&self) -> Result<(), SchemaValidationError> {
        validate_schema(self)
    }
}
