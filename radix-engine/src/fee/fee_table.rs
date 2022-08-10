use scrypto::core::{
    AuthZoneFnIdentifier, BucketFnIdentifier, ComponentFnIdentifier, FnIdentifier,
    NativeFnIdentifier, PackageFnIdentifier, ProofFnIdentifier, SystemFnIdentifier,
    TransactionProcessorFnIdentifier, VaultFnIdentifier, WorktopFnIdentifier,
};
use scrypto::prelude::ResourceManagerFnIdentifier;
use scrypto::{core::Receiver, values::ScryptoValue};

use crate::wasm::{InstructionCostRules, WasmMeteringParams};

pub enum SystemApiCostingEntry<'a> {
    /*
     * Invocation
     */
    /// Invokes a function, native or wasm.
    InvokeFunction {
        fn_identifier: FnIdentifier,
        input: &'a ScryptoValue,
    },
    /// Invokes a method, native or wasm.
    InvokeMethod {
        receiver: Receiver,
        input: &'a ScryptoValue,
    },

    /*
     * RENode
     */
    /// Creates a RENode.
    CreateNode { size: u32 },
    /// Drops a RENode
    DropNode { size: u32 },
    /// Globalizes a RENode.
    GlobalizeNode { size: u32 },

    /*
     * Substate
     */
    /// Borrows a substate
    BorrowSubstate { loaded: bool, size: u32 },
    /// Returns a substate.
    ReturnSubstate { size: u32 },
    /// Reads the data of a Substate
    TakeSubstate { size: u32 },
    /// Reads the data of a Substate
    ReadSubstate { size: u32 },
    /// Updates the data of a Substate
    WriteSubstate { size: u32 },

    /*
     * Misc
     */
    /// Reads the current epoch.
    ReadEpoch,
    /// Read the transaction hash.
    ReadTransactionHash,
    /// Read the transaction network.
    ReadTransactionNetwork,
    /// Generates a UUID.
    GenerateUuid,
    /// Emits a log.
    EmitLog { size: u32 },
    /// Checks if an access rule can be satisfied by the given proofs.
    CheckAccessRule { size: u32 },
}

pub struct FeeTable {
    tx_base_fee: u32,
    tx_decoding_per_byte: u32,
    tx_manifest_verification_per_byte: u32,
    tx_signature_verification_per_sig: u32,
    fixed_low: u32,
    fixed_medium: u32,
    fixed_high: u32,
    wasm_instantiation_per_byte: u32,
    wasm_metering_params: WasmMeteringParams,
}

impl FeeTable {
    pub fn new() -> Self {
        Self {
            tx_base_fee: 10_000,
            tx_decoding_per_byte: 3, // TODO: linear costing is suitable for PUBLISH_PACKAGE manifest; need to bill "blobs" separately
            tx_manifest_verification_per_byte: 1,
            tx_signature_verification_per_sig: 3750,
            wasm_instantiation_per_byte: 1, // TODO: this is currently costing too much!!!
            fixed_low: 100,
            fixed_medium: 500,
            fixed_high: 1000,
            wasm_metering_params: WasmMeteringParams::new(
                InstructionCostRules::tiered(1, 5, 10, 5000),
                512,
            ),
        }
    }

    pub fn tx_base_fee(&self) -> u32 {
        self.tx_base_fee
    }

    pub fn tx_decoding_per_byte(&self) -> u32 {
        self.tx_decoding_per_byte
    }

    pub fn tx_manifest_verification_per_byte(&self) -> u32 {
        self.tx_manifest_verification_per_byte
    }

    pub fn tx_signature_verification_per_sig(&self) -> u32 {
        self.tx_signature_verification_per_sig
    }

    pub fn wasm_instantiation_per_byte(&self) -> u32 {
        self.wasm_instantiation_per_byte
    }

    pub fn wasm_metering_params(&self) -> WasmMeteringParams {
        self.wasm_metering_params.clone()
    }

