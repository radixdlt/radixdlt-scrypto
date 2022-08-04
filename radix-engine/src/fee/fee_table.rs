use scrypto::engine::types::*;
use scrypto::prelude::TypeName;
use scrypto::{core::Receiver, values::ScryptoValue};

use crate::wasm::{InstructionCostRules, WasmMeteringParams};

pub enum SystemApiCostingEntry<'a> {
    /// Invokes a function, native or wasm.
    InvokeFunction {
        type_name: TypeName,
        input: &'a ScryptoValue,
    },

    /// Invokes a method, native or wasm.
    InvokeMethod {
        receiver: Receiver,
        input: &'a ScryptoValue,
    },

    /// Globalizes a RE value.
    Globalize { size: u32 },

    /// Borrows a globalized value.
    BorrowGlobal { loaded: bool, size: u32 },

    /// Borrows a local value.
    BorrowLocal,

    /// Returns a borrowed value.
    ReturnGlobal { size: u32 },

    /// Returns a borrowed value.
    ReturnLocal,

    /// Creates a RE value.
    Create { size: u32 },

    /// Reads the data of a RE value.
    Read { size: u32 },

    /// Updates the data of a RE Value.
    Write { size: u32 },

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
    CheckAccessRule,
}

pub struct FeeTable {
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
            tx_decoding_per_byte: 4,
            tx_manifest_verification_per_byte: 1,
            tx_signature_verification_per_sig: 3750,
            wasm_instantiation_per_byte: 500,
            fixed_low: 1000,
            fixed_medium: 5_000,
            fixed_high: 10_000,
            wasm_metering_params: WasmMeteringParams::new(InstructionCostRules::tiered(50000), 512),
        }
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

    pub fn run_function_cost(
        &self,
        type_name: &TypeName,
        fn_ident: &str,
        input: &ScryptoValue,
    ) -> u32 {
        match type_name {
            TypeName::Package => match fn_ident {
                "publish" => self.fixed_low + input.raw.len() as u32 * 2,
                _ => self.fixed_high,
            },
            TypeName::ResourceManager => match fn_ident {
                "create" => self.fixed_high, // TODO: more investigation about fungibility
                _ => self.fixed_high,
            },
            TypeName::TransactionProcessor => match fn_ident {
                "run" => self.fixed_high, // TODO: per manifest instruction
                _ => self.fixed_high,
            },
            TypeName::Blueprint(_, _) => {
                0 // Costing is through instrumentation
            }
        }
    }

    pub fn run_method_cost(
        &self,
        receiver: &Receiver,
        fn_ident: &str,
        _input: &ScryptoValue,
    ) -> u32 {
        match receiver {
            Receiver::SystemRef => match fn_ident {
                "current_epoch" => self.fixed_low,
                "transaction_hash" => self.fixed_low,
                _ => self.fixed_high,
            },
            Receiver::AuthZoneRef => match fn_ident {
                "pop" => self.fixed_low,
                "push" => self.fixed_low,
                "create_proof" => self.fixed_high, // TODO: charge differently based on auth zone size and fungibility
                "create_proof_by_amount" => self.fixed_high,
                "create_proof_by_ids" => self.fixed_high,
                "clear" => self.fixed_high,
                _ => self.fixed_high,
            },
            Receiver::Component(_) => {
                0 // Costing is through instrumentation
            }
            Receiver::ComponentMetaRef(_) => match fn_ident {
                "add_access_check" => self.fixed_medium,
                _ => self.fixed_high,
            },
            // TODO: I suspect there is a bug with invoking consumed within call frame. Add tests to verify
            Receiver::Consumed(node_id) => match node_id {
                RENodeId::Bucket(_) => self.fixed_medium,
                RENodeId::Proof(_) => self.fixed_medium,
                RENodeId::Worktop => self.fixed_medium,
                RENodeId::KeyValueStore(_) => self.fixed_medium,
                RENodeId::Component(_) => self.fixed_medium,
                RENodeId::Vault(_) => self.fixed_medium,
                RENodeId::ResourceManager(_) => self.fixed_medium,
                RENodeId::Package(_) => self.fixed_high,
                RENodeId::System => self.fixed_high,
            },
            Receiver::NativeRENodeRef(node_id) => {
                match node_id {
                    RENodeId::Bucket(..) => {
                        match fn_ident {
                            "take" => self.fixed_medium,
                            "take_non_fungibles" => self.fixed_medium,
                            "non_fungible_ids" => self.fixed_medium,
                            "put" => self.fixed_medium,
                            "amount" => self.fixed_low,
                            "resource_address" => self.fixed_low,
                            "create_proof" => self.fixed_low,
                            _ => self.fixed_high,
                        }
                    }
                    RENodeId::Proof(..) => {
                        match fn_ident {
                            "amount" => self.fixed_low,
                            "non_fungible_ids" => self.fixed_low,
                            "resource_address" => self.fixed_low,
                            "clone" => self.fixed_high,
                            _ => self.fixed_high,
                        }
                    }
                    RENodeId::ResourceManager(..) => {
                        match fn_ident {
                            "update_auth" => self.fixed_medium,
                            "lock_auth" => self.fixed_medium,
                            "create_vault" => self.fixed_medium,
                            "create_bucket" => self.fixed_medium,
                            "mint" => self.fixed_high,
                            "metadata" => self.fixed_low,
                            "resource_type" => self.fixed_low,
                            "total_supply" => self.fixed_low, // TODO: revisit this after substate refactoring
                            "update_metadata" => self.fixed_medium,
                            "update_non_fungible_data" => self.fixed_medium,
                            "non_fungible_exists" => self.fixed_low,
                            "non_fungible_data" => self.fixed_medium,
                            _ => self.fixed_high,
                        }
                    }
                    _ => self.fixed_high // TODO: Clean this up
                }
            },
            Receiver::VaultRef(_) => match fn_ident {
                "put" => self.fixed_medium,
                "take" => self.fixed_medium, // TODO: revisit this if vault is not loaded in full
                "take_non_fungibles" => self.fixed_medium,
                "amount" => self.fixed_low,
                "resource_address" => self.fixed_low,
                "non_fungible_ids" => self.fixed_medium,
                "create_proof" => self.fixed_high, // TODO: fungibility
                "create_proof_by_amount" => self.fixed_high,
                "create_proof_by_ids" => self.fixed_high,
                "lock_fee" => self.fixed_medium,
                _ => self.fixed_high,
            },
            Receiver::WorktopRef => match fn_ident {
                "put" => self.fixed_medium,
                "take_amount" => self.fixed_medium,
                "take_all" => self.fixed_medium,
                "take_non_fungibles" => self.fixed_medium,
                "assert_contains" => self.fixed_low,
                "assert_contains_amount" => self.fixed_low,
                "assert_contains_non_fungibles" => self.fixed_low,
                "drain" => self.fixed_medium,
                _ => self.fixed_high,
            },
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
            SystemApiCostingEntry::Globalize { size } => self.fixed_high + 200 * size,
            SystemApiCostingEntry::BorrowGlobal { loaded, size } => {
                if loaded {
                    self.fixed_high
                } else {
                    self.fixed_low + 100 * size
                }
            }
            SystemApiCostingEntry::BorrowLocal => self.fixed_medium,
            SystemApiCostingEntry::ReturnGlobal { size } => self.fixed_low + 100 * size,
            SystemApiCostingEntry::ReturnLocal => self.fixed_medium,
            SystemApiCostingEntry::Create { .. } => self.fixed_high,
            SystemApiCostingEntry::Read { .. } => self.fixed_medium,
            SystemApiCostingEntry::Write { .. } => self.fixed_medium,
            SystemApiCostingEntry::ReadEpoch => self.fixed_low,
            SystemApiCostingEntry::ReadTransactionHash => self.fixed_low,
            SystemApiCostingEntry::ReadTransactionNetwork => self.fixed_low,
            SystemApiCostingEntry::GenerateUuid => self.fixed_low,
            SystemApiCostingEntry::EmitLog { size } => self.fixed_low + 10 * size,
            SystemApiCostingEntry::CheckAccessRule => self.fixed_medium,
        }
    }
}
