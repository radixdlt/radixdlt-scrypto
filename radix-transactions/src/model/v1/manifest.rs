use super::*;
use crate::internal_prelude::*;
use core::ops::Deref;

/// ## Using AnyTransactionManifest
/// Typically you'll have a method `my_method` which takes a &impl ReadableManifest.
/// Ideally, we could have an apply method which lets you use this method trivially with
/// an [`AnyTransactionManifest`] - but this would require a function constraint of
/// `F: for<R: ReadableManifest> FnOnce<R, Output>` - which uses higher order type-based trait bounds
/// which don't exist yet (https://github.com/rust-lang/rust/issues/108185).
///
/// So instead, the convention is to also create an `my_method_any` with a switch statement in.
pub enum AnyTransactionManifest {
    V1(TransactionManifestV1),
    SystemV1(SystemTransactionManifestV1),
    V2(TransactionManifestV2),
}

impl From<TransactionManifestV1> for AnyTransactionManifest {
    fn from(value: TransactionManifestV1) -> Self {
        Self::V1(value)
    }
}

impl From<SystemTransactionManifestV1> for AnyTransactionManifest {
    fn from(value: SystemTransactionManifestV1) -> Self {
        Self::SystemV1(value)
    }
}

impl From<TransactionManifestV2> for AnyTransactionManifest {
    fn from(value: TransactionManifestV2) -> Self {
        Self::V2(value)
    }
}

impl AnyTransactionManifest {
    pub fn get_blobs(&self) -> &IndexMap<Hash, Vec<u8>> {
        match self {
            AnyTransactionManifest::V1(m) => m.get_blobs(),
            AnyTransactionManifest::SystemV1(m) => m.get_blobs(),
            AnyTransactionManifest::V2(m) => m.get_blobs(),
        }
    }
}

pub trait BuildableManifest: ReadableManifest + ManifestEncode + Default {
    fn add_instruction(&mut self, instruction: Self::Instruction);
    fn add_blob(&mut self, hash: Hash, content: Vec<u8>);
    fn set_names(&mut self, names: KnownManifestObjectNames);
}

pub trait ReadableManifest {
    type Instruction: ManifestInstruction;
    fn get_instructions(&self) -> &[Self::Instruction];
    fn get_blobs(&self) -> &IndexMap<Hash, Vec<u8>>;
    fn get_preallocated_addresses(&self) -> &[PreAllocatedAddress] {
        &NO_PREALLOCATED_ADDRESSES
    }
    fn get_child_subintents(&self) -> &[SubintentHash] {
        &NO_CHILD_SUBINTENTS
    }
    fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef;

    fn validate(&self) -> Result<(), TransactionValidationError>;
}

static NO_PREALLOCATED_ADDRESSES: [PreAllocatedAddress; 0] = [];
static NO_CHILD_SUBINTENTS: [SubintentHash; 0] = [];

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

impl ReadableManifest for SystemTransactionManifestV1 {
    type Instruction = InstructionV1;

    fn get_instructions(&self) -> &[Self::Instruction] {
        &self.instructions
    }

    fn get_blobs(&self) -> &IndexMap<Hash, Vec<u8>> {
        &self.blobs
    }

    fn get_preallocated_addresses(&self) -> &[PreAllocatedAddress] {
        &self.preallocated_addresses
    }

    fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
        self.object_names.as_ref()
    }

    fn validate(&self) -> Result<(), TransactionValidationError> {
        NotarizedTransactionValidatorV1::validate_instructions_v1(&self.instructions)
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
}

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
}

/// Can be built with a [`ManifestV1Builder`]
#[derive(Debug, Clone, Default, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct TransactionManifestV1 {
    pub instructions: Vec<InstructionV1>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    #[sbor(skip)] // For backwards compatibility, this isn't persisted
    pub object_names: ManifestObjectNames,
}

impl ReadableManifest for TransactionManifestV1 {
    type Instruction = InstructionV1;

    fn get_instructions(&self) -> &[Self::Instruction] {
        &self.instructions
    }

    fn get_blobs(&self) -> &IndexMap<Hash, Vec<u8>> {
        &self.blobs
    }

    fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef {
        self.object_names.as_ref()
    }

    fn validate(&self) -> Result<(), TransactionValidationError> {
        NotarizedTransactionValidatorV1::validate_instructions_v1(&self.instructions)
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
}

impl TransactionManifestV1 {
    pub fn from_intent(intent: &IntentV1) -> Self {
        Self {
            instructions: intent.instructions.0.deref().clone(),
            blobs: intent
                .blobs
                .blobs
                .iter()
                .map(|blob| (hash(&blob.0), blob.0.clone()))
                .collect(),
            object_names: Default::default(),
        }
    }

    pub fn for_intent(self) -> (InstructionsV1, BlobsV1) {
        (self.instructions.into(), self.blobs.into())
    }
}

/// Can be built with a [`ManifestV2Builder`]
#[derive(Debug, Clone, Default, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct TransactionManifestV2 {
    pub instructions: Vec<InstructionV2>,
    pub blobs: IndexMap<Hash, Vec<u8>>,
    pub children: Vec<SubintentHash>,
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

    fn get_child_subintents(&self) -> &[SubintentHash] {
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
}

impl TransactionManifestV2 {
    pub fn from_intent(intent: &IntentCoreV2) -> Self {
        Self {
            instructions: intent.instructions.0.deref().clone(),
            blobs: intent
                .blobs
                .blobs
                .iter()
                .map(|blob| (hash(&blob.0), blob.0.clone()))
                .collect(),
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
