use crate::types::*;

pub const FIXED_LOW_FEE: u32 = 500;
pub const FIXED_MEDIUM_FEE: u32 = 2500;
pub const FIXED_HIGH_FEE: u32 = 5000;

const COSTING_COEFFICENT: u64 = 286;
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
                            EntityType::GlobalGenericComponent => 212,
                            EntityType::GlobalValidator => 227,
                            EntityType::InternalKeyValueStore => 232,
                            EntityType::GlobalPackage => 289,
                            EntityType::GlobalAccount => 300,
                            EntityType::GlobalIdentity => 302,
                            EntityType::InternalAccount => 307,
                            EntityType::GlobalAccessController => 312,
                            EntityType::GlobalNonFungibleResource => 312,
                            EntityType::InternalGenericComponent => 312,
                            EntityType::GlobalFungibleResource => 313,
                            EntityType::InternalFungibleVault => 313,
                            EntityType::InternalNonFungibleVault => 320,
                            _ => 288, // average of above values
                        }
                    } else {
                        288 // average of above values
                    }
                } else {
                    // virtual_node
                    121
                }
            }
            CostingEntry::CreateNode { size: _, node_id } => match node_id.entity_type() {
                Some(EntityType::GlobalAccessController) => 2570,
                Some(EntityType::GlobalAccount) => 2769,
                Some(EntityType::GlobalClock) => 1038,
                Some(EntityType::GlobalEpochManager) => 1222,
                Some(EntityType::GlobalFungibleResource) => 1524,
                Some(EntityType::GlobalGenericComponent) => 2994,
                Some(EntityType::GlobalIdentity) => 1579,
                Some(EntityType::GlobalNonFungibleResource) => 2030,
                Some(EntityType::GlobalPackage) => 1498,
                Some(EntityType::GlobalValidator) => 2526,
                Some(EntityType::GlobalVirtualEcdsaAccount) => 2085,
                Some(EntityType::GlobalVirtualEcdsaIdentity) => 1662,
                Some(EntityType::InternalAccount) => 1434,
                Some(EntityType::InternalFungibleVault) => 1128,
                Some(EntityType::InternalGenericComponent) => 1045,
                Some(EntityType::InternalKeyValueStore) => 897,
                Some(EntityType::InternalNonFungibleVault) => 1148,
                _ => 1715, // average of above values
            },
            CostingEntry::CreateWasmInstance { size: _ } => 3731719,
            CostingEntry::DropLock => 128,
            CostingEntry::DropNode { size: _ } => 2590,
            CostingEntry::Invoke {
                input_size,
                identifier,
            } => match identifier {
                InvocationDebugIdentifier::Function(fn_ident) => {
                    match (&*fn_ident.0.blueprint_name, &*fn_ident.1) {
                        ("AccessController", "create_global") => 848269,
                        ("AccessRules", "create") => 75347,
                        ("Account", "create_advanced") => 340578,
                        ("Bucket", "Bucket_drop_empty") => 161038,
                        ("Bucket", "burn_bucket") => 197409,
                        ("Clock", "create") => 108859,
                        ("ComponentRoyalty", "create") => 16440,
                        ("EpochManager", "create") => 439771,
                        ("Faucet", "new") => 7059991,
                        ("FungibleResourceManager", "create") => 362035,
                        ("FungibleResourceManager", "create_with_initial_supply") => 443806,
                        ("FungibleResourceManager", "create_with_initial_supply_and_address") => {
                            410268
                        }
                        ("GenesisHelper", "init") => 5069068,
                        ("Identity", "create") => 787453,
                        ("Identity", "create_advanced") => 325851,
                        ("Metadata", "create") => 25401,
                        ("Metadata", "create_with_data") => 25636,
                        ("NonFungibleResourceManager", "create") => 395739,
                        ("NonFungibleResourceManager", "create_non_fungible_with_address") => {
                            295664
                        }
                        (
                            "NonFungibleResourceManager",
                            "create_uuid_non_fungible_with_initial_supply",
                        ) => 507866,
                        ("NonFungibleResourceManager", "create_with_initial_supply") => 476475,
                        ("Package", "publish_wasm") => 590211,
                        ("Proof", "Proof_drop") => 263505,
                        ("Radiswap", "instantiate_pool") => 11987757,
                        ("TransactionProcessor", "run") => 2231748,
                        ("Worktop", "Worktop_drop") => 93133,
                        ("Package", "publish_native") => 10 * input_size + 6984, // calculated using linear regression on gathered data (11 calls)
                        ("Package", "publish_wasm_advanced") => 24 * input_size + 341025, // calculated using linear regression on gathered data (56 calls)
                        _ => 7554608, // average of above values and function invokes from all tests
                    }
                }
                InvocationDebugIdentifier::Method(method_ident) => {
                    match (method_ident.1, &*method_ident.2) {
                        (SysModuleId::AccessRules, "set_group_access_rule") => 73044,
                        (SysModuleId::AccessRules, "set_group_access_rule_and_mutability") => 88835,
                        (SysModuleId::AccessRules, "set_group_mutability") => 192477,
                        (SysModuleId::AccessRules, "set_method_access_rule") => 74038,
                        (SysModuleId::AccessRules, "set_method_access_rule_and_mutability") => {
                            89686
                        }
                        (SysModuleId::AccessRules, "set_method_mutability") => 192769,
                        (SysModuleId::Metadata, "get") => 36633,
                        (SysModuleId::Metadata, "remove") => 56982,
                        (SysModuleId::Metadata, "set") => 55853,
                        (SysModuleId::ObjectState, "Bucket_create_proof") => 176987,
                        (SysModuleId::ObjectState, "Bucket_get_amount") => 94103,
                        (SysModuleId::ObjectState, "Bucket_get_non_fungible_local_ids") => 98372,
                        (SysModuleId::ObjectState, "Bucket_get_resource_address") => 91789,
                        (SysModuleId::ObjectState, "Bucket_lock_amount") => 97561,
                        (SysModuleId::ObjectState, "Bucket_lock_non_fungibles") => 100607,
                        (SysModuleId::ObjectState, "Bucket_put") => 103731,
                        (SysModuleId::ObjectState, "Bucket_take") => 167911,
                        (SysModuleId::ObjectState, "Bucket_take_non_fungibles") => 170130,
                        (SysModuleId::ObjectState, "Bucket_unlock_amount") => 100146,
                        (SysModuleId::ObjectState, "Bucket_unlock_non_fungibles") => 99776,
                        (SysModuleId::ObjectState, "PackageRoyalty_claim_royalty") => 590914,
                        (SysModuleId::ObjectState, "PackageRoyalty_set_royalty_config") => 288772,
                        (SysModuleId::ObjectState, "Proof_get_amount") => 94911,
                        (SysModuleId::ObjectState, "Proof_get_non_fungible_local_ids") => 95870,
                        (SysModuleId::ObjectState, "Proof_get_resource_address") => 93326,
                        (SysModuleId::ObjectState, "Worktop_drain") => 95248,
                        (SysModuleId::ObjectState, "Worktop_put") => 278530,
                        (SysModuleId::ObjectState, "Worktop_take") => 361064,
                        (SysModuleId::ObjectState, "Worktop_take_all") => 79769,
                        (SysModuleId::ObjectState, "Worktop_take_non_fungibles") => 197588,
                        (SysModuleId::ObjectState, "assert_access_rule") => 304343,
                        (SysModuleId::ObjectState, "burn") => 271539,
                        (SysModuleId::ObjectState, "call_other_component") => 329789,
                        (SysModuleId::ObjectState, "call_other_component_in_child") => 498593,
                        (SysModuleId::ObjectState, "call_other_component_in_parent") => 368573,
                        (SysModuleId::ObjectState, "call_self") => 265879,
                        (SysModuleId::ObjectState, "cancel_recovery_role_recovery_proposal") => {
                            209342
                        }
                        (SysModuleId::ObjectState, "claim_xrd") => 1146282,
                        (SysModuleId::ObjectState, "clear") => 95485,
                        (SysModuleId::ObjectState, "clear_signature_proofs") => 95816,
                        (SysModuleId::ObjectState, "clone") => 267140,
                        (SysModuleId::ObjectState, "compare_current_time") => 43974,
                        (SysModuleId::ObjectState, "compose_vault_and_bucket_proof") => 3578454,
                        (SysModuleId::ObjectState, "compose_vault_and_bucket_proof_by_amount") => {
                            3374937
                        }
                        (SysModuleId::ObjectState, "compose_vault_and_bucket_proof_by_ids") => {
                            3511931
                        }
                        (SysModuleId::ObjectState, "create_bucket") => 252393,
                        (SysModuleId::ObjectState, "create_clone_drop_vault_proof") => 1894056,
                        (SysModuleId::ObjectState, "create_clone_drop_vault_proof_by_amount") => {
                            1890790
                        }
                        (SysModuleId::ObjectState, "create_clone_drop_vault_proof_by_ids") => {
                            2033484
                        }
                        (SysModuleId::ObjectState, "create_proof") => 356638,
                        (SysModuleId::ObjectState, "create_proof_by_amount") => 255904,
                        (SysModuleId::ObjectState, "create_proof_by_ids") => 425561,
                        (SysModuleId::ObjectState, "create_proof_of_all") => 264232,
                        (SysModuleId::ObjectState, "create_proof_of_amount") => 179484,
                        (SysModuleId::ObjectState, "create_proof_of_non_fungibles") => 262279,
                        (SysModuleId::ObjectState, "create_validator") => 2129546,
                        (SysModuleId::ObjectState, "create_vault") => 317835,
                        (SysModuleId::ObjectState, "cross_component_call") => 2168840,
                        (SysModuleId::ObjectState, "deposit") => 789874,
                        (SysModuleId::ObjectState, "deposit_batch") => 701074,
                        (SysModuleId::ObjectState, "drain") => 97519,
                        (SysModuleId::ObjectState, "free") => 687963,
                        (SysModuleId::ObjectState, "func") => 168653,
                        (SysModuleId::ObjectState, "get_address") => 130942,
                        (SysModuleId::ObjectState, "get_address_in_owned") => 308111,
                        (SysModuleId::ObjectState, "get_address_in_parent") => 170204,
                        (SysModuleId::ObjectState, "get_amount") => 105840,
                        (SysModuleId::ObjectState, "get_component_state") => 297399,
                        (SysModuleId::ObjectState, "get_current_epoch") => 69551,
                        (SysModuleId::ObjectState, "get_current_time") => 50100,
                        (SysModuleId::ObjectState, "get_non_fungible") => 201547,
                        (SysModuleId::ObjectState, "get_non_fungible_local_ids") => 105819,
                        (SysModuleId::ObjectState, "get_resource_type") => 180705,
                        (SysModuleId::ObjectState, "get_secret") => 131898,
                        (SysModuleId::ObjectState, "get_total_supply") => 181791,
                        (SysModuleId::ObjectState, "get_value_via_mut_ref") => 262421,
                        (SysModuleId::ObjectState, "get_value_via_ref") => 204559,
                        (SysModuleId::ObjectState, "initiate_recovery_as_primary") => 213013,
                        (SysModuleId::ObjectState, "initiate_recovery_as_recovery") => 260435,
                        (SysModuleId::ObjectState, "lock_contingent_fee") => 263731,
                        (SysModuleId::ObjectState, "lock_fee") => 258832,
                        (SysModuleId::ObjectState, "lock_fee_and_query_vault") => 755522,
                        (SysModuleId::ObjectState, "lock_fee_and_withdraw") => 613645,
                        (SysModuleId::ObjectState, "lock_fee_and_withdraw_non_fungibles") => 613732,
                        (SysModuleId::ObjectState, "lock_fungible_amount") => 106206,
                        (SysModuleId::ObjectState, "lock_non_fungibles") => 181336,
                        (SysModuleId::ObjectState, "lock_primary_role") => 206405,
                        (SysModuleId::ObjectState, "make_move") => 503743,
                        (SysModuleId::ObjectState, "mint") => 427425,
                        (SysModuleId::ObjectState, "mint_single_uuid") => 372288,
                        (SysModuleId::ObjectState, "mint_uuid") => 368770,
                        (SysModuleId::ObjectState, "next_round") => 96784,
                        (SysModuleId::ObjectState, "non_fungible_exists") => 200862,
                        (SysModuleId::ObjectState, "paid_method") => 778202,
                        (SysModuleId::ObjectState, "parent_get_secret") => 331433,
                        (SysModuleId::ObjectState, "parent_set_secret") => 464997,
                        (SysModuleId::ObjectState, "pop") => 94742,
                        (SysModuleId::ObjectState, "protected_method") => 175205,
                        (SysModuleId::ObjectState, "push") => 99947,
                        (SysModuleId::ObjectState, "push_vault_into_vector") => 1608365,
                        (SysModuleId::ObjectState, "put") => 177234,
                        (SysModuleId::ObjectState, "put_auth") => 1051339,
                        (SysModuleId::ObjectState, "put_bucket") => 195154,
                        (SysModuleId::ObjectState, "put_component_state") => 604254,
                        (SysModuleId::ObjectState, "query_vault_and_lock_fee") => 765882,
                        (
                            SysModuleId::ObjectState,
                            "quick_confirm_primary_role_recovery_proposal",
                        ) => 706296,
                        (
                            SysModuleId::ObjectState,
                            "quick_confirm_recovery_role_recovery_proposal",
                        ) => 669649,
                        (SysModuleId::ObjectState, "recall") => 315845,
                        (SysModuleId::ObjectState, "receive_bucket") => 900049,
                        (SysModuleId::ObjectState, "receive_proof") => 376229,
                        (SysModuleId::ObjectState, "recurse") => 6383838,
                        (SysModuleId::ObjectState, "register") => 384275,
                        (SysModuleId::ObjectState, "remove") => 299002,
                        (SysModuleId::ObjectState, "repay_loan") => 2705069,
                        (SysModuleId::ObjectState, "run_tests_with_external_blueprint") => 1466619,
                        (SysModuleId::ObjectState, "run_tests_with_external_component") => 661512,
                        (SysModuleId::ObjectState, "securify") => 1019600,
                        (SysModuleId::ObjectState, "set") => 50235,
                        (SysModuleId::ObjectState, "set_address") => 212475,
                        (SysModuleId::ObjectState, "set_current_time") => 54220,
                        (SysModuleId::ObjectState, "set_depth") => 619490,
                        (SysModuleId::ObjectState, "set_epoch") => 72751,
                        (SysModuleId::ObjectState, "set_group_access_rule_and_mutability") => 70449,
                        (SysModuleId::ObjectState, "set_method_access_rule_and_mutability") => {
                            85754
                        }
                        (SysModuleId::ObjectState, "set_secret") => 185570,
                        (SysModuleId::ObjectState, "stake") => 1460481,
                        (SysModuleId::ObjectState, "stop_timed_recovery") => 307002,
                        (SysModuleId::ObjectState, "swap") => 3235710,
                        (SysModuleId::ObjectState, "take") => 253438,
                        (SysModuleId::ObjectState, "take_loan") => 2431355,
                        (SysModuleId::ObjectState, "take_non_fungibles") => 260253,
                        (SysModuleId::ObjectState, "test_lock_contingent_fee") => 501780,
                        (SysModuleId::ObjectState, "timed_confirm_recovery") => 721139,
                        (SysModuleId::ObjectState, "total_supply") => 352164,
                        (SysModuleId::ObjectState, "unlock_fungible_amount") => 172016,
                        (SysModuleId::ObjectState, "unlock_non_fungibles") => 181180,
                        (SysModuleId::ObjectState, "unlock_primary_role") => 207860,
                        (SysModuleId::ObjectState, "unregister") => 343990,
                        (SysModuleId::ObjectState, "unstake") => 1825609,
                        (SysModuleId::ObjectState, "update_accept_delegated_stake") => 356084,
                        (SysModuleId::ObjectState, "update_auth") => 369565,
                        (SysModuleId::ObjectState, "update_key") => 437408,
                        (SysModuleId::ObjectState, "update_non_fungible_data") => 305445,
                        (SysModuleId::ObjectState, "update_validator") => 64421,
                        (SysModuleId::ObjectState, "use_vault_proof_for_auth") => 1750892,
                        (SysModuleId::ObjectState, "withdraw") => 334215,
                        (SysModuleId::ObjectState, "withdraw_non_fungibles") => 353223,
                        (SysModuleId::Royalty, "claim_royalty") => 439501,
                        (SysModuleId::Royalty, "set_royalty_config") => 229948,
                        _ => 549662, // average of above values
                    }
                }
                InvocationDebugIdentifier::VirtualLazyLoad => 327891, // average from 2 calls
            },
            CostingEntry::LockSubstate {
                node_id,
                module_id,
                substate_key: _,
            } => match (module_id, node_id.entity_type()) {
                (SysModuleId::AccessRules, Some(EntityType::GlobalAccessController)) => 3711,
                (SysModuleId::AccessRules, Some(EntityType::GlobalAccount)) => 2394,
                (SysModuleId::AccessRules, Some(EntityType::GlobalClock)) => 2113,
                (SysModuleId::AccessRules, Some(EntityType::GlobalEpochManager)) => 2476,
                (SysModuleId::AccessRules, Some(EntityType::GlobalFungibleResource)) => 1094,
                (SysModuleId::AccessRules, Some(EntityType::GlobalGenericComponent)) => 1764,
                (SysModuleId::AccessRules, Some(EntityType::GlobalIdentity)) => 2071,
                (SysModuleId::AccessRules, Some(EntityType::GlobalNonFungibleResource)) => 1058,
                (SysModuleId::AccessRules, Some(EntityType::GlobalPackage)) => 2800,
                (SysModuleId::AccessRules, Some(EntityType::GlobalValidator)) => 2453,
                (SysModuleId::AccessRules, Some(EntityType::GlobalVirtualEcdsaAccount)) => 1159,
                (SysModuleId::AccessRules, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 1110,
                (SysModuleId::Metadata, Some(EntityType::GlobalAccount)) => 1697,
                (SysModuleId::Metadata, Some(EntityType::GlobalFungibleResource)) => 975,
                (SysModuleId::Metadata, Some(EntityType::GlobalGenericComponent)) => 1512,
                (SysModuleId::Metadata, Some(EntityType::GlobalIdentity)) => 1605,
                (SysModuleId::Metadata, Some(EntityType::GlobalPackage)) => 1513,
                (SysModuleId::Metadata, Some(EntityType::GlobalValidator)) => 1598,
                (SysModuleId::Metadata, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 1562,
                (SysModuleId::ObjectState, Some(EntityType::GlobalAccessController)) => 2690,
                (SysModuleId::ObjectState, Some(EntityType::GlobalAccount)) => 1429,
                (SysModuleId::ObjectState, Some(EntityType::GlobalClock)) => 1370,
                (SysModuleId::ObjectState, Some(EntityType::GlobalEpochManager)) => 1644,
                (SysModuleId::ObjectState, Some(EntityType::GlobalFungibleResource)) => 1343,
                (SysModuleId::ObjectState, Some(EntityType::GlobalGenericComponent)) => 1487,
                (SysModuleId::ObjectState, Some(EntityType::GlobalNonFungibleResource)) => 1406,
                (SysModuleId::ObjectState, Some(EntityType::GlobalPackage)) => 1323,
                (SysModuleId::ObjectState, Some(EntityType::GlobalValidator)) => 2705,
                (SysModuleId::ObjectState, Some(EntityType::GlobalVirtualEcdsaAccount)) => 1406,
                (SysModuleId::ObjectState, Some(EntityType::InternalFungibleVault)) => 1636,
                (SysModuleId::ObjectState, Some(EntityType::InternalGenericComponent)) => 1014,
                (SysModuleId::ObjectState, Some(EntityType::InternalKeyValueStore)) => 3494,
                (SysModuleId::ObjectState, Some(EntityType::InternalNonFungibleVault)) => 1358,
                (SysModuleId::Royalty, Some(EntityType::GlobalAccessController)) => 1373,
                (SysModuleId::Royalty, Some(EntityType::GlobalAccount)) => 1354,
                (SysModuleId::Royalty, Some(EntityType::GlobalClock)) => 1380,
                (SysModuleId::Royalty, Some(EntityType::GlobalEpochManager)) => 1385,
                (SysModuleId::Royalty, Some(EntityType::GlobalGenericComponent)) => 1349,
                (SysModuleId::Royalty, Some(EntityType::GlobalIdentity)) => 1357,
                (SysModuleId::Royalty, Some(EntityType::GlobalValidator)) => 1366,
                (SysModuleId::Royalty, Some(EntityType::GlobalVirtualEcdsaAccount)) => 1337,
                (SysModuleId::Royalty, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 1047,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalAccessController)) => 1075,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalAccount)) => 1070,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalClock)) => 1100,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalEpochManager)) => 1120,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalFungibleResource)) => 1073,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalGenericComponent)) => 1078,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalIdentity)) => 1068,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalNonFungibleResource)) => 1076,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalPackage)) => 1075,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalValidator)) => 1082,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalVirtualEcdsaAccount)) => 1318,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 1322,
                (SysModuleId::TypeInfo, Some(EntityType::InternalAccount)) => 932,
                (SysModuleId::TypeInfo, Some(EntityType::InternalFungibleVault)) => 1102,
                (SysModuleId::TypeInfo, Some(EntityType::InternalGenericComponent)) => 916,
                (SysModuleId::TypeInfo, Some(EntityType::InternalKeyValueStore)) => 1041,
                (SysModuleId::TypeInfo, Some(EntityType::InternalNonFungibleVault)) => 963,
                _ => 1514,
            },
            CostingEntry::ReadSubstate { size: _ } => 239,
            CostingEntry::WriteSubstate { size: _ } => 221,
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
