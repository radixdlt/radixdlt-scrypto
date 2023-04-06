use crate::types::*;

pub const FIXED_LOW_FEE: u32 = 500;
pub const FIXED_MEDIUM_FEE: u32 = 2500;
pub const FIXED_HIGH_FEE: u32 = 5000;

const COSTING_COEFFICENT: u32 = 237;
const COSTING_COEFFICENT_DIV_BITS: u32 = 4; // used to divide by shift left operator


pub enum CostingEntry<'a> {
    /* invoke */
    Invoke { input_size: u32, identifier: &'a InvocationDebugIdentifier },

    /* node */
    CreateNode { size: u32, node_id: &'a RENodeId },
    DropNode { size: u32 },
    AllocateNodeId { node_type: &'a AllocateEntityType },

    /* substate */
    LockSubstate {node_id: &'a RENodeId, module_id: &'a NodeModuleId, offset: &'a SubstateOffset} ,
    ReadSubstate { size: u32 },
    WriteSubstate { size: u32 },
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
    /// and data transformed using convert.py script.
    fn kernel_api_cost_from_cpu_usage(&self, entry: &CostingEntry) -> u32 {
        (match entry {
            CostingEntry::AllocateNodeId { node_type } => match node_type {
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
            },
            CostingEntry::CreateNode { size: _, node_id } => match node_id {
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
                },
            CostingEntry::DropLock => 180,
            CostingEntry::DropNode { size: _ } => 4191,
            CostingEntry::GetSubstateRef => 169,
            CostingEntry::Invoke { input_size: _, identifier } => match identifier {
                InvocationDebugIdentifier::Function(fn_ident) => {
                    if fn_ident.1 == "AccessRules" && fn_ident.2 == "create" { 29249 }
                    else if fn_ident.1 == "Account" && fn_ident.2 == "create_advanced" { 184577 }
                    else {
                        0
                    }
                },
                InvocationDebugIdentifier::Method(method_ident) => match method_ident.1 {
                    NodeModuleId::SELF => 0,
                    NodeModuleId::AccessRules => 0,
                    NodeModuleId::AccessRules1 => 0,
                    NodeModuleId::ComponentRoyalty => 0,
                    NodeModuleId::Metadata => 0,
                    NodeModuleId::TypeInfo => 0
                },
                InvocationDebugIdentifier::VirtualLazyLoad => 0
            },
            CostingEntry::LockSubstate { node_id, module_id, offset } => match node_id {
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
                        }
                        ComponentAddress::Normal(_) => match module_id {
                            NodeModuleId::AccessRules => 1110,
                            NodeModuleId::ComponentRoyalty => 684,
                            NodeModuleId::SELF => 902,
                            NodeModuleId::TypeInfo => 667,
                            _ => 0,
                        }
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
                        PackageAddress::Normal(_) => if matches!(module_id, NodeModuleId::SELF) {
                            match offset {
                                SubstateOffset::Package(package_offset) => match package_offset {
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
                    },
                }
                RENodeId::KeyValueStore(_) => match module_id {
                    NodeModuleId::SELF => 1566,
                    NodeModuleId::TypeInfo => 3271,
                    _ => 0
                },
                RENodeId::Object(_) => match module_id {
                    NodeModuleId::SELF => match offset {
                        SubstateOffset::AccessRules(_) => 2932,
                        SubstateOffset::Bucket(_) => ( 833 + 751 + 747 ) / 3, // average of all Bucket matches
                        SubstateOffset::KeyValueStore(_) => 543,
                        SubstateOffset::Proof(_) => 995,
                        SubstateOffset::Vault(_) => ( 924 + 917 ) / 2, // average of all Vault matches
                        SubstateOffset::Worktop(_) => 843,
                        _ => 0,
                    },
                    NodeModuleId::TypeInfo => 802,
                    _ => 0
                },
                
            },
            CostingEntry::ReadSubstate { size: _ } => 552,
            CostingEntry::WriteSubstate { size: _ } => 176,
        }) * COSTING_COEFFICENT >> COSTING_COEFFICENT_DIV_BITS
    }

    fn kernel_api_cost_from_memory_usage(&self, entry: &CostingEntry) -> u32 {
        match entry {
            CostingEntry::CreateNode { size, node_id: _ } => FIXED_MEDIUM_FEE + (100 * size) as u32,
            CostingEntry::DropNode { size } => FIXED_MEDIUM_FEE + (100 * size) as u32,
            CostingEntry::Invoke { input_size, identifier: _ } => FIXED_LOW_FEE + (10 * input_size) as u32,
            CostingEntry::ReadSubstate { size } => FIXED_LOW_FEE + 10 * size,
            CostingEntry::WriteSubstate { size } => FIXED_LOW_FEE + 1000 * size,
            _ => 0
        }
    }

    pub fn kernel_api_cost(&self, entry: CostingEntry) -> u32 {
        self.kernel_api_cost_from_cpu_usage(&entry) + self.kernel_api_cost_from_memory_usage(&entry)
    }
}
