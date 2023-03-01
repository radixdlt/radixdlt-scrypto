use crate::types::*;

pub const FIXED_LOW_FEE: u32 = 500;
pub const FIXED_MEDIUM_FEE: u32 = 2500;
pub const FIXED_HIGH_FEE: u32 = 5000;

pub enum CostingEntry {
    /* invoke */
    Invoke { input_size: u32 },

    /* node */
    CreateNode { size: u32 },
    DropNode { size: u32 },

    /* substate */
    LockSubstate,
    ReadSubstate { size: u32 },
    WriteSubstate { size: u32 },
    DropLock,
    // TODO: more costing after API becomes stable.
}

#[derive(Debug, Clone, ScryptoSbor)]
pub struct FeeTable {
    tx_base_fee: u32,
    tx_payload_cost_per_byte: u32,
    tx_signature_verification_per_sig: u32,
    tx_blob_price_per_byte: u32,
}

impl FeeTable {
    pub fn new() -> Self {
        Self {
            tx_base_fee: 50_000,
            tx_payload_cost_per_byte: 5,
            tx_signature_verification_per_sig: 100_000,
            tx_blob_price_per_byte: 5,
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

    pub fn kernel_api_cost(&self, entry: CostingEntry) -> u32 {
        match entry {
            CostingEntry::Invoke { input_size } => FIXED_LOW_FEE + (10 * input_size) as u32,

            CostingEntry::CreateNode { size } => FIXED_MEDIUM_FEE + (100 * size) as u32,
            CostingEntry::DropNode { size } => FIXED_MEDIUM_FEE + (100 * size) as u32,

            CostingEntry::LockSubstate => FIXED_LOW_FEE,
            CostingEntry::ReadSubstate { size } => FIXED_LOW_FEE + 10 * size,
            CostingEntry::WriteSubstate { size } => FIXED_LOW_FEE + 1000 * size,
            CostingEntry::DropLock => FIXED_LOW_FEE,
        }
    }
}
