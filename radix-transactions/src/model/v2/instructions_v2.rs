use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct InstructionsV2(pub Vec<InstructionV2>);

impl InstructionsV2 {
    pub fn to_vec(&self) -> Vec<InstructionV2> {
        self.0.clone()
    }
}

impl From<Vec<InstructionV2>> for InstructionsV2 {
    fn from(value: Vec<InstructionV2>) -> Self {
        InstructionsV2(value)
    }
}

impl From<InstructionsV2> for Vec<InstructionV2> {
    fn from(value: InstructionsV2) -> Self {
        value.0
    }
}

impl TransactionPartialPrepare for InstructionsV2 {
    type Prepared = PreparedInstructionsV2;
}

// We summarize all the transactions as a single unit (not transaction-by-transaction)
pub type PreparedInstructionsV2 = SummarizedRawValueBodyWithReferences<InstructionsV2>;