    pub fn run_method_cost(
        &self,
        receiver: Option<Receiver>,
        fn_identifier: &FnIdentifier,
        input: &ScryptoValue,
    ) -> u32 {
        match fn_identifier {
            FnIdentifier::Native(native_identifier) => {
                match native_identifier {
                    NativeFnIdentifier::TransactionProcessor(transaction_processor_fn) => {
                        match transaction_processor_fn {
                            TransactionProcessorFnIdentifier::Run => self.fixed_high,
                        }
                    }
                    NativeFnIdentifier::Package(package_fn) => match package_fn {
                        PackageFnIdentifier::Publish => self.fixed_low + input.raw.len() as u32 * 2,
                    },
                    NativeFnIdentifier::AuthZone(auth_zone_ident) => {
                        match auth_zone_ident {
                            AuthZoneFnIdentifier::Pop => self.fixed_low,
                            AuthZoneFnIdentifier::Push => self.fixed_low,
                            AuthZoneFnIdentifier::CreateProof => self.fixed_high, // TODO: charge differently based on auth zone size and fungibility
                            AuthZoneFnIdentifier::CreateProofByAmount => self.fixed_high,
                            AuthZoneFnIdentifier::CreateProofByIds => self.fixed_high,
                            AuthZoneFnIdentifier::Clear => self.fixed_high,
                        }
                    }
                    NativeFnIdentifier::System(system_ident) => match system_ident {
                        SystemFnIdentifier::GetCurrentEpoch => self.fixed_low,
                        SystemFnIdentifier::GetTransactionHash => self.fixed_low,
                        SystemFnIdentifier::SetEpoch => self.fixed_low,
                    },
                    NativeFnIdentifier::Bucket(bucket_ident) => match bucket_ident {
                        BucketFnIdentifier::Take => self.fixed_medium,
                        BucketFnIdentifier::TakeNonFungibles => self.fixed_medium,
                        BucketFnIdentifier::GetNonFungibleIds => self.fixed_medium,
                        BucketFnIdentifier::Put => self.fixed_medium,
                        BucketFnIdentifier::GetAmount => self.fixed_low,
                        BucketFnIdentifier::GetResourceAddress => self.fixed_low,
                        BucketFnIdentifier::CreateProof => self.fixed_low,
                        BucketFnIdentifier::Burn => self.fixed_medium,
                    },
                    NativeFnIdentifier::Proof(proof_ident) => match proof_ident {
                        ProofFnIdentifier::GetAmount => self.fixed_low,
                        ProofFnIdentifier::GetNonFungibleIds => self.fixed_low,
                        ProofFnIdentifier::GetResourceAddress => self.fixed_low,
                        ProofFnIdentifier::Clone => self.fixed_low,
                        ProofFnIdentifier::Drop => self.fixed_medium,
                    },
                    NativeFnIdentifier::ResourceManager(resource_manager_ident) => {
                        match resource_manager_ident {
                            ResourceManagerFnIdentifier::Create => self.fixed_high, // TODO: more investigation about fungibility
                            ResourceManagerFnIdentifier::UpdateAuth => self.fixed_medium,
                            ResourceManagerFnIdentifier::LockAuth => self.fixed_medium,
                            ResourceManagerFnIdentifier::CreateVault => self.fixed_medium,
                            ResourceManagerFnIdentifier::CreateBucket => self.fixed_medium,
                            ResourceManagerFnIdentifier::Mint => self.fixed_high,
                            ResourceManagerFnIdentifier::GetMetadata => self.fixed_low,
                            ResourceManagerFnIdentifier::GetResourceType => self.fixed_low,
                            ResourceManagerFnIdentifier::GetTotalSupply => self.fixed_low,
                            ResourceManagerFnIdentifier::UpdateMetadata => self.fixed_medium,
                            ResourceManagerFnIdentifier::UpdateNonFungibleData => self.fixed_medium,
                            ResourceManagerFnIdentifier::NonFungibleExists => self.fixed_low,
                            ResourceManagerFnIdentifier::GetNonFungible => self.fixed_medium,
                        }
                    }
                    NativeFnIdentifier::Worktop(worktop_ident) => match worktop_ident {
                        WorktopFnIdentifier::Put => self.fixed_medium,
                        WorktopFnIdentifier::TakeAmount => self.fixed_medium,
                        WorktopFnIdentifier::TakeAll => self.fixed_medium,
                        WorktopFnIdentifier::TakeNonFungibles => self.fixed_medium,
                        WorktopFnIdentifier::AssertContains => self.fixed_low,
                        WorktopFnIdentifier::AssertContainsAmount => self.fixed_low,
                        WorktopFnIdentifier::AssertContainsNonFungibles => self.fixed_low,
                        WorktopFnIdentifier::Drain => self.fixed_low,
                    },
                    NativeFnIdentifier::Component(component_ident) => match component_ident {
                        ComponentFnIdentifier::AddAccessCheck => self.fixed_medium,
                    },
                    NativeFnIdentifier::Vault(vault_ident) => {
                        match vault_ident {
                            VaultFnIdentifier::Put => self.fixed_medium,
                            VaultFnIdentifier::Take => self.fixed_medium, // TODO: revisit this if vault is not loaded in full
                            VaultFnIdentifier::TakeNonFungibles => self.fixed_medium,
                            VaultFnIdentifier::GetAmount => self.fixed_low,
                            VaultFnIdentifier::GetResourceAddress => self.fixed_low,
                            VaultFnIdentifier::GetNonFungibleIds => self.fixed_medium,
                            VaultFnIdentifier::CreateProof => self.fixed_high,
                            VaultFnIdentifier::CreateProofByAmount => self.fixed_high,
                            VaultFnIdentifier::CreateProofByIds => self.fixed_high,
                            VaultFnIdentifier::LockFee => self.fixed_medium,
                            VaultFnIdentifier::LockContingentFee => self.fixed_medium,
                        }
                    }
                }
            }
            FnIdentifier::Scrypto { .. } => {
                match receiver {
                    Some(..) => self.fixed_high,
                    None => 0, // Costing is through instrumentation // TODO: Josh question, why only through instrumentation?
                }
            }
        }
    }

