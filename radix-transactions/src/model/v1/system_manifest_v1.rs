use super::*;
use crate::internal_prelude::*;

//=================================================================================
// NOTE:
// This isn't actually embedded as a model - it's just a useful model which we use
// in eg the manifest builder
//=================================================================================

/// Can be built with a [`SystemV1ManifestBuilder`]
#[derive(Debug, Clone, Default, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct SystemTransactionManifestV1 {
    pub instructions: Vec<InstructionV1>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    pub preallocated_addresses: Vec<PreAllocatedAddress>,
    pub object_names: ManifestObjectNames,
}

impl ReadableManifestBase for SystemTransactionManifestV1 {
    fn is_subintent(&self) -> bool {
        false
    }

    fn get_blobs<'a>(&'a self) -> impl Iterator<Item = (&'a Hash, &'a Vec<u8>)> {
        self.blobs.iter()
    }

    fn get_preallocated_addresses(&self) -> &[PreAllocatedAddress] {
        &self.preallocated_addresses
    }

    fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
        self.object_names.as_ref()
    }
}

impl TypedReadableManifest for SystemTransactionManifestV1 {
    type Instruction = InstructionV1;

    fn get_typed_instructions(&self) -> &[Self::Instruction] {
        &self.instructions
    }
}

impl BuildableManifest for SystemTransactionManifestV1 {
    fn add_instruction(&mut self, instruction: Self::Instruction) {
        self.instructions.push(instruction)
    }

    fn add_blob(&mut self, hash: Hash, content: Vec<u8>) {
        self.blobs.insert(hash, content);
    }

    fn set_names(&mut self, names: KnownManifestObjectNames) {
        self.object_names = names.into()
    }

    fn add_preallocated_address(
        &mut self,
        preallocated: PreAllocatedAddress,
    ) -> Result<(), ManifestBuildError> {
        self.preallocated_addresses.push(preallocated);
        Ok(())
    }

    fn preallocation_count(&self) -> usize {
        self.preallocated_addresses.len()
    }

    fn default_test_execution_config_type(&self) -> DefaultTestExecutionConfigType {
        DefaultTestExecutionConfigType::System
    }

    fn into_executable_with_proofs(
        self,
        nonce: u32,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, String> {
        let unique_hash = hash(format!("System txn: {}", nonce));
        self.into_transaction(unique_hash)
            .with_proofs_ref(initial_proofs)
            .into_executable(&validator)
            .map_err(|err| format!("Could not prepare: {err:?}"))
    }
}

impl BuildableManifestSupportingPreallocatedAddresses for SystemTransactionManifestV1 {}

impl SystemTransactionManifestV1 {
    pub fn from_transaction(transaction: &SystemTransactionV1) -> Self {
        Self {
            instructions: transaction.instructions.clone().into(),
            blobs: transaction.blobs.clone().into(),
            preallocated_addresses: transaction.pre_allocated_addresses.clone(),
            object_names: ManifestObjectNames::Unknown,
        }
    }

    pub fn into_transaction(self, unique_hash: Hash) -> SystemTransactionV1 {
        SystemTransactionV1 {
            instructions: self.instructions.into(),
            blobs: self.blobs.into(),
            pre_allocated_addresses: self.preallocated_addresses,
            hash_for_execution: unique_hash,
        }
    }

    pub fn into_transaction_with_proofs(
        self,
        unique_hash: Hash,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
    ) -> SystemTransactionV1WithProofs<'static> {
        self.into_transaction(unique_hash)
            .with_proofs(initial_proofs)
    }
}
