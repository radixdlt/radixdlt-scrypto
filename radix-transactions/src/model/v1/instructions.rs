use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct InstructionsV1(pub Rc<Vec<InstructionV1>>);

impl TransactionPartialPrepare for InstructionsV1 {
    type Prepared = PreparedInstructionsV1;
}

// We summarize all the transactions as a single unit (not transaction-by-transaction)
#[allow(deprecated)]
pub type PreparedInstructionsV1 = SummarizedRawFullValueWithReferences<InstructionsV1>;
