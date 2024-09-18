use super::*;
use crate::internal_prelude::*;

#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
#[sbor(transparent)]
pub struct InstructionsV1(pub Rc<Vec<InstructionV1>>);

impl From<Vec<InstructionV1>> for InstructionsV1 {
    fn from(value: Vec<InstructionV1>) -> Self {
        InstructionsV1(Rc::new(value))
    }
}

impl From<InstructionsV1> for Vec<InstructionV1> {
    fn from(value: InstructionsV1) -> Self {
        value.0.as_ref().clone()
    }
}

impl TransactionPartialPrepare for InstructionsV1 {
    type Prepared = PreparedInstructionsV1;
}

impl ReadableManifest for [InstructionV1] {
    type Instruction = InstructionV1;

    fn is_subintent(&self) -> bool {
        false
    }

    fn get_instructions(&self) -> &[Self::Instruction] {
        self
    }

    fn get_blobs<'a>(&'a self) -> impl Iterator<Item = (&'a Hash, &'a Vec<u8>)> {
        core::iter::empty()
    }

    fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
        ManifestObjectNamesRef::Unknown
    }

    fn validate(&self) -> Result<(), ManifestValidationError> {
        let mut ruleset = ValidationRuleset::all();
        ruleset.validate_blob_refs = false;
        StaticManifestInterpreter::new(ruleset, self).validate()
    }
}

// We summarize all the transactions as a single unit (not transaction-by-transaction)
#[allow(deprecated)]
pub type PreparedInstructionsV1 = SummarizedRawFullValueWithReferences<InstructionsV1>;
