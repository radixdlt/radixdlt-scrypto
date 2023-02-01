use radix_engine_interface::api::types::*;

pub enum SystemApiCostingEntry {
    /*
     * Invocation
     */
    Invoke {
        input_size: u32,
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
            tx_base_fee: 50_000,
            tx_payload_cost_per_byte: 5,
            tx_signature_verification_per_sig: 100_000,
            tx_blob_price_per_byte: 5,
            wasm_instantiation_per_byte: 1, // TODO: Re-enable WASM instantiation cost if it's unavoidable
            fixed_low: 500,
            fixed_medium: 2500,
            fixed_high: 5000,
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

    pub fn run_native_fn_cost(&self, native_fn: &NativeFn) -> u32 {
        match native_fn {
            NativeFn::AuthZoneStack(auth_zone_ident) => {
                match auth_zone_ident {
                    AuthZoneStackFn::Pop => self.fixed_low,
                    AuthZoneStackFn::Push => self.fixed_low,
                    AuthZoneStackFn::CreateProof => self.fixed_high, // TODO: charge differently based on auth zone size and fungibility
                    AuthZoneStackFn::CreateProofByAmount => self.fixed_high,
                    AuthZoneStackFn::CreateProofByIds => self.fixed_high,
                    AuthZoneStackFn::Clear => self.fixed_high,
                    AuthZoneStackFn::Drain => self.fixed_high,
                    AuthZoneStackFn::AssertAccessRule => self.fixed_high,
                }
            }
            NativeFn::EpochManager(epoch_manager_method) => match epoch_manager_method {
                EpochManagerFn::Create => self.fixed_low,
                EpochManagerFn::GetCurrentEpoch => self.fixed_low,
                EpochManagerFn::NextRound => self.fixed_low,
                EpochManagerFn::SetEpoch => self.fixed_low,
                EpochManagerFn::UpdateValidator => self.fixed_low,
                EpochManagerFn::CreateValidator => self.fixed_low,
            },
            NativeFn::Validator(validator_fn) => match validator_fn {
                ValidatorFn::Register => self.fixed_low,
                ValidatorFn::Unregister => self.fixed_low,
                ValidatorFn::Stake => self.fixed_low,
                ValidatorFn::Unstake => self.fixed_low,
                ValidatorFn::ClaimXrd => self.fixed_low,
                ValidatorFn::UpdateKey => self.fixed_low,
                ValidatorFn::UpdateAcceptDelegatedStake => self.fixed_low,
            },
            NativeFn::Clock(clock_method) => match clock_method {
                ClockFn::Create => self.fixed_low,
                ClockFn::SetCurrentTime => self.fixed_low,
                ClockFn::GetCurrentTime => self.fixed_high,
                ClockFn::CompareCurrentTime => self.fixed_high,
            },
            NativeFn::Identity(identity_fn) => match identity_fn {
                IdentityFn::Create => self.fixed_low,
            },
            NativeFn::Bucket(bucket_ident) => match bucket_ident {
                BucketFn::Take => self.fixed_medium,
                BucketFn::TakeNonFungibles => self.fixed_medium,
                BucketFn::GetNonFungibleLocalIds => self.fixed_medium,
                BucketFn::Put => self.fixed_medium,
                BucketFn::GetAmount => self.fixed_low,
                BucketFn::GetResourceAddress => self.fixed_low,
                BucketFn::CreateProof => self.fixed_low,
            },
            NativeFn::Proof(proof_ident) => match proof_ident {
                ProofFn::GetAmount => self.fixed_low,
                ProofFn::GetNonFungibleLocalIds => self.fixed_low,
                ProofFn::GetResourceAddress => self.fixed_low,
                ProofFn::Clone => self.fixed_low,
            },
            NativeFn::ResourceManager(resource_manager_ident) => match resource_manager_ident {
                ResourceManagerFn::CreateNonFungible => self.fixed_high, // TODO: more investigation about fungibility
                ResourceManagerFn::CreateFungible => self.fixed_high, // TODO: more investigation about fungibility
                ResourceManagerFn::CreateNonFungibleWithInitialSupply => self.fixed_high, // TODO: more investigation about fungibility
                ResourceManagerFn::CreateUuidNonFungibleWithInitialSupply => self.fixed_high, // TODO: more investigation about fungibility
                ResourceManagerFn::CreateFungibleWithInitialSupply => self.fixed_high, // TODO: more investigation about fungibility
                ResourceManagerFn::BurnBucket => self.fixed_low,
                ResourceManagerFn::UpdateVaultAuth => self.fixed_medium,
                ResourceManagerFn::LockAuth => self.fixed_medium,
                ResourceManagerFn::CreateVault => self.fixed_medium,
                ResourceManagerFn::CreateBucket => self.fixed_medium,
                ResourceManagerFn::MintNonFungible => self.fixed_high,
                ResourceManagerFn::MintUuidNonFungible => self.fixed_high,
                ResourceManagerFn::MintFungible => self.fixed_high,
                ResourceManagerFn::GetResourceType => self.fixed_low,
                ResourceManagerFn::GetTotalSupply => self.fixed_low,
                ResourceManagerFn::UpdateNonFungibleData => self.fixed_medium,
                ResourceManagerFn::NonFungibleExists => self.fixed_low,
                ResourceManagerFn::GetNonFungible => self.fixed_medium,
                ResourceManagerFn::Burn => self.fixed_medium,
            },
            NativeFn::Worktop(worktop_ident) => match worktop_ident {
                WorktopFn::Put => self.fixed_medium,
                WorktopFn::TakeAmount => self.fixed_medium,
                WorktopFn::TakeAll => self.fixed_medium,
                WorktopFn::TakeNonFungibles => self.fixed_medium,
                WorktopFn::AssertContains => self.fixed_low,
                WorktopFn::AssertContainsAmount => self.fixed_low,
                WorktopFn::AssertContainsNonFungibles => self.fixed_low,
                WorktopFn::Drain => self.fixed_low,
            },
            NativeFn::Logger(logger_method) => match logger_method {
                LoggerFn::Log => self.fixed_low,
            },
            NativeFn::AccessRulesChain(component_ident) => match component_ident {
                AccessRulesChainFn::AddAccessCheck => self.fixed_low,
                AccessRulesChainFn::SetMethodAccessRule => self.fixed_low,
                AccessRulesChainFn::SetMethodMutability => self.fixed_low,
                AccessRulesChainFn::SetGroupAccessRule => self.fixed_low,
                AccessRulesChainFn::SetGroupMutability => self.fixed_low,
                AccessRulesChainFn::GetLength => self.fixed_low,
            },
            NativeFn::Metadata(metadata_method) => match metadata_method {
                MetadataFn::Set => self.fixed_low,
                MetadataFn::Get => self.fixed_low,
            },
            NativeFn::Component(method_ident) => match method_ident {
                ComponentFn::Globalize => self.fixed_high,
                ComponentFn::GlobalizeWithOwner => self.fixed_high,
                ComponentFn::SetRoyaltyConfig => self.fixed_medium,
                ComponentFn::ClaimRoyalty => self.fixed_medium,
            },
            NativeFn::Package(method_ident) => match method_ident {
                PackageFn::Publish => self.fixed_high,
                PackageFn::SetRoyaltyConfig => self.fixed_medium,
                PackageFn::ClaimRoyalty => self.fixed_medium,
            },
            NativeFn::Vault(vault_ident) => {
                match vault_ident {
                    VaultFn::Put => self.fixed_medium,
                    VaultFn::Take => self.fixed_medium, // TODO: revisit this if vault is not loaded in full
                    VaultFn::TakeNonFungibles => self.fixed_medium,
                    VaultFn::GetAmount => self.fixed_low,
                    VaultFn::GetResourceAddress => self.fixed_low,
                    VaultFn::GetNonFungibleLocalIds => self.fixed_medium,
                    VaultFn::CreateProof => self.fixed_high,
                    VaultFn::CreateProofByAmount => self.fixed_high,
                    VaultFn::CreateProofByIds => self.fixed_high,
                    VaultFn::LockFee => self.fixed_medium,
                    VaultFn::Recall => self.fixed_low,
                    VaultFn::RecallNonFungibles => self.fixed_low,
                }
            }
            NativeFn::TransactionRuntime(ident) => match ident {
                TransactionRuntimeFn::Get => self.fixed_low,
                TransactionRuntimeFn::GenerateUuid => self.fixed_low,
            },
            NativeFn::TransactionProcessor(transaction_processor_fn) => {
                match transaction_processor_fn {
                    TransactionProcessorFn::Run => self.fixed_high,
                }
            }
            NativeFn::AccessController(access_controller_fn) => match access_controller_fn {
                AccessControllerFn::CreateGlobal => self.fixed_low,

                AccessControllerFn::CreateProof => self.fixed_low,

                AccessControllerFn::InitiateRecoveryAsPrimary => self.fixed_low,
                AccessControllerFn::InitiateRecoveryAsRecovery => self.fixed_low,

                AccessControllerFn::QuickConfirmPrimaryRoleRecoveryProposal => self.fixed_low,
                AccessControllerFn::QuickConfirmRecoveryRoleRecoveryProposal => self.fixed_low,

                AccessControllerFn::TimedConfirmRecovery => self.fixed_low,

                AccessControllerFn::CancelPrimaryRoleRecoveryProposal => self.fixed_low,
                AccessControllerFn::CancelRecoveryRoleRecoveryProposal => self.fixed_low,

                AccessControllerFn::LockPrimaryRole => self.fixed_low,
                AccessControllerFn::UnlockPrimaryRole => self.fixed_low,

                AccessControllerFn::StopTimedRecovery => self.fixed_low,
            },
        }
    }

    pub fn system_api_cost(&self, entry: SystemApiCostingEntry) -> u32 {
        match entry {
            SystemApiCostingEntry::Invoke { input_size, .. } => {
                self.fixed_low + (5 * input_size) as u32
            }

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
