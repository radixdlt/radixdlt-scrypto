use crate::types::*;
use scrypto::core::{
    FnIdent, MethodFnIdent, MethodIdent, NativeFunctionFnIdent, ResourceManagerFunctionFnIdent,
    SystemFunctionFnIdent,
};

pub enum SystemApiCostingEntry<'a> {
    /*
     * Invocation
     */
    Invoke {
        function_identifier: FnIdent,
        input: &'a ScryptoValue,
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
    /// Globalizes a RENode.
    GlobalizeNode {
        size: u32,
    },
    /// Borrows a RENode.
    BorrowNode {
        loaded: bool,
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
    /// Returns a substate.
    ReturnSubstate {
        size: u32,
    },
    /// Takes a substate
    TakeSubstate {
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

    /*
     * Misc
     */
    /// Reads the current epoch.
    ReadEpoch,
    /// Reads the transaction hash.
    ReadTransactionHash,
    /// Reads blob in transaction
    ReadBlob {
        size: u32,
    },
    /// Generates a UUID.
    GenerateUuid,
    /// Emits a log.
    EmitLog {
        size: u32,
    },
}

pub struct FeeTable {
    tx_base_fee: u32,
    tx_manifest_decoding_per_byte: u32,
    tx_manifest_verification_per_byte: u32,
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
            tx_manifest_decoding_per_byte: 3,
            tx_manifest_verification_per_byte: 1,
            tx_signature_verification_per_sig: 3750,
            tx_blob_price_per_byte: 1,
            wasm_instantiation_per_byte: 1, // TODO: this is currently costing too much!!!
            fixed_low: 100,
            fixed_medium: 500,
            fixed_high: 1000,
        }
    }

    pub fn tx_base_fee(&self) -> u32 {
        self.tx_base_fee
    }

    pub fn tx_manifest_decoding_per_byte(&self) -> u32 {
        self.tx_manifest_decoding_per_byte
    }

    pub fn tx_manifest_verification_per_byte(&self) -> u32 {
        self.tx_manifest_verification_per_byte
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

    pub fn run_fn_cost(&self, fn_ident: &FnIdent, input: &ScryptoValue) -> u32 {
        match fn_ident {
            FnIdent::Function(function_ident) => {
                match function_ident {
                    FunctionIdent::Native(NativeFunctionFnIdent::TransactionProcessor(
                        transaction_processor_fn,
                    )) => match transaction_processor_fn {
                        TransactionProcessorFunctionFnIdent::Run => self.fixed_high,
                    },
                    FunctionIdent::Native(NativeFunctionFnIdent::Package(package_fn)) => {
                        match package_fn {
                            PackageFunctionFnIdent::Publish => {
                                self.fixed_low + input.raw.len() as u32 * 2
                            }
                        }
                    }
                    FunctionIdent::Native(NativeFunctionFnIdent::System(system_ident)) => {
                        match system_ident {
                            SystemFunctionFnIdent::Create => self.fixed_low,
                        }
                    }
                    FunctionIdent::Native(NativeFunctionFnIdent::ResourceManager(
                        resource_manager_ident,
                    )) => {
                        match resource_manager_ident {
                            ResourceManagerFunctionFnIdent::Create => self.fixed_high, // TODO: more investigation about fungibility
                        }
                    }
                    FunctionIdent::Scrypto { .. } => 0, // Costing is through instrumentation // TODO: Josh question, why only through instrumentation?
                }
            }
            FnIdent::Method(MethodIdent {
                fn_ident: method_fn_ident,
                ..
            }) => {
                match method_fn_ident {
                    MethodFnIdent::Native(NativeMethodFnIdent::AuthZone(auth_zone_ident)) => {
                        match auth_zone_ident {
                            AuthZoneMethodFnIdent::Pop => self.fixed_low,
                            AuthZoneMethodFnIdent::Push => self.fixed_low,
                            AuthZoneMethodFnIdent::CreateProof => self.fixed_high, // TODO: charge differently based on auth zone size and fungibility
                            AuthZoneMethodFnIdent::CreateProofByAmount => self.fixed_high,
                            AuthZoneMethodFnIdent::CreateProofByIds => self.fixed_high,
                            AuthZoneMethodFnIdent::Clear => self.fixed_high,
                            AuthZoneMethodFnIdent::Drain => self.fixed_high,
                        }
                    }
                    MethodFnIdent::Native(NativeMethodFnIdent::System(system_ident)) => {
                        match system_ident {
                            SystemMethodFnIdent::GetCurrentEpoch => self.fixed_low,
                            SystemMethodFnIdent::GetTransactionHash => self.fixed_low,
                            SystemMethodFnIdent::SetEpoch => self.fixed_low,
                        }
                    }
                    MethodFnIdent::Native(NativeMethodFnIdent::Bucket(bucket_ident)) => {
                        match bucket_ident {
                            BucketMethodFnIdent::Take => self.fixed_medium,
                            BucketMethodFnIdent::TakeNonFungibles => self.fixed_medium,
                            BucketMethodFnIdent::GetNonFungibleIds => self.fixed_medium,
                            BucketMethodFnIdent::Put => self.fixed_medium,
                            BucketMethodFnIdent::GetAmount => self.fixed_low,
                            BucketMethodFnIdent::GetResourceAddress => self.fixed_low,
                            BucketMethodFnIdent::CreateProof => self.fixed_low,
                            BucketMethodFnIdent::Burn => self.fixed_medium,
                        }
                    }
                    MethodFnIdent::Native(NativeMethodFnIdent::Proof(proof_ident)) => {
                        match proof_ident {
                            ProofMethodFnIdent::GetAmount => self.fixed_low,
                            ProofMethodFnIdent::GetNonFungibleIds => self.fixed_low,
                            ProofMethodFnIdent::GetResourceAddress => self.fixed_low,
                            ProofMethodFnIdent::Clone => self.fixed_low,
                            ProofMethodFnIdent::Drop => self.fixed_medium,
                        }
                    }
                    MethodFnIdent::Native(NativeMethodFnIdent::ResourceManager(
                        resource_manager_ident,
                    )) => match resource_manager_ident {
                        ResourceManagerMethodFnIdent::UpdateAuth => self.fixed_medium,
                        ResourceManagerMethodFnIdent::LockAuth => self.fixed_medium,
                        ResourceManagerMethodFnIdent::CreateVault => self.fixed_medium,
                        ResourceManagerMethodFnIdent::CreateBucket => self.fixed_medium,
                        ResourceManagerMethodFnIdent::Mint => self.fixed_high,
                        ResourceManagerMethodFnIdent::GetMetadata => self.fixed_low,
                        ResourceManagerMethodFnIdent::GetResourceType => self.fixed_low,
                        ResourceManagerMethodFnIdent::GetTotalSupply => self.fixed_low,
                        ResourceManagerMethodFnIdent::UpdateMetadata => self.fixed_medium,
                        ResourceManagerMethodFnIdent::UpdateNonFungibleData => self.fixed_medium,
                        ResourceManagerMethodFnIdent::NonFungibleExists => self.fixed_low,
                        ResourceManagerMethodFnIdent::GetNonFungible => self.fixed_medium,
                    },
                    MethodFnIdent::Native(NativeMethodFnIdent::Worktop(worktop_ident)) => {
                        match worktop_ident {
                            WorktopMethodFnIdent::Put => self.fixed_medium,
                            WorktopMethodFnIdent::TakeAmount => self.fixed_medium,
                            WorktopMethodFnIdent::TakeAll => self.fixed_medium,
                            WorktopMethodFnIdent::TakeNonFungibles => self.fixed_medium,
                            WorktopMethodFnIdent::AssertContains => self.fixed_low,
                            WorktopMethodFnIdent::AssertContainsAmount => self.fixed_low,
                            WorktopMethodFnIdent::AssertContainsNonFungibles => self.fixed_low,
                            WorktopMethodFnIdent::Drain => self.fixed_low,
                        }
                    }
                    MethodFnIdent::Native(NativeMethodFnIdent::Component(component_ident)) => {
                        match component_ident {
                            ComponentMethodFnIdent::AddAccessCheck => self.fixed_medium,
                        }
                    }
                    MethodFnIdent::Native(NativeMethodFnIdent::Vault(vault_ident)) => {
                        match vault_ident {
                            VaultMethodFnIdent::Put => self.fixed_medium,
                            VaultMethodFnIdent::Take => self.fixed_medium, // TODO: revisit this if vault is not loaded in full
                            VaultMethodFnIdent::TakeNonFungibles => self.fixed_medium,
                            VaultMethodFnIdent::GetAmount => self.fixed_low,
                            VaultMethodFnIdent::GetResourceAddress => self.fixed_low,
                            VaultMethodFnIdent::GetNonFungibleIds => self.fixed_medium,
                            VaultMethodFnIdent::CreateProof => self.fixed_high,
                            VaultMethodFnIdent::CreateProofByAmount => self.fixed_high,
                            VaultMethodFnIdent::CreateProofByIds => self.fixed_high,
                            VaultMethodFnIdent::LockFee => self.fixed_medium,
                            VaultMethodFnIdent::LockContingentFee => self.fixed_medium,
                        }
                    }
                    MethodFnIdent::Scrypto { .. } => self.fixed_high,
                }
            }
        }
    }

    pub fn system_api_cost(&self, entry: SystemApiCostingEntry) -> u32 {
        match entry {
            SystemApiCostingEntry::Invoke { input, .. } => {
                self.fixed_low + (5 * input.raw.len() + 10 * input.value_count()) as u32
            }

            SystemApiCostingEntry::ReadOwnedNodes => self.fixed_low,
            SystemApiCostingEntry::CreateNode { .. } => self.fixed_medium,
            SystemApiCostingEntry::DropNode { .. } => self.fixed_medium,
            SystemApiCostingEntry::GlobalizeNode { size } => self.fixed_high + 200 * size,
            SystemApiCostingEntry::BorrowNode { loaded, size } => {
                if loaded {
                    self.fixed_high
                } else {
                    self.fixed_low + 100 * size
                }
            }

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
            SystemApiCostingEntry::ReadBlob { size } => self.fixed_low + size,
            SystemApiCostingEntry::GenerateUuid => self.fixed_low,
            SystemApiCostingEntry::EmitLog { size } => self.fixed_low + 10 * size,
        }
    }
}
