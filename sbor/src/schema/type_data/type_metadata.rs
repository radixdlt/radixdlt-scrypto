use borrow::Borrow;

use crate::rust::prelude::*;
use crate::*;

/// This is the struct used in the Schema
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct NovelTypeMetadata {
    pub type_hash: TypeHash,
    pub type_metadata: TypeMetadata,
}

/// This enables the type to be represented as eg JSON
/// Also used to facilitate type reconstruction
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub struct TypeMetadata {
    pub type_name: Option<Cow<'static, str>>,
    pub child_names: Option<ChildNames>,
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum ChildNames {
    NamedFields(Vec<Cow<'static, str>>),
    EnumVariants(IndexMap<u8, TypeMetadata>),
}

impl TypeMetadata {
    pub fn unnamed() -> Self {
        Self {
            type_name: None,
            child_names: None,
        }
    }

    pub fn no_child_names(name: &'static str) -> Self {
        Self {
            type_name: Some(Cow::Borrowed(name)),
            child_names: None,
        }
    }

    pub fn struct_fields(name: &'static str, field_names: &[&'static str]) -> Self {
        let field_names = field_names
            .iter()
            .map(|field_name| Cow::Borrowed(*field_name))
            .collect();
        Self {
            type_name: Some(Cow::Borrowed(name)),
            child_names: Some(ChildNames::NamedFields(field_names)),
        }
    }

    pub fn enum_variants(name: &'static str, variant_naming: IndexMap<u8, TypeMetadata>) -> Self {
        Self {
            type_name: Some(Cow::Borrowed(name)),
            child_names: Some(ChildNames::EnumVariants(variant_naming)),
        }
    }

    pub fn with_name(mut self, name: Option<Cow<'static, str>>) -> Self {
        self.type_name = name;
        self
    }

    pub fn with_type_hash(self, type_hash: TypeHash) -> NovelTypeMetadata {
        NovelTypeMetadata {
            type_hash,
            type_metadata: self,
        }
    }

    pub fn get_name(&self) -> Option<&str> {
        self.type_name.as_ref().map(|c| c.as_ref())
    }

    pub fn get_name_string(&self) -> Option<String> {
        self.type_name.as_ref().map(|c| c.to_string())
    }

    pub fn get_field_names<'a>(&'a self) -> Option<&'a [Cow<'static, str>]> {
        match &self.child_names {
            Some(ChildNames::NamedFields(field_names)) => Some(field_names.as_slice()),
            _ => None,
        }
    }

    pub fn get_field_name<'a>(&'a self, field_index: usize) -> Option<&'a str> {
        match &self.child_names {
            Some(ChildNames::NamedFields(field_names)) => {
                Some(field_names.get(field_index)?.borrow())
            }
            _ => None,
        }
    }

    pub fn get_enum_variant_data<'a>(&'a self, discriminator: u8) -> Option<&'a TypeMetadata> {
        match &self.child_names {
            Some(ChildNames::EnumVariants(variants)) => variants.get(&discriminator),
            _ => None,
        }
    }

    pub fn get_matching_tuple_data(&self, fields_length: usize) -> TupleData {
        TupleData {
            name: self.get_name(),
            field_names: self
                .get_field_names()
                .filter(|field_names| field_names.len() == fields_length),
        }
    }

    pub fn get_matching_enum_variant_data(
        &self,
        variant_id: u8,
        fields_length: usize,
    ) -> EnumVariantData {
        let enum_name = self.get_name();
        let Some(ChildNames::EnumVariants(variants)) = &self.child_names else {
            return EnumVariantData {
                enum_name,
                variant_name: None,
                field_names: None,
            };
        };
        let Some(variant_metadata) = variants.get(&variant_id) else {
            return EnumVariantData {
                enum_name,
                variant_name: None,
                field_names: None,
            };
        };
        EnumVariantData {
            enum_name,
            variant_name: variant_metadata.get_name(),
            field_names: variant_metadata
                .get_field_names()
                .filter(|field_names| field_names.len() == fields_length),
        }
    }
}

#[derive(Debug, Default)]
pub struct TupleData<'s> {
    pub name: Option<&'s str>,
    pub field_names: Option<&'s [Cow<'static, str>]>,
}

#[derive(Debug, Default)]
pub struct EnumVariantData<'s> {
    pub enum_name: Option<&'s str>,
    pub variant_name: Option<&'s str>,
    pub field_names: Option<&'s [Cow<'static, str>]>,
}
