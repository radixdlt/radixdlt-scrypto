use crate::rust::prelude::*;
use crate::*;

/// An array of custom type kinds, and associated extra information which can attach to the type kinds
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
// NB - the generic parameter E isn't embedded in the value model itself - instead:
// * Via TypeKind, E::CustomTypeKind<LocalTypeIndex> gets embedded
// * Via TypeValidation, E::CustomTypeValidation gets embedded
// So theses are the child types which need to be registered with the sbor macro for it to compile
#[sbor(child_types = "E::CustomTypeKind<LocalTypeIndex>, E::CustomTypeValidation")]
pub struct Schema<E: CustomTypeExtension> {
    pub type_kinds: Vec<SchemaTypeKind<E>>,
    pub type_metadata: Vec<TypeMetadata>, // TODO: reconsider adding type hash when it's ready!
    pub type_validations: Vec<SchemaTypeValidation<E>>,
}

pub type SchemaTypeKind<E> =
    TypeKind<<E as CustomTypeExtension>::CustomValueKind, SchemaCustomTypeKind<E>, LocalTypeIndex>;
pub type SchemaCustomTypeKind<E> = <E as CustomTypeExtension>::CustomTypeKind<LocalTypeIndex>;
pub type SchemaTypeValidation<E> = TypeValidation<<E as CustomTypeExtension>::CustomTypeValidation>;
pub type SchemaCustomTypeValidation<E> = <E as CustomTypeExtension>::CustomTypeValidation;

impl<E: CustomTypeExtension> Schema<E> {
    pub fn empty() -> Self {
        Self {
            type_kinds: vec![],
            type_metadata: vec![],
            type_validations: vec![],
        }
    }

    pub fn resolve_type_kind(
        &self,
        type_index: LocalTypeIndex,
    ) -> Option<&TypeKind<E::CustomValueKind, E::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>>
    {
        match type_index {
            LocalTypeIndex::WellKnown(index) => {
                E::resolve_well_known_type(index).map(|data| &data.kind)
            }
            LocalTypeIndex::SchemaLocalIndex(index) => self.type_kinds.get(index),
        }
    }

    pub fn resolve_type_metadata(&self, type_index: LocalTypeIndex) -> Option<&TypeMetadata> {
        match type_index {
            LocalTypeIndex::WellKnown(index) => {
                E::resolve_well_known_type(index).map(|data| &data.metadata)
            }
            LocalTypeIndex::SchemaLocalIndex(index) => self.type_metadata.get(index),
        }
    }

    pub fn resolve_type_validation(
        &self,
        type_index: LocalTypeIndex,
    ) -> Option<&TypeValidation<E::CustomTypeValidation>> {
        match type_index {
            LocalTypeIndex::WellKnown(index) => {
                E::resolve_well_known_type(index).map(|data| &data.validation)
            }
            LocalTypeIndex::SchemaLocalIndex(index) => self.type_validations.get(index),
        }
    }

    pub fn validate(&self) -> Result<(), SchemaValidationError> {
        validate_schema(self)
    }
}
