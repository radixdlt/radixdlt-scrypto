use crate::internal_prelude::KeyValueStoreGenericSubstitutions;
use crate::ScryptoSbor;
use sbor::rust::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct KeyValueStoreInfo {
    pub generic_substitutions: KeyValueStoreGenericSubstitutions,
}
