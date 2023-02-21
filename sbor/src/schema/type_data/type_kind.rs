use super::*;
use crate::rust::collections::BTreeMap;
use crate::rust::vec::Vec;

/// A schema for the values that a codec can decode / views as valid
#[derive(Debug, Clone, PartialEq, Eq, Sbor)]
#[sbor(child_types = "C,L")]
pub enum TypeKind<X: CustomValueKind, C: CustomTypeKind<L, CustomValueKind = X>, L: SchemaTypeLink>
{
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

    Enum { variants: BTreeMap<u8, Vec<L>> },

    Map { key_type: L, value_type: L },

    // Custom Types
    Custom(C),
}
