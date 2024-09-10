use crate::internal_prelude::*;

pub trait BuildableManifest: ReadableManifest + ManifestEncode + Default + Eq + Debug {
    fn add_instruction(&mut self, instruction: Self::Instruction);
    fn add_blob(&mut self, hash: Hash, content: Vec<u8>);
    fn set_names(&mut self, names: KnownManifestObjectNames);
    fn add_child_subintent(&mut self, _hash: SubintentHash) -> Result<(), ManifestBuildError> {
        Err(ManifestBuildError::ChildSubintentsUnsupportedByManifestByManifestType)
    }
    fn add_preallocated_address(
        &mut self,
        _preallocated: PreAllocatedAddress,
    ) -> Result<(), ManifestBuildError> {
        Err(ManifestBuildError::PreallocatedAddressesUnsupportedByManifestType)
    }
    fn preallocation_count(&self) -> usize {
        0
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ManifestBuildError {
    ChildSubintentsUnsupportedByManifestByManifestType,
    PreallocatedAddressesUnsupportedByManifestType,
}

/// A trait indicating the manifest supports children.
/// In that case, it's expected `add_child_subintent`` does not error.
pub trait BuildableManifestSupportingChildren: BuildableManifest {}

/// A trait indicating the manifest supports children.
/// In that case, it's expected `add_preallocated_address` should not error.
pub trait BuildableManifestSupportingPreallocatedAddresses: BuildableManifest {}

/// A trait indicating the manifest has a parent
pub trait BuildableManifestWithParent: BuildableManifest {}

pub trait ReadableManifest {
    type Instruction: ManifestInstructionSet;
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
