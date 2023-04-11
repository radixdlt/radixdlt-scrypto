use crate::types::*;

pub const FIXED_LOW_FEE: u32 = 500;
pub const FIXED_MEDIUM_FEE: u32 = 2500;
pub const FIXED_HIGH_FEE: u32 = 5000;

const COSTING_COEFFICENT: u64 = 237;
const COSTING_COEFFICENT_DIV_BITS: u64 = 9; // used to divide by shift left operator, original value: 4

pub enum CostingEntry<'a> {
    /* invoke */
    Invoke {
        input_size: u32,
        identifier: &'a InvocationDebugIdentifier,
    },
    CreateWasmInstance {
        size: u32,
    },

    /* node */
    CreateNode {
        size: u32,
        node_id: &'a NodeId,
    },
    DropNode {
        size: u32,
    },
    AllocateNodeId {
        entity_type: &'a EntityType,
    },

    /* substate */
    LockSubstate {
        node_id: &'a NodeId,
        module_id: &'a SysModuleId,
        substate_key: &'a SubstateKey,
    },
    ReadSubstate {
        size: u32,
    },
    WriteSubstate {
        size: u32,
    },
    GetSubstateRef,
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

    /// CPU instructions usage numbers obtained from test runs with 'resource_tracker` feature enabled
    /// and transformed (classified and groupped) using convert.py script.
    fn kernel_api_cost_from_cpu_usage(&self, entry: &CostingEntry) -> u32 {
        ((match entry {
            CostingEntry::AllocateNodeId { entity_type } => match entity_type {
                EntityType::GlobalAccount => 111,
                EntityType::GlobalGenericComponent => 714,
                EntityType::GlobalFungibleResource => 1093,
                EntityType::GlobalPackage => 1197,
                EntityType::InternalKeyValueStore => 12,
                _ => 0,
            },
            CostingEntry::CreateNode { size: _, node_id: _ } => 0,
            CostingEntry::CreateWasmInstance { size } => {
                size / 26 // approx. by average from 10 calls (2 groups)
            }
            CostingEntry::DropLock => 180,
            CostingEntry::DropNode { size: _ } => 4191,
            CostingEntry::GetSubstateRef => 169,
            CostingEntry::Invoke {
                input_size,
                identifier,
            } => match identifier {
                InvocationDebugIdentifier::Function(fn_ident) => {
                    match (&*fn_ident.0.blueprint_name, &*fn_ident.1) {
                        ("AccessRules", "create") => 29249,
                        ("Account", "create_advanced") => 184577,
                        ("Clock", "create") => 66722,
                        ("ComponentRoyalty", "create") => 9963,
                        ("EpochManager", "create") => 213624,
                        ("Faucet", "new") => 5708511,
                        ("FungibleResourceManager", "create_with_initial_supply") => 246986,
                        ("FungibleResourceManager", "create_with_initial_supply_and_address") => {
                            192718
                        }
                        ("Metadata", "create") => 17906,
                        ("Metadata", "create_with_data") => 11172,
                        ("MoveTest", "move_bucket") => 7845424,
                        ("MoveTest", "move_proof") => 1091059,
                        ("NonFungibleResourceManager", "create_non_fungible_with_address") => {
                            136818
                        }
                        ("Package", "publish_native") => 5 * input_size + 9975, // calculated using linear regression on gathered data (11 calls)
                        ("Package", "publish_wasm_advanced") => 12 * input_size + 3117837, // calculated using straight line equetion (basing on 2 calls only)
                        ("Proof", "Proof_drop") => 65827,
                        ("TransactionProcessor", "run") => 9040534,
                        ("Worktop", "Worktop_drop") => 24143,
                        _ => 0,
                    }
                }
                InvocationDebugIdentifier::Method(method_ident) => 0,
                InvocationDebugIdentifier::VirtualLazyLoad => 0,
            },
            CostingEntry::LockSubstate {
                node_id,
                module_id,
                substate_key,
            } => 0,
            CostingEntry::ReadSubstate { size: _ } => 552,
            CostingEntry::WriteSubstate { size: _ } => 176,
        }) as u64
            * COSTING_COEFFICENT
            >> COSTING_COEFFICENT_DIV_BITS) as u32
    }

    fn kernel_api_cost_from_memory_usage(&self, entry: &CostingEntry) -> u32 {
        match entry {
            CostingEntry::CreateNode { size, node_id: _ } => 100 * size,
            CostingEntry::DropNode { size } => 100 * size,
            CostingEntry::Invoke {
                input_size,
                identifier: _,
            } => 10 * input_size,
            CostingEntry::ReadSubstate { size } => 10 * size,
            CostingEntry::WriteSubstate { size } => 1000 * size,
            _ => 0,
        }
    }

    pub fn kernel_api_cost(&self, entry: CostingEntry) -> u32 {
        self.kernel_api_cost_from_cpu_usage(&entry) + self.kernel_api_cost_from_memory_usage(&entry)
    }
}
