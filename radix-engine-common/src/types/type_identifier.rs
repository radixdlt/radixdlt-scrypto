use crate::internal_prelude::*;

define_wrapped_hash!(
    /// Represents a particular schema under a package
    SchemaHash
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub struct TypeIdentifier(pub SchemaHash, pub LocalTypeIndex);
