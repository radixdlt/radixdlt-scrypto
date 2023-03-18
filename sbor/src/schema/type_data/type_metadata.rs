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

    pub fn enum_variants(name: &'static str, variant_naming: BTreeMap<u8, TypeMetadata>) -> Self {
        Self {
            type_name: Some(Cow::Borrowed(name)),
            child_names: Some(ChildNames::EnumVariants(variant_naming)),
        }
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
}

#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
pub enum ChildNames {
    NamedFields(Vec<Cow<'static, str>>),
    EnumVariants(BTreeMap<u8, TypeMetadata>),
}
