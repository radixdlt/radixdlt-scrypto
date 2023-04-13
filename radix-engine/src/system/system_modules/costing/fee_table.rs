use crate::system::system_callback::SystemInvocation;
use crate::types::*;

pub const FIXED_LOW_FEE: u32 = 500;
pub const FIXED_MEDIUM_FEE: u32 = 2500;
pub const FIXED_HIGH_FEE: u32 = 5000;

const COSTING_COEFFICENT: u64 = 320;
const COSTING_COEFFICENT_DIV_BITS: u64 = 4; // used to divide by shift left operator
const COSTING_COEFFICENT_DIV_BITS_ADDON: u64 = 5; // used to scale up or down all cpu instruction costing

pub enum CostingEntry<'a> {
    /* invoke */
    Invoke {
        input_size: u32,
        identifier: &'a SystemInvocation,
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
        entity_type: Option<EntityType>,
        virtual_node: bool,
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
    DropLock,
    ReadBucket,
    ReadProof,
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
            CostingEntry::AllocateNodeId {
                entity_type,
                virtual_node,
            } => {
                if !virtual_node {
                    if entity_type.is_some() {
                        match entity_type.unwrap() {
                            EntityType::GlobalAccessController => 300,
                            EntityType::GlobalAccount => 290,
                            EntityType::GlobalFungibleResource => 307,
                            EntityType::GlobalGenericComponent => 208,
                            EntityType::GlobalIdentity => 290,
                            EntityType::GlobalNonFungibleResource => 300,
                            EntityType::GlobalPackage => 291,
                            EntityType::GlobalValidator => 220,
                            EntityType::InternalAccount => 294,
                            EntityType::InternalFungibleVault => 301,
                            EntityType::InternalGenericComponent => 301,
                            EntityType::InternalKeyValueStore => 222,
                            EntityType::InternalNonFungibleVault => 308,
                            _ => 279, // average of above values
                        }
                    } else {
                        279 // average of above values
                    }
                } else {
                    // virtual_node
                    193
                }
            }
            CostingEntry::CreateNode { size: _, node_id } => match node_id.entity_type() {
                Some(EntityType::GlobalAccessController) => 2696,
                Some(EntityType::GlobalAccount) => 2804,
                Some(EntityType::GlobalClock) => 1128,
                Some(EntityType::GlobalEpochManager) => 1348,
                Some(EntityType::GlobalFungibleResource) => 1855,
                Some(EntityType::GlobalGenericComponent) => 3114,
                Some(EntityType::GlobalIdentity) => 1627,
                Some(EntityType::GlobalNonFungibleResource) => 2103,
                Some(EntityType::GlobalPackage) => 1661,
                Some(EntityType::GlobalValidator) => 2762,
                Some(EntityType::GlobalVirtualEcdsaAccount) => 2426,
                Some(EntityType::GlobalVirtualEcdsaIdentity) => 1708,
                Some(EntityType::InternalAccount) => 1381,
                Some(EntityType::InternalFungibleVault) => 1108,
                Some(EntityType::InternalGenericComponent) => 1025,
                Some(EntityType::InternalKeyValueStore) => 881,
                Some(EntityType::InternalNonFungibleVault) => 1113,
                _ => 1808, // average of above values
            },
            CostingEntry::DropLock => 136,
            CostingEntry::DropNode { size: _ } => 2520, // average of gathered data
            CostingEntry::Invoke {
                input_size: _,
                identifier: _,
            } => 0, // todo
            CostingEntry::LockSubstate {
                node_id,
                module_id,
                substate_key: _,
            } => match (module_id, node_id.entity_type()) {
                (SysModuleId::AccessRules, Some(EntityType::GlobalAccessController)) => 3716,
                (SysModuleId::AccessRules, Some(EntityType::GlobalAccount)) => 2387,
                (SysModuleId::AccessRules, Some(EntityType::GlobalClock)) => 2104,
                (SysModuleId::AccessRules, Some(EntityType::GlobalEpochManager)) => 2489,
                (SysModuleId::AccessRules, Some(EntityType::GlobalFungibleResource)) => 1070,
                (SysModuleId::AccessRules, Some(EntityType::GlobalGenericComponent)) => 1740,
                (SysModuleId::AccessRules, Some(EntityType::GlobalIdentity)) => 2054,
                (SysModuleId::AccessRules, Some(EntityType::GlobalNonFungibleResource)) => 1051,
                (SysModuleId::AccessRules, Some(EntityType::GlobalPackage)) => 1806,
                (SysModuleId::AccessRules, Some(EntityType::GlobalValidator)) => 2432,
                (SysModuleId::AccessRules, Some(EntityType::GlobalVirtualEcdsaAccount)) => 1075,
                (SysModuleId::AccessRules, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 1084,
                (SysModuleId::Metadata, Some(EntityType::GlobalFungibleResource)) => 564,
                (SysModuleId::Metadata, Some(EntityType::GlobalIdentity)) => 1586,
                (SysModuleId::Metadata, Some(EntityType::GlobalPackage)) => 1506,
                (SysModuleId::Metadata, Some(EntityType::GlobalValidator)) => 1575,
                (SysModuleId::Metadata, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 1549,
                (SysModuleId::Royalty, Some(EntityType::GlobalAccessController)) => 1364,
                (SysModuleId::Royalty, Some(EntityType::GlobalAccount)) => 1342,
                (SysModuleId::Royalty, Some(EntityType::GlobalClock)) => 1380,
                (SysModuleId::Royalty, Some(EntityType::GlobalEpochManager)) => 1375,
                (SysModuleId::Royalty, Some(EntityType::GlobalGenericComponent)) => 1341,
                (SysModuleId::Royalty, Some(EntityType::GlobalIdentity)) => 1340,
                (SysModuleId::Royalty, Some(EntityType::GlobalValidator)) => 1351,
                (SysModuleId::Royalty, Some(EntityType::GlobalVirtualEcdsaAccount)) => 1047,
                (SysModuleId::Royalty, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 1032,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalAccessController)) => 1060,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalAccount)) => 1058,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalClock)) => 1096,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalEpochManager)) => 1099,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalFungibleResource)) => 1063,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalGenericComponent)) => 1064,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalIdentity)) => 1055,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalNonFungibleResource)) => 1063,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalValidator)) => 1062,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalVirtualEcdsaAccount)) => 1450,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 1301,
                (SysModuleId::TypeInfo, Some(EntityType::InternalAccount)) => 910,
                (SysModuleId::TypeInfo, Some(EntityType::InternalFungibleVault)) => 1084,
                (SysModuleId::TypeInfo, Some(EntityType::InternalGenericComponent)) => 907,
                (SysModuleId::TypeInfo, Some(EntityType::InternalKeyValueStore)) => 1035,
                (SysModuleId::TypeInfo, Some(EntityType::InternalNonFungibleVault)) => 946,
                _ => 1465, // average of above values
            },
            CostingEntry::ReadBucket => 191,
            CostingEntry::ReadProof => 243,
            CostingEntry::ReadSubstate { size: _ } => 222,
            CostingEntry::WriteSubstate { size: _ } => 218,
        }) as u64
            * COSTING_COEFFICENT
            >> (COSTING_COEFFICENT_DIV_BITS + COSTING_COEFFICENT_DIV_BITS_ADDON)) as u32
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
