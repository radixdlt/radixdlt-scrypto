use crate::crypto::Hash;
use crate::Sbor;
use sbor::LocalTypeIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub struct TypeIdentifier(pub Hash, pub LocalTypeIndex);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Sbor)]
pub enum TypeSubstitutionRef {
    Local(TypeIdentifier),
}