    pub fn system_api_cost(&self, entry: SystemApiCostingEntry) -> u32 {
        match entry {
            SystemApiCostingEntry::InvokeFunction { input, .. } => {
                self.fixed_low + (5 * input.raw.len() + 10 * input.value_count()) as u32
            }
            SystemApiCostingEntry::InvokeMethod { input, .. } => {
                self.fixed_low + (5 * input.raw.len() + 10 * input.value_count()) as u32
            }

            SystemApiCostingEntry::CreateNode { .. } => self.fixed_medium,
            SystemApiCostingEntry::DropNode { .. } => self.fixed_medium,
            SystemApiCostingEntry::GlobalizeNode { size } => self.fixed_high + 200 * size,

            SystemApiCostingEntry::BorrowSubstate { loaded, size } => {
                if loaded {
                    self.fixed_high
                } else {
                    self.fixed_low + 100 * size
                }
            }
            SystemApiCostingEntry::ReturnSubstate { size } => self.fixed_low + 100 * size,
            SystemApiCostingEntry::TakeSubstate { .. } => self.fixed_medium,
            SystemApiCostingEntry::ReadSubstate { .. } => self.fixed_medium,
            SystemApiCostingEntry::WriteSubstate { .. } => self.fixed_medium,

            SystemApiCostingEntry::ReadEpoch => self.fixed_low,
            SystemApiCostingEntry::ReadTransactionHash => self.fixed_low,
            SystemApiCostingEntry::ReadTransactionNetwork => self.fixed_low,
            SystemApiCostingEntry::GenerateUuid => self.fixed_low,
            SystemApiCostingEntry::EmitLog { size } => self.fixed_low + 10 * size,
            SystemApiCostingEntry::CheckAccessRule { .. } => self.fixed_medium,
        }
    }
}
