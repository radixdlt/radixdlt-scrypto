use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct InstructionsV1(pub Vec<InstructionV1>);

impl From<Vec<InstructionV1>> for InstructionsV1 {
    fn from(value: Vec<InstructionV1>) -> Self {
        InstructionsV1(value)
    }
}

impl From<InstructionsV1> for Vec<InstructionV1> {
    fn from(value: InstructionsV1) -> Self {
        value.0
    }
}

impl TransactionPartialPrepare for InstructionsV1 {
    type Prepared = PreparedInstructionsV1;
}

// We summarize all the transactions as a single unit (not transaction-by-transaction)
#[allow(deprecated)]
pub type PreparedInstructionsV1 = SummarizedRawFullValueWithReferences<InstructionsV1>;
