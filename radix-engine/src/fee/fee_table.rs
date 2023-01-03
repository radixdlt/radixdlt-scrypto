use radix_engine_interface::api::types::{
    AccessRulesChainMethod, AuthZoneStackMethod, BucketMethod, ClockFunction, ClockMethod,
    ComponentFunction, ComponentMethod, EpochManagerFunction, EpochManagerMethod, LoggerMethod,
    MetadataMethod, NativeFunction, NativeMethod, PackageFunction, PackageMethod, ProofMethod,
    ResourceManagerFunction, ResourceManagerMethod, TransactionHashMethod,
    TransactionProcessorFunction, VaultMethod, WorktopMethod,
};

pub enum SystemApiCostingEntry {
    /*
     * Invocation
     */
    Invoke {
        input_size: u32,
        ownership_count: u32,
    },

    /*
     * RENode
     */
    ReadOwnedNodes,
    /// Creates a RENode.
    CreateNode {
        size: u32,
    },
    /// Drops a RENode
    DropNode {
        size: u32,
    },

    /*
     * Substate
     */
    /// Borrows a substate
    BorrowSubstate {
        loaded: bool,
        size: u32,
    },
    LockSubstate {
        size: u32,
    },
    /// Reads the data of a Substate
    ReadSubstate {
        size: u32,
    },
    /// Updates the data of a Substate
    WriteSubstate {
        size: u32,
    },
    DropLock,

    /*
     * Misc
     */
    /// Reads blob in transaction
    ReadBlob {
        size: u32,
    },
}

pub struct FeeTable {
    tx_base_fee: u32,
    tx_payload_cost_per_byte: u32,
    tx_signature_verification_per_sig: u32,
    tx_blob_price_per_byte: u32,
    fixed_low: u32,
    fixed_medium: u32,
    fixed_high: u32,
    wasm_instantiation_per_byte: u32,
}

impl FeeTable {
    pub fn new() -> Self {
        Self {
            tx_base_fee: 10_000,
            tx_payload_cost_per_byte: 1,
            tx_signature_verification_per_sig: 3750,
            tx_blob_price_per_byte: 1,
            wasm_instantiation_per_byte: 0, // TODO: Re-enable WASM instantiation cost if it's unavoidable
            fixed_low: 100,
            fixed_medium: 500,
            fixed_high: 1000,
        }
    }

    pub fn tx_base_fee(&self) -> u32 {
        self.tx_base_fee
    }

    pub fn tx_payload_cost_per_byte(&self) -> u32 {
        self.tx_payload_cost_per_byte
    }

    pub fn tx_signature_verification_per_sig(&self) -> u32 {
        self.tx_signature_verification_per_sig
    }

    pub fn tx_blob_price_per_byte(&self) -> u32 {
        self.tx_blob_price_per_byte
    }

    pub fn wasm_instantiation_per_byte(&self) -> u32 {
        self.wasm_instantiation_per_byte
    }

    pub fn run_native_function_cost(&self, native_function: &NativeFunction) -> u32 {
        match native_function {
            NativeFunction::Component(component_func) => match component_func {
                ComponentFunction::Globalize => self.fixed_high,
                ComponentFunction::GlobalizeWithOwner => self.fixed_high,
            },
            NativeFunction::TransactionProcessor(transaction_processor_fn) => {
                match transaction_processor_fn {
                    TransactionProcessorFunction::Run => self.fixed_high,
                }
            }
            NativeFunction::Package(package_fn) => match package_fn {
                PackageFunction::Publish => self.fixed_low,
            },
            NativeFunction::EpochManager(epoch_manager_fn) => match epoch_manager_fn {
                EpochManagerFunction::Create => self.fixed_low,
            },
            NativeFunction::Clock(clock_fn) => match clock_fn {
                ClockFunction::Create => self.fixed_low,
            },
            NativeFunction::ResourceManager(resource_manager_ident) => {
                match resource_manager_ident {
                    ResourceManagerFunction::Create => self.fixed_high, // TODO: more investigation about fungibility
                    ResourceManagerFunction::BurnBucket => self.fixed_low,
                }
            }
        }
    }

