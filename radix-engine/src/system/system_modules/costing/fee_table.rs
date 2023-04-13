use crate::types::*;
use crate::system::system_callback::SystemInvocation;

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
            CostingEntry::CreateWasmInstance { size } => 24 * size + 11081, // calculated using linear regression on gathered data
            CostingEntry::DropLock => 136,
            CostingEntry::DropNode { size: _ } => 2520, // average of gathered data
            CostingEntry::Invoke {
                input_size,
                identifier,
            } => match identifier {
                InvocationDebugIdentifier::Function(fn_ident) => {
                    match (&*fn_ident.0.blueprint_name, &*fn_ident.1) {
                        ("AccessController", "create_global") => 795969,
                        ("AccessRules", "create") => 73544,
                        ("Account", "create_advanced") => 325058,
                        ("Bucket", "Bucket_drop_empty") => 149485,
                        ("Bucket", "burn_bucket") => 181894,
                        ("Clock", "create") => 104242,
                        ("ComponentRoyalty", "create") => 16055,
                        ("EpochManager", "create") => 417490,
                        ("Faucet", "new") => 6884668,
                        ("FungibleResourceManager", "create") => 343343,
                        ("FungibleResourceManager", "create_with_initial_supply") => 414210,
                        ("FungibleResourceManager", "create_with_initial_supply_and_address") => {
                            387105
                        }
                        ("GenesisHelper", "init") => 4860566,
                        ("Identity", "create") => 740089,
                        ("Identity", "create_advanced") => 311984,
                        ("Metadata", "create") => 24237,
                        ("Metadata", "create_with_data") => 24372,
                        ("NonFungibleResourceManager", "create") => 285038,
                        ("NonFungibleResourceManager", "create_non_fungible_with_address") => {
                            279965
                        }
                        ("NonFungibleResourceManager", "create_with_initial_supply") => 452078,
                        ("Package", "publish_wasm") => 556374,
                        ("Proof", "Proof_drop") => 246568,
                        ("TransactionProcessor", "run") => 2036313,
                        ("Worktop", "Worktop_drop") => 84794,
                        ("Package", "publish_native") => 10 * input_size + 7365, // calculated using linear regression on gathered data
                        ("Package", "publish_wasm_advanced") => 29 * input_size + 119758, // calculated using linear regression on gathered data
                        _ => 883074, // average of above values and function invokes from all tests
                    }
                }
                InvocationDebugIdentifier::Method(method_ident) => {
                    match (method_ident.1, &*method_ident.2) {
                        (SysModuleId::AccessRules, "set_group_access_rule") => 68705,
                        (SysModuleId::AccessRules, "set_group_access_rule_and_mutability") => 83670,
                        (SysModuleId::AccessRules, "set_method_access_rule") => 69485,
                        (SysModuleId::AccessRules, "set_method_access_rule_and_mutability") => {
                            86304
                        }
                        (SysModuleId::Metadata, "set") => 233518,
                        (SysModuleId::ObjectState, "Bucket_create_proof") => 163783,
                        (SysModuleId::ObjectState, "Bucket_get_amount") => 88240,
                        (SysModuleId::ObjectState, "Bucket_get_non_fungible_local_ids") => 93142,
                        (SysModuleId::ObjectState, "Bucket_get_resource_address") => 85511,
                        (SysModuleId::ObjectState, "Bucket_put") => 97078,
                        (SysModuleId::ObjectState, "Bucket_take") => 156423,
                        (SysModuleId::ObjectState, "Bucket_unlock_amount") => 93832,
                        (SysModuleId::ObjectState, "Bucket_unlock_non_fungibles") => 93187,
                        (SysModuleId::ObjectState, "Proof_get_amount") => 88912,
                        (SysModuleId::ObjectState, "Proof_get_non_fungible_local_ids") => 90432,
                        (SysModuleId::ObjectState, "Proof_get_resource_address") => 85378,
                        (SysModuleId::ObjectState, "Worktop_drain") => 87413,
                        (SysModuleId::ObjectState, "Worktop_put") => 261330,
                        (SysModuleId::ObjectState, "Worktop_take") => 336622,
                        (SysModuleId::ObjectState, "Worktop_take_all") => 74492,
                        (SysModuleId::ObjectState, "Worktop_take_non_fungibles") => 262550,
                        (SysModuleId::ObjectState, "burn") => 264553,
                        (SysModuleId::ObjectState, "cancel_recovery_role_recovery_proposal") => {
                            198184
                        }
                        (SysModuleId::ObjectState, "claim_xrd") => 1075226,
                        (SysModuleId::ObjectState, "clear") => 90214,
                        (SysModuleId::ObjectState, "clear_signature_proofs") => 89839,
                        (SysModuleId::ObjectState, "compare_current_time") => 48901,
                        (SysModuleId::ObjectState, "create_bucket") => 232587,
                        (SysModuleId::ObjectState, "create_proof") => 327414,
                        (SysModuleId::ObjectState, "create_proof_by_amount") => 315575,
                        (SysModuleId::ObjectState, "create_proof_of_all") => 238251,
                        (SysModuleId::ObjectState, "create_proof_of_amount") => 240935,
                        (SysModuleId::ObjectState, "create_validator") => 2001663,
                        (SysModuleId::ObjectState, "create_vault") => 287299,
                        (SysModuleId::ObjectState, "deposit") => 739866,
                        (SysModuleId::ObjectState, "deposit_batch") => 638888,
                        (SysModuleId::ObjectState, "free") => 737524,
                        (SysModuleId::ObjectState, "get_amount") => 101070,
                        (SysModuleId::ObjectState, "get_current_epoch") => 66552,
                        (SysModuleId::ObjectState, "get_current_time") => 48720,
                        (SysModuleId::ObjectState, "get_non_fungible") => 190066,
                        (SysModuleId::ObjectState, "get_resource_type") => 168642,
                        (SysModuleId::ObjectState, "get_total_supply") => 169726,
                        (SysModuleId::ObjectState, "initiate_recovery_as_primary") => 201066,
                        (SysModuleId::ObjectState, "initiate_recovery_as_recovery") => 247943,
                        (SysModuleId::ObjectState, "lock_fee") => 245977,
                        (SysModuleId::ObjectState, "lock_fee_and_withdraw") => 580162,
                        (SysModuleId::ObjectState, "lock_fee_and_withdraw_non_fungibles") => 579870,
                        (SysModuleId::ObjectState, "lock_fungible_amount") => 99990,
                        (SysModuleId::ObjectState, "lock_primary_role") => 195280,
                        (SysModuleId::ObjectState, "mint") => 383918,
                        (SysModuleId::ObjectState, "mint_single_uuid") => 346602,
                        (SysModuleId::ObjectState, "pop") => 89468,
                        (SysModuleId::ObjectState, "push") => 91801,
                        (SysModuleId::ObjectState, "put") => 163825,
                        (
                            SysModuleId::ObjectState,
                            "quick_confirm_primary_role_recovery_proposal",
                        ) => 670369,
                        (
                            SysModuleId::ObjectState,
                            "quick_confirm_recovery_role_recovery_proposal",
                        ) => 632787,
                        (SysModuleId::ObjectState, "recall") => 300435,
                        (SysModuleId::ObjectState, "securify") => 961255,
                        (SysModuleId::ObjectState, "set_current_time") => 52727,
                        (SysModuleId::ObjectState, "set_epoch") => 70550,
                        (SysModuleId::ObjectState, "set_group_access_rule_and_mutability") => 84093,
                        (SysModuleId::ObjectState, "set_method_access_rule_and_mutability") => {
                            80665
                        }
                        (SysModuleId::ObjectState, "stop_timed_recovery") => 290086,
                        (SysModuleId::ObjectState, "take") => 229293,
                        (SysModuleId::ObjectState, "take_non_fungibles") => 241335,
                        (SysModuleId::ObjectState, "timed_confirm_recovery") => 685974,
                        (SysModuleId::ObjectState, "unlock_fungible_amount") => 160982,
                        (SysModuleId::ObjectState, "unlock_non_fungibles") => 170017,
                        (SysModuleId::ObjectState, "unlock_primary_role") => 196773,
                        (SysModuleId::ObjectState, "unstake") => 1716311,
                        (SysModuleId::ObjectState, "update_validator") => 61761,
                        (SysModuleId::ObjectState, "withdraw") => 312330,
                        _ => 289251, // average of above values
                    }
                }
                InvocationDebugIdentifier::VirtualLazyLoad => 334636, // average from 2 calls
            },
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
                (SysModuleId::ObjectState, Some(EntityType::GlobalAccessController)) => 2648,
                (SysModuleId::ObjectState, Some(EntityType::GlobalAccount)) => 1419,
                (SysModuleId::ObjectState, Some(EntityType::GlobalClock)) => 1552,
                (SysModuleId::ObjectState, Some(EntityType::GlobalEpochManager)) => 1602,
                (SysModuleId::ObjectState, Some(EntityType::GlobalFungibleResource)) => 1329,
                (SysModuleId::ObjectState, Some(EntityType::GlobalGenericComponent)) => 1463,
                (SysModuleId::ObjectState, Some(EntityType::GlobalNonFungibleResource)) => 1393,
                (SysModuleId::ObjectState, Some(EntityType::GlobalPackage)) => 1319,
                (SysModuleId::ObjectState, Some(EntityType::GlobalValidator)) => 1657,
                (SysModuleId::ObjectState, Some(EntityType::GlobalVirtualEcdsaAccount)) => 1404,
                (SysModuleId::ObjectState, Some(EntityType::InternalFungibleVault)) => 1621,
                (SysModuleId::ObjectState, Some(EntityType::InternalGenericComponent)) => 999,
                (SysModuleId::ObjectState, Some(EntityType::InternalKeyValueStore)) => 3698,
                (SysModuleId::ObjectState, Some(EntityType::InternalNonFungibleVault)) => 1342,
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
