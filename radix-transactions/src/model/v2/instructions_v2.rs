use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct InstructionsV2(pub Rc<Vec<InstructionV2>>);

impl From<Vec<InstructionV2>> for InstructionsV2 {
    fn from(value: Vec<InstructionV2>) -> Self {
        InstructionsV2(Rc::new(value))
    }
}

impl From<InstructionsV2> for Vec<InstructionV2> {
    fn from(value: InstructionsV2) -> Self {
        value.0.as_ref().clone()
    }
}

impl TransactionPartialPrepare for InstructionsV2 {
    type Prepared = PreparedInstructionsV2;
}

// We summarize all the transactions as a single unit (not transaction-by-transaction)
pub type PreparedInstructionsV2 = SummarizedRawValueBodyWithReferences<InstructionsV2>;