    pub fn run_native_method_cost(&self, native_method: &NativeMethod) -> u32 {
        match native_method {
            NativeMethod::AuthZoneStack(auth_zone_ident) => {
                match auth_zone_ident {
                    AuthZoneStackMethod::Pop => self.fixed_low,
                    AuthZoneStackMethod::Push => self.fixed_low,
                    AuthZoneStackMethod::CreateProof => self.fixed_high, // TODO: charge differently based on auth zone size and fungibility
                    AuthZoneStackMethod::CreateProofByAmount => self.fixed_high,
                    AuthZoneStackMethod::CreateProofByIds => self.fixed_high,
                    AuthZoneStackMethod::Clear => self.fixed_high,
                    AuthZoneStackMethod::Drain => self.fixed_high,
                    AuthZoneStackMethod::AssertAccessRule => self.fixed_high,
                }
            }
            NativeMethod::EpochManager(epoch_manager_method) => match epoch_manager_method {
                EpochManagerMethod::GetCurrentEpoch => self.fixed_low,
                EpochManagerMethod::SetEpoch => self.fixed_low,
            },
            NativeMethod::Clock(clock_method) => match clock_method {
                ClockMethod::SetCurrentTime => self.fixed_low,
                ClockMethod::GetCurrentTime => self.fixed_high,
                ClockMethod::CompareCurrentTime => self.fixed_high,
            },
            NativeMethod::Bucket(bucket_ident) => match bucket_ident {
                BucketMethod::Take => self.fixed_medium,
                BucketMethod::TakeNonFungibles => self.fixed_medium,
                BucketMethod::GetNonFungibleIds => self.fixed_medium,
                BucketMethod::Put => self.fixed_medium,
                BucketMethod::GetAmount => self.fixed_low,
                BucketMethod::GetResourceAddress => self.fixed_low,
                BucketMethod::CreateProof => self.fixed_low,
            },
            NativeMethod::Proof(proof_ident) => match proof_ident {
                ProofMethod::GetAmount => self.fixed_low,
                ProofMethod::GetNonFungibleIds => self.fixed_low,
                ProofMethod::GetResourceAddress => self.fixed_low,
                ProofMethod::Clone => self.fixed_low,
            },
            NativeMethod::ResourceManager(resource_manager_ident) => match resource_manager_ident {
                ResourceManagerMethod::UpdateVaultAuth => self.fixed_medium,
                ResourceManagerMethod::LockAuth => self.fixed_medium,
                ResourceManagerMethod::CreateVault => self.fixed_medium,
                ResourceManagerMethod::CreateBucket => self.fixed_medium,
                ResourceManagerMethod::Mint => self.fixed_high,
                ResourceManagerMethod::GetResourceType => self.fixed_low,
                ResourceManagerMethod::GetTotalSupply => self.fixed_low,
                ResourceManagerMethod::UpdateNonFungibleData => self.fixed_medium,
                ResourceManagerMethod::NonFungibleExists => self.fixed_low,
                ResourceManagerMethod::GetNonFungible => self.fixed_medium,
                ResourceManagerMethod::Burn => self.fixed_medium,
            },
            NativeMethod::Worktop(worktop_ident) => match worktop_ident {
                WorktopMethod::Put => self.fixed_medium,
                WorktopMethod::TakeAmount => self.fixed_medium,
                WorktopMethod::TakeAll => self.fixed_medium,
                WorktopMethod::TakeNonFungibles => self.fixed_medium,
                WorktopMethod::AssertContains => self.fixed_low,
                WorktopMethod::AssertContainsAmount => self.fixed_low,
                WorktopMethod::AssertContainsNonFungibles => self.fixed_low,
                WorktopMethod::Drain => self.fixed_low,
            },
            NativeMethod::Logger(logger_method) => match logger_method {
                LoggerMethod::Log => self.fixed_low,
            },
            NativeMethod::AccessRulesChain(component_ident) => match component_ident {
                AccessRulesChainMethod::AddAccessCheck => self.fixed_low,
                AccessRulesChainMethod::SetMethodAccessRule => self.fixed_low,
                AccessRulesChainMethod::SetMethodMutability => self.fixed_low,
                AccessRulesChainMethod::SetGroupAccessRule => self.fixed_low,
                AccessRulesChainMethod::SetGroupMutability => self.fixed_low,
                AccessRulesChainMethod::GetLength => self.fixed_low,
            },
            NativeMethod::Metadata(metadata_method) => match metadata_method {
                MetadataMethod::Set => self.fixed_low,
                MetadataMethod::Get => self.fixed_low,
            },
            NativeMethod::Component(method_ident) => match method_ident {
                ComponentMethod::SetRoyaltyConfig => self.fixed_medium,
                ComponentMethod::ClaimRoyalty => self.fixed_medium,
            },
            NativeMethod::Package(method_ident) => match method_ident {
                PackageMethod::SetRoyaltyConfig => self.fixed_medium,
                PackageMethod::ClaimRoyalty => self.fixed_medium,
            },
            NativeMethod::Vault(vault_ident) => {
                match vault_ident {
                    VaultMethod::Put => self.fixed_medium,
                    VaultMethod::Take => self.fixed_medium, // TODO: revisit this if vault is not loaded in full
                    VaultMethod::TakeNonFungibles => self.fixed_medium,
                    VaultMethod::GetAmount => self.fixed_low,
                    VaultMethod::GetResourceAddress => self.fixed_low,
                    VaultMethod::GetNonFungibleIds => self.fixed_medium,
                    VaultMethod::CreateProof => self.fixed_high,
                    VaultMethod::CreateProofByAmount => self.fixed_high,
                    VaultMethod::CreateProofByIds => self.fixed_high,
                    VaultMethod::LockFee => self.fixed_medium,
                    VaultMethod::Recall => self.fixed_low,
                    VaultMethod::RecallNonFungibles => self.fixed_low,
                }
            }
            NativeMethod::TransactionHash(ident) => match ident {
                TransactionHashMethod::Get => self.fixed_low,
                TransactionHashMethod::GenerateUuid => self.fixed_low,
            },
        }
    }

    pub fn system_api_cost(&self, entry: SystemApiCostingEntry) -> u32 {
        match entry {
            SystemApiCostingEntry::Invoke {
                input_size,
                ownership_count,
                ..
            } => self.fixed_low + (5 * input_size + 10 * ownership_count) as u32,

            SystemApiCostingEntry::ReadOwnedNodes => self.fixed_low,
            SystemApiCostingEntry::CreateNode { .. } => self.fixed_medium,
            SystemApiCostingEntry::DropNode { .. } => self.fixed_medium,

            SystemApiCostingEntry::BorrowSubstate { loaded, size } => {
                if loaded {
                    self.fixed_high
                } else {
                    self.fixed_low + 100 * size
                }
            }
            SystemApiCostingEntry::LockSubstate { .. } => self.fixed_low,
            SystemApiCostingEntry::ReadSubstate { .. } => self.fixed_medium,
            SystemApiCostingEntry::WriteSubstate { .. } => self.fixed_medium,
            SystemApiCostingEntry::DropLock => self.fixed_low,

            SystemApiCostingEntry::ReadBlob { size } => self.fixed_low + size,
        }
    }
}
