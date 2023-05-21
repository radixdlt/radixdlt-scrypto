use super::*;
use crate::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
#[sbor(transparent)]
pub struct InstructionsV1(pub Vec<InstructionV1>);

impl TransactionPartialEncode for InstructionsV1 {
    type Prepared = PreparedInstructionsV1;
}

// We summarize all the transactions as a single unit (not transaction-by-transaction)
pub type PreparedInstructionsV1 = SummarizedRawFullBodyWithReferences<InstructionsV1>;
