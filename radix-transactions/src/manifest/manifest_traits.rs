use crate::internal_prelude::*;

pub trait BuildableManifest: ReadableManifest + ManifestEncode + Default {
    fn add_instruction(&mut self, instruction: Self::Instruction);
    fn add_blob(&mut self, hash: Hash, content: Vec<u8>);
    fn set_names(&mut self, names: KnownManifestObjectNames);
}

pub trait BuildableManifestWithChildSupport: BuildableManifest {
    fn add_child_subintent(&mut self, hash: SubintentHash);
}

/// A trait indicating the manifest has a parent
pub trait BuildableManifestWithParent: BuildableManifest {}

pub trait ReadableManifest {
    type Instruction: ManifestInstruction;
    fn get_instructions(&self) -> &[Self::Instruction];
    fn get_blobs(&self) -> &IndexMap<Hash, Vec<u8>>;
    fn get_preallocated_addresses(&self) -> &[PreAllocatedAddress] {
        &NO_PREALLOCATED_ADDRESSES
    }
    fn get_child_subintents(&self) -> &[ChildSubintent] {
        &NO_CHILD_SUBINTENTS
    }
    fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef;

    fn validate(&self) -> Result<(), TransactionValidationError>;
}

static NO_PREALLOCATED_ADDRESSES: [PreAllocatedAddress; 0] = [];
static NO_CHILD_SUBINTENTS: [ChildSubintent; 0] = [];
