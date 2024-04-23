use crate::rust::prelude::*;
use crate::*;

define_single_versioned!(
    #[derive(Debug, Clone, PartialEq, Eq, Sbor)]
    #[sbor(child_types = "S::CustomTypeKind<LocalTypeId>, S::CustomTypeValidation")]
    pub enum VersionedSchema<S: CustomSchema> => Schema<S> = SchemaV1::<S>
);

impl<S: CustomSchema> VersionedSchema<S> {
    pub fn v1(&self) -> &SchemaV1<S> {
        self.as_unique_latest_ref()
    }

    pub fn v1_mut(&mut self) -> &mut SchemaV1<S> {
        self.as_unique_latest_mut()
    }
}

impl<S: CustomSchema> VersionedSchema<S> {
    pub fn empty() -> Self {
        Schema::empty().into()
    }
}

impl<S: CustomSchema> Default for VersionedSchema<S> {
    fn default() -> Self {
        Self::empty()
    }
}

/// An array of custom type kinds, and associated extra information which can attach to the type kinds
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
// NB - the generic parameter E isn't embedded in the value model itself - instead:
// * Via TypeKind, S::CustomTypeKind<LocalTypeId> gets embedded
// * Via TypeValidation, S::CustomTypeValidation gets embedded
// So theses are the child types which need to be registered with the sbor macro for it to compile
#[sbor(child_types = "S::CustomTypeKind<LocalTypeId>, S::CustomTypeValidation")]
pub struct SchemaV1<S: CustomSchema> {
    pub type_kinds: Vec<SchemaTypeKind<S>>,
    pub type_metadata: Vec<TypeMetadata>, // TODO: reconsider adding type hash when it's ready!
    pub type_validations: Vec<TypeValidation<S::CustomTypeValidation>>,
}

pub type SchemaTypeKind<S> =
    TypeKind<<S as CustomSchema>::CustomTypeKind<LocalTypeId>, LocalTypeId>;

impl<S: CustomSchema> SchemaV1<S> {
    pub fn empty() -> Self {
        Self {
            type_kinds: vec![],
            type_metadata: vec![],
            type_validations: vec![],
        }
    }

    pub fn resolve_type_kind(
        &self,
        type_id: LocalTypeId,
    ) -> Option<&TypeKind<S::CustomTypeKind<LocalTypeId>, LocalTypeId>> {
        match type_id {
            LocalTypeId::WellKnown(index) => {
                S::resolve_well_known_type(index).map(|data| &data.kind)
            }
            LocalTypeId::SchemaLocalIndex(index) => self.type_kinds.get(index),
        }
    }

    pub fn resolve_type_metadata(&self, type_id: LocalTypeId) -> Option<&TypeMetadata> {
        match type_id {
            LocalTypeId::WellKnown(index) => {
                S::resolve_well_known_type(index).map(|data| &data.metadata)
            }
            LocalTypeId::SchemaLocalIndex(index) => self.type_metadata.get(index),
        }
    }

    pub fn resolve_matching_tuple_metadata(
        &self,
        type_id: LocalTypeId,
        fields_length: usize,
    ) -> TupleData<'_> {
        self.resolve_type_metadata(type_id)
            .map(|m| m.get_matching_tuple_data(fields_length))
            .unwrap_or_default()
    }

    pub fn resolve_matching_enum_metadata<'s>(
        &self,
        type_id: LocalTypeId,
        variant_id: u8,
        fields_length: usize,
    ) -> EnumVariantData<'_> {
        self.resolve_type_metadata(type_id)
            .map(|m| m.get_matching_enum_variant_data(variant_id, fields_length))
            .unwrap_or_default()
    }

    pub fn resolve_matching_array_metadata(&self, type_id: LocalTypeId) -> ArrayData<'_> {
        let Some(TypeKind::Array { element_type }) = self.resolve_type_kind(type_id) else {
            return ArrayData::default();
        };
        ArrayData {
            array_name: self
                .resolve_type_metadata(type_id)
                .and_then(|m| m.get_name()),
            element_name: self
                .resolve_type_metadata(*element_type)
                .and_then(|m| m.get_name()),
        }
    }

    pub fn resolve_matching_map_metadata(&self, type_id: LocalTypeId) -> MapData<'_> {
        let Some(TypeKind::Map {
            key_type,
            value_type,
        }) = self.resolve_type_kind(type_id)
        else {
            return MapData::default();
        };
        MapData {
            map_name: self
                .resolve_type_metadata(type_id)
                .and_then(|m| m.get_name()),
            key_name: self
                .resolve_type_metadata(*key_type)
                .and_then(|m| m.get_name()),
            value_name: self
                .resolve_type_metadata(*value_type)
                .and_then(|m| m.get_name()),
        }
    }

    pub fn resolve_type_name_from_metadata(&self, type_id: LocalTypeId) -> Option<&'_ str> {
        self.resolve_type_metadata(type_id)
            .and_then(|m| m.get_name())
    }

    pub fn resolve_type_validation(
        &self,
        type_id: LocalTypeId,
    ) -> Option<&TypeValidation<S::CustomTypeValidation>> {
        match type_id {
            LocalTypeId::WellKnown(index) => {
                S::resolve_well_known_type(index).map(|data| &data.validation)
            }
            LocalTypeId::SchemaLocalIndex(index) => self.type_validations.get(index),
        }
    }

    pub fn validate(&self) -> Result<(), SchemaValidationError> {
        validate_schema(self)
    }
}

impl<S: CustomSchema> Default for SchemaV1<S> {
    fn default() -> Self {
        Self::empty()
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
