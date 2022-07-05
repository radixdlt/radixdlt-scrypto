use scrypto::{core::SNodeRef, values::ScryptoValue};

use crate::{
    engine::REValue,
    wasm::{InstructionCostRules, WasmMeteringParams},
};

pub enum SystemApiCostingEntry<'a> {
    /// Invokes a function, native or wasm.
    InvokeFunction {
        receiver: &'a SNodeRef,
        input: &'a ScryptoValue,
    },

    /// Globalizes a RE value.
    Globalize { size: u32 },

    /// Borrows a globalized value.
    Borrow {
        global: bool,
        loaded: bool,
        size: u32,
    },

    /// Returns a borrowed value.
    Return { global: bool, size: u32 },

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

    /// Generates a UUID.
    GenerateUuid,

    /// Emits a log.
    EmitLog { size: u32 },

    /// Checks if an access rule can be satisfied by the given proofs.
    CheckAccessRule,
}

pub struct FeeTable {
    tx_decoding_per_byte: u32,
    tx_verification_per_byte: u32,
    tx_signature_validation_per_sig: u32,
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
            tx_verification_per_byte: 1,
            tx_signature_validation_per_sig: 3750,
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

    pub fn tx_verification_per_byte(&self) -> u32 {
        self.tx_verification_per_byte
    }

    pub fn tx_signature_validation_per_sig(&self) -> u32 {
        self.tx_signature_validation_per_sig
    }

    pub fn wasm_instantiation_per_byte(&self) -> u32 {
        self.wasm_instantiation_per_byte
    }

    pub fn wasm_metering_params(&self) -> WasmMeteringParams {
        self.wasm_metering_params.clone()
    }

    pub fn function_cost(&self, receiver: &SNodeRef, fn_ident: &str, input: &ScryptoValue) -> u32 {
        match receiver {
            SNodeRef::SystemStatic => todo!(),
            SNodeRef::PackageStatic => todo!(),
            SNodeRef::AuthZoneRef => todo!(),
            SNodeRef::Scrypto(_) => 0,
            SNodeRef::Component(_) => todo!(),
            SNodeRef::ResourceStatic => todo!(),
            SNodeRef::ResourceRef(_) => todo!(),
            SNodeRef::Consumed(_) => todo!(),
            SNodeRef::BucketRef(_) => todo!(),
            SNodeRef::ProofRef(_) => todo!(),
            SNodeRef::VaultRef(_) => todo!(),
            SNodeRef::TransactionProcessor => todo!(),
        }
    }

    pub fn system_api_cost(&self, entry: SystemApiCostingEntry) -> u32 {
        match entry {
            SystemApiCostingEntry::InvokeFunction { receiver, input } => todo!(),
            SystemApiCostingEntry::Globalize { size } => todo!(),
            SystemApiCostingEntry::Borrow {
                global,
                loaded,
                size,
            } => todo!(),
            SystemApiCostingEntry::Return { global, size } => todo!(),
            SystemApiCostingEntry::Create { size } => todo!(),
            SystemApiCostingEntry::Read { size } => todo!(),
            SystemApiCostingEntry::Write { size } => todo!(),
            SystemApiCostingEntry::ReadEpoch => todo!(),
            SystemApiCostingEntry::ReadTransactionHash => todo!(),
            SystemApiCostingEntry::GenerateUuid => todo!(),
            SystemApiCostingEntry::EmitLog { size } => todo!(),
            SystemApiCostingEntry::CheckAccessRule => todo!(),
        }
    }
}
