use crate::system::system_callback::SystemInvocation;
use crate::types::*;

pub const FIXED_LOW_FEE: u32 = 500;
pub const FIXED_MEDIUM_FEE: u32 = 2500;
pub const FIXED_HIGH_FEE: u32 = 5000;

const COSTING_COEFFICENT: u64 = 335;
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
                            EntityType::GlobalAccessController => 287,
                            EntityType::GlobalAccount => 278,
                            EntityType::GlobalFungibleResource => 287,
                            EntityType::GlobalGenericComponent => 198,
                            EntityType::GlobalIdentity => 276,
                            EntityType::GlobalNonFungibleResource => 283,
                            EntityType::GlobalPackage => 264,
                            EntityType::GlobalValidator => 212,
                            EntityType::InternalAccount => 278,
                            EntityType::InternalFungibleVault => 284,
                            EntityType::InternalGenericComponent => 283,
                            EntityType::InternalKeyValueStore => 210,
                            EntityType::InternalNonFungibleVault => 292,
                            _ => 264, // average of above values
                        }
                    } else {
                        264 // average of above values
                    }
                } else {
                    // virtual_node
                    106
                }
            }
            CostingEntry::CreateNode { size: _, node_id } => match node_id.entity_type() {
                Some(EntityType::GlobalAccessController) => 2411,
                Some(EntityType::GlobalAccount) => 2264,
                Some(EntityType::GlobalClock) => 1017,
                Some(EntityType::GlobalEpochManager) => 1247,
                Some(EntityType::GlobalFungibleResource) => 1646,
                Some(EntityType::GlobalGenericComponent) => 2399,
                Some(EntityType::GlobalIdentity) => 1477,
                Some(EntityType::GlobalNonFungibleResource) => 1614,
                Some(EntityType::GlobalPackage) => 1513,
                Some(EntityType::GlobalValidator) => 2392,
                Some(EntityType::GlobalVirtualEcdsaAccount) => 1613,
                Some(EntityType::GlobalVirtualEcdsaIdentity) => 1559,
                Some(EntityType::InternalAccount) => 959,
                Some(EntityType::InternalFungibleVault) => 1018,
                Some(EntityType::InternalGenericComponent) => 961,
                Some(EntityType::InternalKeyValueStore) => 851,
                Some(EntityType::InternalNonFungibleVault) => 1035,
                _ => 1528, // average of above values
            },
            CostingEntry::DropLock => 128,
            CostingEntry::DropNode { size: _ } => 918, // average of gathered data
            CostingEntry::Invoke {
                input_size,
                identifier,
            } => match &identifier.ident {
                FnIdent::Application(fn_name) => match (
                    identifier.blueprint.blueprint_name.as_str(),
                    fn_name.as_str(),
                ) {
                    ("AccessController", "cancel_recovery_role_recovery_proposal") => 181490,
                    ("AccessController", "create_global") => 741192,
                    ("AccessController", "create_proof") => 400413,
                    ("AccessController", "initiate_recovery_as_primary") => 186689,
                    ("AccessController", "initiate_recovery_as_recovery") => 229008,
                    ("AccessController", "lock_primary_role") => 178610,
                    ("AccessController", "quick_confirm_primary_role_recovery_proposal") => 634316,
                    ("AccessController", "quick_confirm_recovery_role_recovery_proposal") => 605304,
                    ("AccessController", "stop_timed_recovery") => 274951,
                    ("AccessController", "timed_confirm_recovery") => 647382,
                    ("AccessController", "unlock_primary_role") => 179869,
                    ("AccessRules", "create") => 62276,
                    ("AccessRules", "set_group_access_rule") => 70865,
                    ("AccessRules", "set_group_access_rule_and_mutability") => 58438,
                    ("AccessRules", "set_group_mutability") => 170731,
                    ("AccessRules", "set_method_access_rule") => 66465,
                    ("AccessRules", "set_method_access_rule_and_mutability") => 72561,
                    ("AccessRules", "set_method_mutability") => 171252,
                    ("Account", "create_advanced") => 290590,
                    ("Account", "create_proof") => 313982,
                    ("Account", "create_proof_by_amount") => 225838,
                    ("Account", "create_proof_by_ids") => 308795,
                    ("Account", "deposit") => 450463,
                    ("Account", "deposit_batch") => 615123,
                    ("Account", "lock_contingent_fee") => 231209,
                    ("Account", "lock_fee") => 306680,
                    ("Account", "lock_fee_and_withdraw") => 547559,
                    ("Account", "lock_fee_and_withdraw_non_fungibles") => 551091,
                    ("Account", "securify") => 572609,
                    ("Account", "withdraw") => 294450,
                    ("Account", "withdraw_non_fungibles") => 313789,
                    ("AuthZone", "clear") => 81024,
                    ("AuthZone", "clear_signature_proofs") => 80909,
                    ("AuthZone", "create_proof") => 353820,
                    ("AuthZone", "create_proof_by_amount") => 463745,
                    ("AuthZone", "create_proof_by_ids") => 496086,
                    ("AuthZone", "drain") => 82448,
                    ("AuthZone", "pop") => 79789,
                    ("AuthZone", "push") => 84632,
                    ("Bucket", "Bucket_create_proof") => 156585,
                    ("Bucket", "Bucket_drop_empty") => 138087,
                    ("Bucket", "Bucket_get_amount") => 83434,
                    ("Bucket", "Bucket_get_non_fungible_local_ids") => 87887,
                    ("Bucket", "Bucket_get_resource_address") => 78804,
                    ("Bucket", "Bucket_lock_amount") => 82724,
                    ("Bucket", "Bucket_lock_non_fungibles") => 85513,
                    ("Bucket", "Bucket_put") => 88276,
                    ("Bucket", "Bucket_take") => 147031,
                    ("Bucket", "Bucket_take_non_fungibles") => 148596,
                    ("Bucket", "Bucket_unlock_amount") => 85025,
                    ("Bucket", "Bucket_unlock_non_fungibles") => 84923,
                    ("Bucket", "burn_bucket") => 171185,
                    ("Clock", "compare_current_time") => 37259,
                    ("Clock", "create") => 82812,
                    ("Clock", "get_current_time") => 42077,
                    ("Clock", "set_current_time") => 44019,
                    ("ComponentRoyalty", "claim_royalty") => 398431,
                    ("ComponentRoyalty", "create") => 11237,
                    ("ComponentRoyalty", "set_royalty_config") => 223395,
                    ("EpochManager", "create") => 362720,
                    ("EpochManager", "create_validator") => 1904854,
                    ("EpochManager", "get_current_epoch") => 59444,
                    ("EpochManager", "next_round") => 81885,
                    ("EpochManager", "set_epoch") => 61125,
                    ("EpochManager", "update_validator") => 53754,
                    ("Faucet", "free") => 695784,
                    ("Faucet", "lock_fee") => 494464,
                    ("Faucet", "new") => 6535166,
                    ("FungibleResourceManager", "burn") => 231555,
                    ("FungibleResourceManager", "create") => 340665,
                    ("FungibleResourceManager", "create_bucket") => 220605,
                    ("FungibleResourceManager", "create_vault") => 271664,
                    ("FungibleResourceManager", "create_with_initial_supply") => 405197,
                    ("FungibleResourceManager", "create_with_initial_supply_and_address") => 352256,
                    ("FungibleResourceManager", "get_resource_type") => 157744,
                    ("FungibleResourceManager", "get_total_supply") => 158632,
                    ("FungibleResourceManager", "mint") => 358265,
                    ("FungibleVault", "create_proof_of_all") => 226060,
                    ("FungibleVault", "create_proof_of_amount") => 156156,
                    ("FungibleVault", "get_amount") => 92261,
                    ("FungibleVault", "lock_fee") => 223623,
                    ("FungibleVault", "lock_fungible_amount") => 90156,
                    ("FungibleVault", "put") => 150406,
                    ("FungibleVault", "recall") => 283445,
                    ("FungibleVault", "take") => 221763,
                    ("FungibleVault", "unlock_fungible_amount") => 148047,
                    ("GenesisHelper", "init") => 4569094,
                    ("Identity", "create") => 673880,
                    ("Identity", "create_advanced") => 271947,
                    ("Identity", "securify") => 538544,
                    ("Metadata", "create") => 18519,
                    ("Metadata", "create_with_data") => 18392,
                    ("Metadata", "get") => 31823,
                    ("Metadata", "remove") => 49766,
                    ("Metadata", "set") => 49489,
                    ("NonFungibleResourceManager", "burn") => 261326,
                    ("NonFungibleResourceManager", "create") => 368652,
                    ("NonFungibleResourceManager", "create_bucket") => 238222,
                    ("NonFungibleResourceManager", "create_non_fungible_with_address") => 248922,
                    (
                        "NonFungibleResourceManager",
                        "create_uuid_non_fungible_with_initial_supply",
                    ) => 470621,
                    ("NonFungibleResourceManager", "create_vault") => 306876,
                    ("NonFungibleResourceManager", "create_with_initial_supply") => 439458,
                    ("NonFungibleResourceManager", "get_non_fungible") => 176643,
                    ("NonFungibleResourceManager", "get_resource_type") => 171302,
                    ("NonFungibleResourceManager", "get_total_supply") => 174823,
                    ("NonFungibleResourceManager", "mint") => 404238,
                    ("NonFungibleResourceManager", "mint_single_uuid") => 325318,
                    ("NonFungibleResourceManager", "mint_uuid") => 324056,
                    ("NonFungibleResourceManager", "non_fungible_exists") => 174706,
                    ("NonFungibleResourceManager", "update_non_fungible_data") => 269194,
                    ("NonFungibleVault", "create_proof_of_all") => 236221,
                    ("NonFungibleVault", "create_proof_of_amount") => 226422,
                    ("NonFungibleVault", "create_proof_of_non_fungibles") => 229425,
                    ("NonFungibleVault", "get_amount") => 92070,
                    ("NonFungibleVault", "get_non_fungible_local_ids") => 93095,
                    ("NonFungibleVault", "lock_non_fungibles") => 156268,
                    ("NonFungibleVault", "put") => 158008,
                    ("NonFungibleVault", "recall") => 267720,
                    ("NonFungibleVault", "take") => 228152,
                    ("NonFungibleVault", "take_non_fungibles") => 228294,
                    ("NonFungibleVault", "unlock_non_fungibles") => 156532,
                    ("Package", "PackageRoyalty_claim_royalty") => 542170,
                    ("Package", "PackageRoyalty_set_royalty_config") => 256237,
                    ("Package", "publish_wasm") => 521324,
                    ("Proof", "Proof_drop") => 225030,
                    ("Proof", "Proof_get_amount") => 82525,
                    ("Proof", "Proof_get_non_fungible_local_ids") => 84021,
                    ("Proof", "Proof_get_resource_address") => 79495,
                    ("Proof", "clone") => 232409,
                    ("Radiswap", "instantiate_pool") => 11222983,
                    ("Radiswap", "swap") => 3381443,
                    ("TransactionProcessor", "run") => 2040533,
                    ("Validator", "claim_xrd") => 1019930,
                    ("Validator", "register") => 341788,
                    ("Validator", "stake") => 1292740,
                    ("Validator", "unregister") => 299565,
                    ("Validator", "unstake") => 1620979,
                    ("Validator", "update_accept_delegated_stake") => 313021,
                    ("Validator", "update_key") => 386889,
                    ("Worktop", "Worktop_drain") => 80356,
                    ("Worktop", "Worktop_drop") => 76247,
                    ("Worktop", "Worktop_put") => 244598,
                    ("Worktop", "Worktop_take") => 320777,
                    ("Worktop", "Worktop_take_all") => 68469,
                    ("Worktop", "Worktop_take_non_fungibles") => 174567,
                    ("Package", "publish_native") => (input_size * 13 + 10908) >> 2, // calculated using linear regression on gathered data
                    ("Package", "publish_wasm_advanced") => input_size * 25 + 218153, // calculated using linear regression on gathered data
                    _ => 454121, // average of above values
                },
                FnIdent::System(value) => {
                    match (identifier.blueprint.blueprint_name.as_str(), value) {
                        ("Identity", 0) => 333450,
                        ("Account", 0) => 220335,
                        _ => 276893, // average of above values
                    }
                }
                _ => 365507, // average of above values
            },
            CostingEntry::LockSubstate {
                node_id,
                module_id,
                substate_key: _,
            } => match (module_id, node_id.entity_type()) {
                (SysModuleId::AccessRules, Some(EntityType::GlobalAccessController)) => 3517,
                (SysModuleId::AccessRules, Some(EntityType::GlobalAccount)) => 2253,
                (SysModuleId::AccessRules, Some(EntityType::GlobalClock)) => 1973,
                (SysModuleId::AccessRules, Some(EntityType::GlobalEpochManager)) => 2350,
                (SysModuleId::AccessRules, Some(EntityType::GlobalFungibleResource)) => 1045,
                (SysModuleId::AccessRules, Some(EntityType::GlobalGenericComponent)) => 1655,
                (SysModuleId::AccessRules, Some(EntityType::GlobalIdentity)) => 1943,
                (SysModuleId::AccessRules, Some(EntityType::GlobalNonFungibleResource)) => 1012,
                (SysModuleId::AccessRules, Some(EntityType::GlobalPackage)) => 2638,
                (SysModuleId::AccessRules, Some(EntityType::GlobalValidator)) => 2315,
                (SysModuleId::AccessRules, Some(EntityType::GlobalVirtualEcdsaAccount)) => 1028,
                (SysModuleId::AccessRules, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 1019,
                (SysModuleId::Metadata, Some(EntityType::GlobalAccount)) => 1645,
                (SysModuleId::Metadata, Some(EntityType::GlobalFungibleResource)) => 1221,
                (SysModuleId::Metadata, Some(EntityType::GlobalGenericComponent)) => 1412,
                (SysModuleId::Metadata, Some(EntityType::GlobalIdentity)) => 1460,
                (SysModuleId::Metadata, Some(EntityType::GlobalPackage)) => 1415,
                (SysModuleId::Metadata, Some(EntityType::GlobalValidator)) => 1475,
                (SysModuleId::Metadata, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 1496,
                (SysModuleId::ObjectMap, Some(EntityType::InternalGenericComponent)) => 653,
                (SysModuleId::ObjectMap, Some(EntityType::InternalKeyValueStore)) => 1827,
                (SysModuleId::ObjectTuple, Some(EntityType::GlobalAccessController)) => 2255,
                (SysModuleId::ObjectTuple, Some(EntityType::GlobalAccount)) => 1064,
                (SysModuleId::ObjectTuple, Some(EntityType::GlobalClock)) => 1000,
                (SysModuleId::ObjectTuple, Some(EntityType::GlobalEpochManager)) => 1304,
                (SysModuleId::ObjectTuple, Some(EntityType::GlobalFungibleResource)) => 1001,
                (SysModuleId::ObjectTuple, Some(EntityType::GlobalGenericComponent)) => 1110,
                (SysModuleId::ObjectTuple, Some(EntityType::GlobalNonFungibleResource)) => 1052,
                (SysModuleId::ObjectTuple, Some(EntityType::GlobalPackage)) => 971,
                (SysModuleId::ObjectTuple, Some(EntityType::GlobalValidator)) => 2270,
                (SysModuleId::ObjectTuple, Some(EntityType::GlobalVirtualEcdsaAccount)) => 1050,
                (SysModuleId::ObjectTuple, Some(EntityType::InternalFungibleVault)) => 1274,
                (SysModuleId::ObjectTuple, Some(EntityType::InternalGenericComponent)) => 875,
                (SysModuleId::ObjectTuple, Some(EntityType::InternalNonFungibleVault)) => 990,
                (SysModuleId::Royalty, Some(EntityType::GlobalAccessController)) => 1292,
                (SysModuleId::Royalty, Some(EntityType::GlobalAccount)) => 1285,
                (SysModuleId::Royalty, Some(EntityType::GlobalClock)) => 1286,
                (SysModuleId::Royalty, Some(EntityType::GlobalEpochManager)) => 1302,
                (SysModuleId::Royalty, Some(EntityType::GlobalGenericComponent)) => 1273,
                (SysModuleId::Royalty, Some(EntityType::GlobalIdentity)) => 1292,
                (SysModuleId::Royalty, Some(EntityType::GlobalValidator)) => 1292,
                (SysModuleId::Royalty, Some(EntityType::GlobalVirtualEcdsaAccount)) => 1268,
                (SysModuleId::Royalty, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 987,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalAccessController)) => 1006,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalAccount)) => 1013,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalClock)) => 1025,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalEpochManager)) => 1035,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalFungibleResource)) => 1004,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalGenericComponent)) => 1007,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalIdentity)) => 1000,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalNonFungibleResource)) => 1006,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalPackage)) => 1009,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalValidator)) => 1011,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalVirtualEcdsaAccount)) => 1013,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 1017,
                (SysModuleId::TypeInfo, Some(EntityType::InternalAccount)) => 1156,
                (SysModuleId::TypeInfo, Some(EntityType::InternalFungibleVault)) => 1019,
                (SysModuleId::TypeInfo, Some(EntityType::InternalGenericComponent)) => 863,
                (SysModuleId::TypeInfo, Some(EntityType::InternalKeyValueStore)) => 989,
                (SysModuleId::TypeInfo, Some(EntityType::InternalNonFungibleVault)) => 912,
                _ => 1332, // average of above values
            },
            CostingEntry::ReadBucket => 186,
            CostingEntry::ReadProof => 236,
            CostingEntry::ReadSubstate { size: _ } => 220,
            CostingEntry::WriteSubstate { size: _ } => 205,
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
