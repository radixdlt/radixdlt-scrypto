use crate::internal_prelude::*;

pub trait BuildableManifest:
    TypedReadableManifest
    + Into<AnyManifest>
    + TryFrom<AnyManifest, Error = ()>
    + ManifestEncode
    + Default
    + Eq
    + Debug
{
    fn add_instruction(&mut self, instruction: Self::Instruction);
    fn add_blob(&mut self, hash: Hash, content: Vec<u8>);
    fn set_names(&mut self, names: KnownManifestObjectNames);
    fn set_names_if_known(&mut self, names: impl Into<ManifestObjectNames>) {
        match names.into() {
            ManifestObjectNames::Unknown => {}
            ManifestObjectNames::Known(known_names) => self.set_names(known_names),
        };
    }
    fn add_child_subintent(&mut self, _hash: SubintentHash) -> Result<(), ManifestBuildError> {
        Err(ManifestBuildError::ChildSubintentsUnsupportedByManifestType)
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

    fn default_test_execution_config_type(&self) -> DefaultTestExecutionConfigType;
    fn into_executable_with_proofs(
        self,
        nonce: u32,
        initial_proofs: BTreeSet<NonFungibleGlobalId>,
        validator: &TransactionValidator,
    ) -> Result<ExecutableTransaction, String>;

    fn to_raw(self) -> Result<RawManifest, EncodeError> {
        let any_manifest: AnyManifest = self.into();
        any_manifest.to_raw()
    }

    fn from_raw(raw: &RawManifest) -> Result<Self, String> {
        let any_manifest = AnyManifest::from_raw(raw)
            .map_err(|err| format!("Could not decode as `AnyManifest`: {err:?}"))?;
        Self::try_from(any_manifest)
            .map_err(|()| format!("Encoded manifest was not of the correct type"))
    }

    fn decode_arbitrary(bytes: impl AsRef<[u8]>) -> Result<Self, String> {
        let any_manifest = AnyManifest::attempt_decode_from_arbitrary_payload(bytes.as_ref())?;
        Self::try_from(any_manifest)
            .map_err(|()| format!("Encoded manifest was not of the correct type"))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum DefaultTestExecutionConfigType {
    Notarized,
    System,
    Test,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ManifestBuildError {
    ChildSubintentsUnsupportedByManifestType,
    PreallocatedAddressesUnsupportedByManifestType,
}

/// A trait indicating the manifest supports children.
/// In that case, it's expected `add_child_subintent` does not error.
pub trait BuildableManifestSupportingChildren: BuildableManifest {}

/// A trait indicating the manifest supports children.
/// In that case, it's expected `add_preallocated_address` should not error.
pub trait BuildableManifestSupportingPreallocatedAddresses: BuildableManifest {}

/// A trait indicating the manifest has a parent
pub trait BuildableManifestWithParent: BuildableManifest {}

pub trait TypedReadableManifest: ReadableManifestBase {
    type Instruction: ManifestInstructionSet;
    fn get_typed_instructions(&self) -> &[Self::Instruction];
}

pub trait ReadableManifestBase {
    fn is_subintent(&self) -> bool;
    fn get_blobs<'a>(&'a self) -> impl Iterator<Item = (&'a Hash, &'a Vec<u8>)>;
    fn get_preallocated_addresses(&self) -> &[PreAllocatedAddress] {
        &NO_PREALLOCATED_ADDRESSES
    }
    fn get_child_subintents(&self) -> &[ChildSubintent] {
        &NO_CHILD_SUBINTENTS
    }
    fn get_known_object_names_ref(&self) -> ManifestObjectNamesRef;
}

/// An object-safe  of ReadableManifest
pub trait ReadableManifest: ReadableManifestBase {
    fn iter_instruction_effects(&self) -> impl Iterator<Item = ManifestInstructionEffect>;
    fn iter_cloned_instructions(&self) -> impl Iterator<Item = AnyInstruction>;
    fn instruction_count(&self) -> usize;
    /// Panics if index is out of bounds
    fn instruction_effect(&self, index: usize) -> ManifestInstructionEffect;

    fn validate(&self, ruleset: ValidationRuleset) -> Result<(), ManifestValidationError> {
        StaticManifestInterpreter::new(ruleset, self).validate()
    }
}

impl<T: TypedReadableManifest + ?Sized> ReadableManifest for T {
    fn iter_instruction_effects(&self) -> impl Iterator<Item = ManifestInstructionEffect> {
        self.get_typed_instructions().iter().map(|i| i.effect())
    }

    fn iter_cloned_instructions(&self) -> impl Iterator<Item = AnyInstruction> {
        self.get_typed_instructions()
            .iter()
            .map(|i| i.clone().into())
    }

    fn instruction_count(&self) -> usize {
        self.get_typed_instructions().len()
    }

    fn instruction_effect(&self, index: usize) -> ManifestInstructionEffect {
        self.get_typed_instructions()[index].effect()
    }
}

static NO_PREALLOCATED_ADDRESSES: [PreAllocatedAddress; 0] = [];
static NO_CHILD_SUBINTENTS: [ChildSubintent; 0] = [];
