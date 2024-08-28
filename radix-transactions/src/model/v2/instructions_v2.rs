use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct InstructionsV2(pub Rc<Vec<InstructionV1>>);

impl TransactionPartialPrepare for InstructionsV2 {
    type Prepared = PreparedInstructionsV2;
}

// We summarize all the transactions as a single unit (not transaction-by-transaction)
pub type PreparedInstructionsV2 = SummarizedRawValueBodyWithReferences<InstructionsV2>;
