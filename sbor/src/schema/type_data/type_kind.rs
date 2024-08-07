use super::*;
use crate::rust::collections::IndexMap;
use crate::rust::vec::Vec;

pub type LocalTypeKind<S> = TypeKind<<S as CustomSchema>::CustomLocalTypeKind, LocalTypeId>;
pub type AggregatorTypeKind<S> =
    TypeKind<<S as CustomSchema>::CustomAggregatorTypeKind, RustTypeId>;

/// A schema for the values that a codec can decode / views as valid
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
#[sbor(child_types = "T, L", categorize_types = "L")]
pub enum TypeKind<T: CustomTypeKind<L>, L: SchemaTypeLink> {
    Any,

    // Simple Types
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    String,

    // Composite Types
    Array { element_type: L },

    Tuple { field_types: Vec<L> },

    Enum { variants: IndexMap<u8, Vec<L>> },

    Map { key_type: L, value_type: L },

    // Custom Types
    Custom(T),
}

impl<T: CustomTypeKind<L>, L: SchemaTypeLink> TypeKind<T, L> {
    pub fn label(&self) -> TypeKindLabel<T::CustomTypeKindLabel> {
        match self {
            TypeKind::Any => TypeKindLabel::Any,
            TypeKind::Bool => TypeKindLabel::Bool,
            TypeKind::I8 => TypeKindLabel::I8,
            TypeKind::I16 => TypeKindLabel::I16,
            TypeKind::I32 => TypeKindLabel::I32,
            TypeKind::I64 => TypeKindLabel::I64,
            TypeKind::I128 => TypeKindLabel::I128,
            TypeKind::U8 => TypeKindLabel::U8,
            TypeKind::U16 => TypeKindLabel::U16,
            TypeKind::U32 => TypeKindLabel::U32,
            TypeKind::U64 => TypeKindLabel::U64,
            TypeKind::U128 => TypeKindLabel::U128,
            TypeKind::String => TypeKindLabel::String,
            TypeKind::Array { .. } => TypeKindLabel::Array,
            TypeKind::Tuple { .. } => TypeKindLabel::Tuple,
            TypeKind::Enum { .. } => TypeKindLabel::Enum,
            TypeKind::Map { .. } => TypeKindLabel::Map,
            TypeKind::Custom(custom) => TypeKindLabel::Custom(custom.label()),
        }
    }

    pub fn category_name(&self) -> &'static str {
        self.label().name()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Sbor)]
pub enum TypeKindLabel<T: CustomTypeKindLabel> {
    Any,

    // Simple Types
    Bool,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    String,

    // Composite Types
    Array,
    Tuple,
    Enum,
    Map,

    // Custom Types
    Custom(T),
}

impl<C: CustomTypeKindLabel> TypeKindLabel<C> {
    pub fn name(&self) -> &'static str {
        match self {
            TypeKindLabel::Any => "Any",
            TypeKindLabel::Bool => "Bool",
            TypeKindLabel::I8 => "I8",
            TypeKindLabel::I16 => "I16",
            TypeKindLabel::I32 => "I32",
            TypeKindLabel::I64 => "I64",
            TypeKindLabel::I128 => "I128",
            TypeKindLabel::U8 => "U8",
            TypeKindLabel::U16 => "U16",
            TypeKindLabel::U32 => "U32",
            TypeKindLabel::U64 => "U64",
            TypeKindLabel::U128 => "U128",
            TypeKindLabel::String => "String",
            TypeKindLabel::Array => "Array",
            TypeKindLabel::Tuple => "Tuple",
            TypeKindLabel::Enum => "Enum",
            TypeKindLabel::Map => "Map",
            TypeKindLabel::Custom(custom) => custom.name(),
        }
    }
}
