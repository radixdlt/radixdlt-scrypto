use crate::crypto::Hash;
use crate::Sbor;
use sbor::LocalTypeIndex;

/// An Identifier for a structural type. This can be treated almost
/// like a pointer as two equivalent type identifiers will map to
/// the same schema
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub struct TypeIdentifier(pub Hash, pub LocalTypeIndex);

/// A reference to the type to substitute with for the case of
/// generics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum GenericSubstitution {
    Local(TypeIdentifier),
}
