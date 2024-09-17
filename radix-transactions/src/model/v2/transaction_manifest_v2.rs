use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// This isn't actually embedded as a model - it's just a useful model which we use
// in eg the manifest builder
//=================================================================================

/// Can be built with a [`ManifestV2Builder`]
#[derive(Debug, Clone, Default, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct TransactionManifestV2 {
    pub instructions: Vec<InstructionV2>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    pub children: Vec<ChildSubintent>,
    pub object_names: ManifestObjectNames,
}

impl ReadableManifest for TransactionManifestV2 {
    type Instruction = InstructionV2;

    fn get_instructions(&self) -> &[Self::Instruction] {
        &self.instructions
    }

    fn get_blobs(&self) -> &IndexMap<Hash, Vec<u8>> {
        &self.blobs
    }

    fn get_child_subintents(&self) -> &[ChildSubintent] {
        &self.children
    }

    fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
        self.object_names.as_ref()
    }

    fn validate(&self) -> Result<(), TransactionValidationError> {
        temporary_noop_validate();
        Ok(())
    }
}

#[deprecated]
fn temporary_noop_validate() {}

impl BuildableManifest for TransactionManifestV2 {
    fn add_instruction(&mut self, instruction: Self::Instruction) {
        self.instructions.push(instruction)
    }

    fn add_blob(&mut self, hash: Hash, content: Vec<u8>) {
        self.blobs.insert(hash, content);
    }

    fn set_names(&mut self, names: KnownManifestObjectNames) {
        self.object_names = names.into()
    }

    fn add_child_subintent(&mut self, hash: SubintentHash) -> Result<(), ManifestBuildError> {
        self.children.push(ChildSubintent { hash });
        Ok(())
    }

    fn suggested_execution_config_type(&self) -> SuggestedExecutionConfigType {
        SuggestedExecutionConfigType::Test
    }

    fn into_executable_with_proofs(
        self,
        nonce: u32,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, String> {
        TestTransaction::new_v2_builder(nonce)
            .finish_with_root_intent(self, initial_proofs)
            .into_executable(validator)
            .map_err(|err| format!("Could not prepare: {err:?}"))
    }
}

impl BuildableManifestSupportingChildren for TransactionManifestV2 {}

impl TransactionManifestV2 {
    pub fn from_intent_core(intent: &IntentCoreV2) -> Self {
        Self {
            instructions: intent.instructions.clone().into(),
            blobs: intent.blobs.clone().into(),
            children: intent.children.children.clone(),
            object_names: ManifestObjectNames::Unknown,
        }
    }

    pub fn for_intent(self) -> (InstructionsV2, BlobsV1, ChildIntentsV2) {
        (
            InstructionsV2(Rc::new(self.instructions)),
            self.blobs.into(),
            ChildIntentsV2 {
                children: self.children,
            },
        )
    }
}
