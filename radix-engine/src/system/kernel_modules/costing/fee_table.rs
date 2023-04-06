use crate::types::*;

pub const FIXED_LOW_FEE: u32 = 500;
pub const FIXED_MEDIUM_FEE: u32 = 2500;
pub const FIXED_HIGH_FEE: u32 = 5000;

const COSTING_COEFFICENT: u32 = 237;
const COSTING_COEFFICENT_DIV_BITS: u32 = 4; // used to divide by shift left operator


pub enum CostingEntry<'a> {
    /* invoke */
    Invoke { input_size: u32 },

    /* node */
    CreateNode { node_id: &'a RENodeId },
    DropNode { size: u32 },
    AllocateNodeId { node_type: &'a AllocateEntityType },

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

    fn kernel_api_cost_from_cpu_usage(&self, entry: &CostingEntry) -> u32 {
        match entry {
            CostingEntry::Invoke { input_size } => FIXED_LOW_FEE + (10 * input_size) as u32,

            CostingEntry::LockSubstate => FIXED_LOW_FEE,
            CostingEntry::ReadSubstate { size } => FIXED_LOW_FEE + 10 * size,
            CostingEntry::WriteSubstate { size } => FIXED_LOW_FEE + 1000 * size,
            
            // new implementation

            CostingEntry::CreateNode { node_id } => (match node_id {
                    RENodeId::KeyValueStore(_) => 493,
                    RENodeId::Object(_) => 3290,
                    RENodeId::GlobalObject(address) => match address {
                        Address::Component(component) => match component {
                            ComponentAddress::AccessController(_) => 0,
                            ComponentAddress::Account(_) => 10050,
                            ComponentAddress::Clock(_) => 5049,
                            ComponentAddress::EpochManager(_) => 7471,
                            ComponentAddress::Identity(_) => 0,
                            ComponentAddress::Normal(_) => 6111,
                            ComponentAddress::Validator(_) => 0,
                            _ => 0,
                        },
                        Address::Resource(resource_type) => match resource_type {
                            ResourceAddress::Fungible(_) => 32715,
                            ResourceAddress::NonFungible(_) => 17973,
                        }
                        Address::Package(package_type) => match package_type {
                            PackageAddress::Normal(_) => 4964,
                        }                        
                    }
                }) * COSTING_COEFFICENT >> COSTING_COEFFICENT_DIV_BITS,
            CostingEntry::DropNode { size: _ } => 4191 * COSTING_COEFFICENT >> COSTING_COEFFICENT_DIV_BITS,
            CostingEntry::DropLock => 180 * COSTING_COEFFICENT >> COSTING_COEFFICENT_DIV_BITS,
            CostingEntry::AllocateNodeId { node_type } => (match node_type {
                AllocateEntityType::GlobalAccount => 111,
                AllocateEntityType::GlobalComponent => 714,
                AllocateEntityType::GlobalFungibleResourceManager => 1093,
                AllocateEntityType::GlobalNonFungibleResourceManager => 0,
                AllocateEntityType::GlobalPackage => 1197,
                AllocateEntityType::GlobalEpochManager => 0,
                AllocateEntityType::GlobalValidator => 0,
                AllocateEntityType::GlobalAccessController => 0,
                AllocateEntityType::GlobalIdentity => 0,
                AllocateEntityType::KeyValueStore => 12,
                AllocateEntityType::Object => 11,
                AllocateEntityType::Vault => 21,
            }) * COSTING_COEFFICENT >> COSTING_COEFFICENT_DIV_BITS
        }
    }

    fn kernel_api_cost_from_memory_usage(&self, _entry: &CostingEntry) -> u32 {
        // todo
        0
    }

    pub fn kernel_api_cost(&self, entry: CostingEntry) -> u32 {
        self.kernel_api_cost_from_cpu_usage(&entry) + self.kernel_api_cost_from_memory_usage(&entry)
    }
}
