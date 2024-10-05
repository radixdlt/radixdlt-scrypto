use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// This isn't actually embedded as a model - it's just a useful model which we use
// in eg the manifest builder
//=================================================================================

/// Can be built with a [`ManifestV1Builder`]
#[derive(Debug, Clone, Default, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct TransactionManifestV1 {
    pub instructions: Vec<InstructionV1>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    pub object_names: ManifestObjectNames,
}

impl ReadableManifestBase for TransactionManifestV1 {
    fn is_subintent(&self) -> bool {
        false
    }

    fn get_blobs<'a>(&'a self) -> impl Iterator<Item = (&'a Hash, &'a Vec<u8>)> {
        self.blobs.iter()
    }

    fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
        self.object_names.as_ref()
    }
}

impl TypedReadableManifest for TransactionManifestV1 {
    type Instruction = InstructionV1;

    fn get_typed_instructions(&self) -> &[Self::Instruction] {
        &self.instructions
    }
}

impl BuildableManifest for TransactionManifestV1 {
    fn add_instruction(&mut self, instruction: Self::Instruction) {
        self.instructions.push(instruction)
    }

    fn add_blob(&mut self, hash: Hash, content: Vec<u8>) {
        self.blobs.insert(hash, content);
    }

    fn set_names(&mut self, names: KnownManifestObjectNames) {
        self.object_names = names.into()
    }

    fn default_test_execution_config_type(&self) -> DefaultTestExecutionConfigType {
        DefaultTestExecutionConfigType::Test
    }

    fn into_executable_with_proofs(
        self,
        nonce: u32,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, String> {
        TestTransaction::new_v1_from_nonce(self, nonce, initial_proofs)
            .into_executable(&validator)
            .map_err(|err| format!("Could not prepare: {err:?}"))
    }
}

impl TransactionManifestV1 {
    pub fn from_intent(intent: &IntentV1) -> Self {
        Self {
            instructions: intent.instructions.clone().into(),
            blobs: intent.blobs.clone().into(),
            object_names: Default::default(),
        }
    }

    pub fn for_intent(self) -> (InstructionsV1, BlobsV1) {
        (self.instructions.into(), self.blobs.into())
    }
}

/// A decompile target for old manifests which have been persisted.
#[derive(Debug, Clone, Default, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct LegacyTransactionManifestV1 {
    pub instructions: Vec<InstructionV1>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    #[sbor(skip)] // For backwards compatibility, this isn't persisted
    pub object_names: ManifestObjectNames,
}

impl From<LegacyTransactionManifestV1> for TransactionManifestV1 {
    fn from(value: LegacyTransactionManifestV1) -> Self {
        Self {
            instructions: value.instructions,
            blobs: value.blobs,
            object_names: value.object_names,
        }
    }
}
