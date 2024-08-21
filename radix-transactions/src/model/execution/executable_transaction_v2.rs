use crate::internal_prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableTransactionV2 {
    pub(crate) primary: ExecutableCore,
    pub(crate) subintents: IndexMap<TransactionIntentHash, ExecutableCore>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableCore {}
