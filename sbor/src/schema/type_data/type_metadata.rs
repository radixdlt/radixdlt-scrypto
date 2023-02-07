use crate::rust::borrow::Cow;
use crate::rust::collections::BTreeMap;
use crate::rust::vec::Vec;
use crate::TypeHash;

/// This is the struct used in the Schema
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NovelTypeMetadata {
    pub type_hash: TypeHash,
    pub type_metadata: TypeMetadata,
}

/// This enables the type to be represented as eg JSON
/// Also used to facilitate type reconstruction
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeMetadata {
    pub type_name: Cow<'static, str>,
    pub children: Children,
}

impl TypeMetadata {
    pub fn with_type_hash(self, type_hash: TypeHash) -> NovelTypeMetadata {
        NovelTypeMetadata {
            type_hash,
            type_metadata: self,
        }
    }

    pub fn named_no_child_names(name: &'static str) -> Self {
        Self {
            type_name: Cow::Borrowed(name),
            children: Children::None,
        }
    }

    pub fn named_with_fields(name: &'static str, field_names: &[&'static str]) -> Self {
        let field_names = field_names
            .iter()
            .map(|field_name| FieldMetadata {
                field_name: Cow::Borrowed(*field_name),
            })
            .collect();
        Self {
            type_name: Cow::Borrowed(name),
            children: Children::Fields(field_names),
        }
    }

    pub fn named_with_variants(
        name: &'static str,
        variant_naming: BTreeMap<u8, TypeMetadata>,
    ) -> Self {
        Self {
            type_name: Cow::Borrowed(name),
            children: Children::Variants(variant_naming),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Children {
    #[default]
    None,
    Fields(Vec<FieldMetadata>),
    Variants(BTreeMap<u8, TypeMetadata>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FieldMetadata {
    pub field_name: Cow<'static, str>,
}
