use crate::types::*;

#[derive(Debug, Clone, PartialEq, Eq)]
#[scrypto(TypeId, Encode, Decode)]
pub struct TransactionHashSubstate {
    pub hash: Hash,
}
