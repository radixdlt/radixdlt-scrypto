use crate::rust::prelude::*;
use crate::*;

/// An array of custom type kinds, and associated extra information which can attach to the type kinds
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
// NB - the generic parameter E isn't embedded in the value model itself - instead:
// * Via TypeKind, E::CustomTypeKind<LocalTypeIndex> gets embedded
// * Via TypeValidation, E::CustomTypeValidation gets embedded
// So theses are the child types which need to be registered with the sbor macro for it to compile
#[sbor(child_types = "S::CustomTypeKind<LocalTypeIndex>, S::CustomTypeValidation")]
pub struct Schema<S: CustomSchema> {
    pub type_kinds: Vec<SchemaTypeKind<S>>,
    pub type_metadata: Vec<TypeMetadata>, // TODO: reconsider adding type hash when it's ready!
    pub type_validations: Vec<TypeValidation<S::CustomTypeValidation>>,
}

pub type SchemaTypeKind<S> =
    TypeKind<<S as CustomSchema>::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>;

impl<S: CustomSchema> Schema<S> {
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
    ) -> Option<&TypeKind<S::CustomTypeKind<LocalTypeIndex>, LocalTypeIndex>> {
        match type_index {
            LocalTypeIndex::WellKnown(index) => {
                S::resolve_well_known_type(index).map(|data| &data.kind)
            }
            LocalTypeIndex::SchemaLocalIndex(index) => self.type_kinds.get(index),
        }
    }

    pub fn resolve_type_metadata(&self, type_index: LocalTypeIndex) -> Option<&TypeMetadata> {
        match type_index {
            LocalTypeIndex::WellKnown(index) => {
                S::resolve_well_known_type(index).map(|data| &data.metadata)
            }
            LocalTypeIndex::SchemaLocalIndex(index) => self.type_metadata.get(index),
        }
    }

    pub fn resolve_matching_tuple_metadata(
        &self,
        type_index: LocalTypeIndex,
        fields_length: usize,
    ) -> TupleData<'_> {
        self.resolve_type_metadata(type_index)
            .map(|m| m.get_matching_tuple_data(fields_length))
            .unwrap_or_default()
    }

    pub fn resolve_matching_enum_metadata<'s>(
        &self,
        type_index: LocalTypeIndex,
        variant_id: u8,
        fields_length: usize,
    ) -> EnumVariantData<'_> {
        self.resolve_type_metadata(type_index)
            .map(|m| m.get_matching_enum_variant_data(variant_id, fields_length))
            .unwrap_or_default()
    }

    pub fn resolve_matching_array_metadata(&self, type_index: LocalTypeIndex) -> ArrayData<'_> {
        let Some(TypeKind::Array { element_type }) = self.resolve_type_kind(type_index) else {
            return ArrayData::default();
        };
        ArrayData {
            array_name: self
                .resolve_type_metadata(type_index)
                .and_then(|m| m.get_name()),
            element_name: self
                .resolve_type_metadata(*element_type)
                .and_then(|m| m.get_name()),
        }
    }

    pub fn resolve_matching_map_metadata(&self, type_index: LocalTypeIndex) -> MapData<'_> {
        let Some(TypeKind::Map { key_type, value_type }) = self.resolve_type_kind(type_index) else {
            return MapData::default();
        };
        MapData {
            map_name: self
                .resolve_type_metadata(type_index)
                .and_then(|m| m.get_name()),
            key_name: self
                .resolve_type_metadata(*key_type)
                .and_then(|m| m.get_name()),
            value_name: self
                .resolve_type_metadata(*value_type)
                .and_then(|m| m.get_name()),
        }
    }

    pub fn resolve_type_name_from_metadata(&self, type_index: LocalTypeIndex) -> Option<&'_ str> {
        self.resolve_type_metadata(type_index)
            .and_then(|m| m.get_name())
    }

    pub fn resolve_type_validation(
        &self,
        type_index: LocalTypeIndex,
    ) -> Option<&TypeValidation<S::CustomTypeValidation>> {
        match type_index {
            LocalTypeIndex::WellKnown(index) => {
                S::resolve_well_known_type(index).map(|data| &data.validation)
            }
            LocalTypeIndex::SchemaLocalIndex(index) => self.type_validations.get(index),
        }
    }

    pub fn validate(&self) -> Result<(), SchemaValidationError> {
        validate_schema(self)
    }
}

#[derive(Debug, Default)]
pub struct ArrayData<'m> {
    pub array_name: Option<&'m str>,
    pub element_name: Option<&'m str>,
}

#[derive(Debug, Default)]
pub struct MapData<'m> {
    pub map_name: Option<&'m str>,
    pub key_name: Option<&'m str>,
    pub value_name: Option<&'m str>,
}
