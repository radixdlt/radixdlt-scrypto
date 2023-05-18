use super::*;
use crate::prelude::*;

pub use super::super::Instruction as InstructionV1;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
#[sbor(transparent)]
pub struct InstructionsV1(pub Vec<InstructionV1>);

// We summarize all the transactions as a single unit (not transaction-by-transaction)
pub type PreparedInstructionsV1 = SummarizedRawFullBody<InstructionsV1>;
