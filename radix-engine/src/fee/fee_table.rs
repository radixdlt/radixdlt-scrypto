use crate::types::*;
use scrypto::core::{
    FnIdent, MethodIdent, NativeFunction, ReceiverMethodIdent, ResourceManagerFunction,
    SystemFunction,
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
            wasm_instantiation_per_byte: 0, // TODO: Re-enable WASM instantiation cost if it's unavoidable
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
                    FunctionIdent::Native(NativeFunction::TransactionProcessor(
                        transaction_processor_fn,
                    )) => match transaction_processor_fn {
                        TransactionProcessorFunction::Run => self.fixed_high,
                    },
                    FunctionIdent::Native(NativeFunction::Package(package_fn)) => {
                        match package_fn {
                            PackageFunction::Publish => self.fixed_low + input.raw.len() as u32 * 2,
                        }
                    }
                    FunctionIdent::Native(NativeFunction::System(system_ident)) => {
                        match system_ident {
                            SystemFunction::Create => self.fixed_low,
                        }
                    }
                    FunctionIdent::Native(NativeFunction::ResourceManager(
                        resource_manager_ident,
                    )) => {
                        match resource_manager_ident {
                            ResourceManagerFunction::Create => self.fixed_high, // TODO: more investigation about fungibility
                        }
                    }
                    FunctionIdent::Scrypto { .. } => 0, // Costing is through instrumentation // TODO: Josh question, why only through instrumentation?
                }
            }
            FnIdent::Method(ReceiverMethodIdent { method_ident, .. }) => {
                match method_ident {
                    MethodIdent::Native(NativeMethod::AuthZone(auth_zone_ident)) => {
                        match auth_zone_ident {
                            AuthZoneMethod::Pop => self.fixed_low,
                            AuthZoneMethod::Push => self.fixed_low,
                            AuthZoneMethod::CreateProof => self.fixed_high, // TODO: charge differently based on auth zone size and fungibility
                            AuthZoneMethod::CreateProofByAmount => self.fixed_high,
                            AuthZoneMethod::CreateProofByIds => self.fixed_high,
                            AuthZoneMethod::Clear => self.fixed_high,
                            AuthZoneMethod::Drain => self.fixed_high,
                        }
                    }
                    MethodIdent::Native(NativeMethod::System(system_ident)) => match system_ident {
                        SystemMethod::GetCurrentEpoch => self.fixed_low,
                        SystemMethod::GetTransactionHash => self.fixed_low,
                        SystemMethod::SetEpoch => self.fixed_low,
                    },
                    MethodIdent::Native(NativeMethod::Bucket(bucket_ident)) => match bucket_ident {
                        BucketMethod::Take => self.fixed_medium,
                        BucketMethod::TakeNonFungibles => self.fixed_medium,
                        BucketMethod::GetNonFungibleIds => self.fixed_medium,
                        BucketMethod::Put => self.fixed_medium,
                        BucketMethod::GetAmount => self.fixed_low,
                        BucketMethod::GetResourceAddress => self.fixed_low,
                        BucketMethod::CreateProof => self.fixed_low,
                        BucketMethod::Burn => self.fixed_medium,
                    },
                    MethodIdent::Native(NativeMethod::Proof(proof_ident)) => match proof_ident {
                        ProofMethod::GetAmount => self.fixed_low,
                        ProofMethod::GetNonFungibleIds => self.fixed_low,
                        ProofMethod::GetResourceAddress => self.fixed_low,
                        ProofMethod::Clone => self.fixed_low,
                        ProofMethod::Drop => self.fixed_medium,
                    },
                    MethodIdent::Native(NativeMethod::ResourceManager(resource_manager_ident)) => {
                        match resource_manager_ident {
                            ResourceManagerMethod::UpdateAuth => self.fixed_medium,
                            ResourceManagerMethod::LockAuth => self.fixed_medium,
                            ResourceManagerMethod::CreateVault => self.fixed_medium,
                            ResourceManagerMethod::CreateBucket => self.fixed_medium,
                            ResourceManagerMethod::Mint => self.fixed_high,
                            ResourceManagerMethod::GetMetadata => self.fixed_low,
                            ResourceManagerMethod::GetResourceType => self.fixed_low,
                            ResourceManagerMethod::GetTotalSupply => self.fixed_low,
                            ResourceManagerMethod::UpdateMetadata => self.fixed_medium,
                            ResourceManagerMethod::UpdateNonFungibleData => self.fixed_medium,
                            ResourceManagerMethod::NonFungibleExists => self.fixed_low,
                            ResourceManagerMethod::GetNonFungible => self.fixed_medium,
                        }
                    }
                    MethodIdent::Native(NativeMethod::Worktop(worktop_ident)) => {
                        match worktop_ident {
                            WorktopMethod::Put => self.fixed_medium,
                            WorktopMethod::TakeAmount => self.fixed_medium,
                            WorktopMethod::TakeAll => self.fixed_medium,
                            WorktopMethod::TakeNonFungibles => self.fixed_medium,
                            WorktopMethod::AssertContains => self.fixed_low,
                            WorktopMethod::AssertContainsAmount => self.fixed_low,
                            WorktopMethod::AssertContainsNonFungibles => self.fixed_low,
                            WorktopMethod::Drain => self.fixed_low,
                        }
                    }
                    MethodIdent::Native(NativeMethod::Component(component_ident)) => {
                        match component_ident {
                            ComponentMethod::AddAccessCheck => self.fixed_medium,
                        }
                    }
                    MethodIdent::Native(NativeMethod::Vault(vault_ident)) => {
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
                            VaultMethod::LockContingentFee => self.fixed_medium,
                        }
                    }
                    MethodIdent::Scrypto { .. } => self.fixed_high,
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

            SystemApiCostingEntry::LockSubstate { .. } => self.fixed_low,
            SystemApiCostingEntry::TakeSubstate { .. } => self.fixed_medium,
            SystemApiCostingEntry::ReadSubstate { .. } => self.fixed_medium,
            SystemApiCostingEntry::WriteSubstate { .. } => self.fixed_medium,
            SystemApiCostingEntry::DropLock => self.fixed_low,

            SystemApiCostingEntry::ReadEpoch => self.fixed_low,
            SystemApiCostingEntry::ReadTransactionHash => self.fixed_low,
            SystemApiCostingEntry::ReadBlob { size } => self.fixed_low + size,
            SystemApiCostingEntry::GenerateUuid => self.fixed_low,
            SystemApiCostingEntry::EmitLog { size } => self.fixed_low + 10 * size,
        }
    }
}
