use crate::internal_prelude::*;

define_wrapped_hash!(
    /// Represents a particular schema under a package
    SchemaHash
);

/// An Identifier for a structural type. This can be treated almost
/// like a pointer as two equivalent type identifiers will map to
/// the same schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub struct TypeIdentifier(pub SchemaHash, pub LocalTypeIndex);

/// A reference to the type to substitute with for the case of
/// generics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum GenericSubstitution {
    Local(TypeIdentifier),
}
