use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// This isn't actually embedded as a model - it's just a useful model which we use
// in eg the manifest builder
//=================================================================================

/// Can be built with a [`ManifestV2Builder`]
#[derive(Debug, Clone, Default, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct SubintentManifestV2 {
    pub instructions: Vec<InstructionV2>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    pub children: IndexSet<ChildSubintentSpecifier>,
    pub object_names: ManifestObjectNames,
}

impl ReadableManifestBase for SubintentManifestV2 {
    fn is_subintent(&self) -> bool {
        true
    }

    fn get_blobs<'a>(&'a self) -> impl Iterator<Item = (&'a Hash, &'a Vec<u8>)> {
        self.blobs.iter()
    }

    fn get_child_subintent_hashes<'a>(
        &'a self,
    ) -> impl ExactSizeIterator<Item = &'a ChildSubintentSpecifier> {
        self.children.iter()
    }

    fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
        self.object_names.as_ref()
    }
}

impl TypedReadableManifest for SubintentManifestV2 {
    type Instruction = InstructionV2;

    fn get_typed_instructions(&self) -> &[Self::Instruction] {
        &self.instructions
    }
}

impl BuildableManifestWithParent for SubintentManifestV2 {}

impl BuildableManifestSupportingChildren for SubintentManifestV2 {}

impl BuildableManifest for SubintentManifestV2 {
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
        if !self.children.insert(ChildSubintentSpecifier { hash }) {
            return Err(ManifestBuildError::DuplicateChildSubintentHash);
        }
        Ok(())
    }

    fn default_test_execution_config_type(&self) -> DefaultTestExecutionConfigType {
        DefaultTestExecutionConfigType::Test
    }

    fn into_executable_with_proofs(
        self,
        _nonce: u32,
        _initial_proofs: BTreeSet<NonFungibleGlobalId>,
        _validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, String> {
        Err("A subintent manifest is not executable by itself. See the docs on `TestTransaction::new_v2_builder` for an alternative approach, to wrap the manifest in a parent test transaction.".to_string())
    }
}

impl SubintentManifestV2 {
    pub fn from_intent_core(intent: &IntentCoreV2) -> Self {
        Self {
            instructions: intent.instructions.clone().into(),
            blobs: intent.blobs.clone().into(),
            children: intent.children.children.clone(),
            object_names: ManifestObjectNames::Unknown,
        }
    }

    pub fn for_intent(self) -> (InstructionsV2, BlobsV1, ChildSubintentSpecifiersV2) {
        (
            self.instructions.into(),
            self.blobs.into(),
            ChildSubintentSpecifiersV2 {
                children: self.children,
            },
        )
    }

    pub fn for_intent_with_names(
        self,
    ) -> (
        InstructionsV2,
        BlobsV1,
        ChildSubintentSpecifiersV2,
        ManifestObjectNames,
    ) {
        (
            self.instructions.into(),
            self.blobs.into(),
            ChildSubintentSpecifiersV2 {
                children: self.children,
            },
            self.object_names,
        )
    }
}
