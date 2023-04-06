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
        node_id: &'a RENodeId,
    },
    DropNode {
        size: u32,
    },
    AllocateNodeId {
        node_type: &'a AllocateEntityType,
    },

    /* substate */
    LockSubstate {
        node_id: &'a RENodeId,
        module_id: &'a NodeModuleId,
        offset: &'a SubstateOffset,
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
            CostingEntry::AllocateNodeId { node_type } => match node_type {
                AllocateEntityType::GlobalAccount => 111,
                AllocateEntityType::GlobalComponent => 714,
                AllocateEntityType::GlobalFungibleResourceManager => 1093,
                AllocateEntityType::GlobalPackage => 1197,
                AllocateEntityType::KeyValueStore => 12,
                AllocateEntityType::Object => 11,
                AllocateEntityType::Vault => 21,
                _ => 0,
            },
            CostingEntry::CreateNode { size: _, node_id } => match node_id {
                RENodeId::KeyValueStore(_) => 493,
                RENodeId::Object(_) => 3290,
                RENodeId::GlobalObject(address) => match address {
                    Address::Component(component) => match component {
                        ComponentAddress::Account(_) => 10050,
                        ComponentAddress::Clock(_) => 5049,
                        ComponentAddress::EpochManager(_) => 7471,
                        ComponentAddress::Normal(_) => 6111,
                        _ => 0,
                    },
                    Address::Resource(resource_type) => match resource_type {
                        ResourceAddress::Fungible(_) => 32715,
                        ResourceAddress::NonFungible(_) => 17973,
                    },
                    Address::Package(package_type) => match package_type {
                        PackageAddress::Normal(_) => 4964,
                    },
                },
            },
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
                    match (&*fn_ident.1, &*fn_ident.2) {
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
                InvocationDebugIdentifier::Method(method_ident) => match method_ident.1 {
                    NodeModuleId::SELF => match &*method_ident.2 {
                        "Bucket_create_proof" => 81014,
                        "Bucket_get_amount" => 27029,
                        "Bucket_get_resource_address" => 27497,
                        "Bucket_unlock_amount" => 32359,
                        "Worktop_drain" => 33199,
                        "Worktop_put" => 76813,
                        "Worktop_take_all" => 10797,
                        "create_vault" => 35931,
                        "deposit_batch" => 175766,
                        "free" => 546446,
                        "get_current_epoch" => 69146,
                        "lock_fee" => 157017,
                        "put" => 42799,
                        "receive_bucket" => 368023,
                        "receive_proof" => 200679,
                        "set_method_access_rule_and_mutability" => 37408,
                        "take" => 90009,
                        _ => 0,
                    },
                    _ => 0,
                },
                InvocationDebugIdentifier::VirtualLazyLoad => 0,
            },
            CostingEntry::LockSubstate {
                node_id,
                module_id,
                offset,
            } => match node_id {
                RENodeId::GlobalObject(address) => match address {
                    Address::Component(component) => match component {
                        ComponentAddress::Account(_) => match module_id {
                            NodeModuleId::AccessRules => 3277,
                            NodeModuleId::ComponentRoyalty => 743,
                            NodeModuleId::SELF => 1244,
                            NodeModuleId::TypeInfo => 612,
                            _ => 0,
                        },
                        ComponentAddress::EpochManager(_) => match module_id {
                            NodeModuleId::AccessRules => 5447,
                            NodeModuleId::ComponentRoyalty => 806,
                            NodeModuleId::SELF => 2827,
                            NodeModuleId::TypeInfo => 1011,
                            _ => 0,
                        },
                        ComponentAddress::Normal(_) => match module_id {
                            NodeModuleId::AccessRules => 1110,
                            NodeModuleId::ComponentRoyalty => 684,
                            NodeModuleId::SELF => 902,
                            NodeModuleId::TypeInfo => 667,
                            _ => 0,
                        },
                        _ => 0,
                    },
                    Address::Resource(resource_type) => match resource_type {
                        ResourceAddress::Fungible(_) => match module_id {
                            NodeModuleId::AccessRules1 => 3376,
                            NodeModuleId::AccessRules => 2685,
                            NodeModuleId::SELF => 555,
                            NodeModuleId::TypeInfo => 620,
                            _ => 0,
                        },
                        ResourceAddress::NonFungible(_) => 0,
                    },
                    Address::Package(package_type) => match package_type {
                        PackageAddress::Normal(_) => {
                            if matches!(module_id, NodeModuleId::SELF) {
                                match offset {
                                    SubstateOffset::Package(package_offset) => match package_offset
                                    {
                                        PackageOffset::Code => 269,
                                        PackageOffset::CodeType => 519,
                                        PackageOffset::FunctionAccessRules => 647,
                                        PackageOffset::Info => 374,
                                        PackageOffset::Royalty => 567,
                                    },
                                    _ => 0,
                                }
                            } else {
                                0
                            }
                        }
                    },
                },
                RENodeId::KeyValueStore(_) => match module_id {
                    NodeModuleId::SELF => 1566,
                    NodeModuleId::TypeInfo => 3271,
                    _ => 0,
                },
                RENodeId::Object(_) => match module_id {
                    NodeModuleId::SELF => match offset {
                        SubstateOffset::AccessRules(_) => 2932,
                        SubstateOffset::Bucket(_) => (833 + 751 + 747) / 3, // average of all Bucket matches
                        SubstateOffset::KeyValueStore(_) => 543,
                        SubstateOffset::Proof(_) => 995,
                        SubstateOffset::Vault(_) => (924 + 917) / 2, // average of all Vault matches
                        SubstateOffset::Worktop(_) => 843,
                        _ => 0,
                    },
                    NodeModuleId::TypeInfo => 802,
                    _ => 0,
                },
            },
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
