use crate::system::system_callback::SystemInvocation;
use crate::types::*;

pub const FIXED_LOW_FEE: u32 = 500;
pub const FIXED_MEDIUM_FEE: u32 = 2500;
pub const FIXED_HIGH_FEE: u32 = 5000;

const COSTING_COEFFICENT: u64 = 335;
const COSTING_COEFFICENT_DIV_BITS: u64 = 4; // used to divide by shift left operator
const COSTING_COEFFICENT_DIV_BITS_ADDON: u64 = 6; // used to scale up or down all cpu instruction costing

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
        virtual_node: bool,
    },

    /* substate */
    LockSubstate {
        node_id: &'a NodeId,
        module_id: &'a SysModuleId,
        substate_key: &'a SubstateKey,
    },
    LockSubstateFirstTime,
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
            CostingEntry::AllocateNodeId { virtual_node } => {
                if *virtual_node {
                    113
                } else {
                    212
                }
            }
            CostingEntry::CreateNode { size: _, node_id } => match node_id.entity_type() {
                Some(EntityType::GlobalAccessController) => 1736,
                Some(EntityType::GlobalAccount) => 1640,
                Some(EntityType::GlobalClock) => 987,
                Some(EntityType::GlobalEpochManager) => 1203,
                Some(EntityType::GlobalFungibleResource) => 1160,
                Some(EntityType::GlobalGenericComponent) => 2370,
                Some(EntityType::GlobalIdentity) => 838,
                Some(EntityType::GlobalNonFungibleResource) => 1587,
                Some(EntityType::GlobalPackage) => 1493,
                Some(EntityType::GlobalValidator) => 2374,
                Some(EntityType::GlobalVirtualEcdsaAccount) => 1590,
                Some(EntityType::GlobalVirtualEcdsaIdentity) => 906,
                Some(EntityType::InternalAccount) => 329,
                Some(EntityType::InternalFungibleVault) => 368,
                Some(EntityType::InternalGenericComponent) => 336,
                Some(EntityType::InternalKeyValueStore) => 828,
                Some(EntityType::InternalNonFungibleVault) => 356,
                _ => 1182, // average of above values
            },
            CostingEntry::DropLock => 114,
            CostingEntry::DropNode { size: _ } => 324, // average of gathered data
            CostingEntry::Invoke {
                input_size,
                identifier,
            } => match &identifier.ident {
                FnIdent::Application(fn_name) => match (
                    identifier.blueprint.blueprint_name.as_str(),
                    fn_name.as_str(),
                ) {
                    ("AccessController", "cancel_recovery_role_recovery_proposal") => 150860,
                    ("AccessController", "create_global") => 625871,
                    ("AccessController", "create_proof") => 348146,
                    ("AccessController", "initiate_recovery_as_primary") => 153643,
                    ("AccessController", "initiate_recovery_as_recovery") => 180020,
                    ("AccessController", "lock_primary_role") => 148500,
                    ("AccessController", "quick_confirm_primary_role_recovery_proposal") => 496785,
                    ("AccessController", "quick_confirm_recovery_role_recovery_proposal") => 472253,
                    ("AccessController", "stop_timed_recovery") => 220240,
                    ("AccessController", "timed_confirm_recovery") => 502227,
                    ("AccessController", "unlock_primary_role") => 149792,
                    ("AccessRules", "create") => 62071,
                    ("AccessRules", "set_group_access_rule") => 49335,
                    ("AccessRules", "set_group_access_rule_and_mutability") => 58507,
                    ("AccessRules", "set_group_mutability") => 143886,
                    ("AccessRules", "set_method_access_rule") => 49944,
                    ("AccessRules", "set_method_access_rule_and_mutability") => 58830,
                    ("AccessRules", "set_method_mutability") => 144136,
                    ("Account", "create_advanced") => 214769,
                    ("Account", "create_proof") => 270726,
                    ("Account", "create_proof_by_amount") => 188957,
                    ("Account", "create_proof_by_ids") => 268647,
                    ("Account", "deposit") => 451134,
                    ("Account", "deposit_batch") => 548662,
                    ("Account", "lock_contingent_fee") => 195753,
                    ("Account", "lock_fee") => 272297,
                    ("Account", "lock_fee_and_withdraw") => 487906,
                    ("Account", "lock_fee_and_withdraw_non_fungibles") => 491586,
                    ("Account", "securify") => 504675,
                    ("Account", "withdraw") => 256821,
                    ("Account", "withdraw_non_fungibles") => 274091,
                    ("AuthZone", "clear") => 70545,
                    ("AuthZone", "clear_signature_proofs") => 69885,
                    ("AuthZone", "create_proof") => 304813,
                    ("AuthZone", "create_proof_by_amount") => 374357,
                    ("AuthZone", "create_proof_by_ids") => 420809,
                    ("AuthZone", "drain") => 70903,
                    ("AuthZone", "pop") => 68617,
                    ("AuthZone", "push") => 70544,
                    ("Bucket", "Bucket_create_proof") => 136828,
                    ("Bucket", "Bucket_drop_empty") => 138335,
                    ("Bucket", "Bucket_get_amount") => 71523,
                    ("Bucket", "Bucket_get_non_fungible_local_ids") => 73029,
                    ("Bucket", "Bucket_get_resource_address") => 69372,
                    ("Bucket", "Bucket_lock_amount") => 71538,
                    ("Bucket", "Bucket_lock_non_fungibles") => 72278,
                    ("Bucket", "Bucket_put") => 73970,
                    ("Bucket", "Bucket_take") => 132523,
                    ("Bucket", "Bucket_take_non_fungibles") => 133129,
                    ("Bucket", "Bucket_unlock_amount") => 72000,
                    ("Bucket", "Bucket_unlock_non_fungibles") => 71782,
                    ("Bucket", "burn_bucket") => 146562,
                    ("Clock", "compare_current_time") => 21977,
                    ("Clock", "create") => 82707,
                    ("Clock", "get_current_time") => 27418,
                    ("Clock", "set_current_time") => 28501,
                    ("ComponentRoyalty", "claim_royalty") => 338240,
                    ("ComponentRoyalty", "create") => 11210,
                    ("ComponentRoyalty", "set_royalty_config") => 164577,
                    ("EpochManager", "create") => 362236,
                    ("EpochManager", "create_validator") => 1598879,
                    ("EpochManager", "get_current_epoch") => 44925,
                    ("EpochManager", "next_round") => 61384,
                    ("EpochManager", "set_epoch") => 45371,
                    ("EpochManager", "update_validator") => 37241,
                    ("Faucet", "free") => 640620,
                    ("Faucet", "lock_fee") => 462708,
                    ("Faucet", "new") => 6535050,
                    ("FungibleResourceManager", "burn") => 212493,
                    ("FungibleResourceManager", "create") => 261408,
                    ("FungibleResourceManager", "create_bucket") => 206080,
                    ("FungibleResourceManager", "create_vault") => 255636,
                    ("FungibleResourceManager", "create_with_initial_supply") => 323158,
                    ("FungibleResourceManager", "create_with_initial_supply_and_address") => 351778,
                    ("FungibleResourceManager", "get_resource_type") => 143996,
                    ("FungibleResourceManager", "get_total_supply") => 145197,
                    ("FungibleResourceManager", "mint") => 333785,
                    ("FungibleVault", "create_proof_of_all") => 203485,
                    ("FungibleVault", "create_proof_of_amount") => 139486,
                    ("FungibleVault", "get_amount") => 78722,
                    ("FungibleVault", "lock_fee") => 207640,
                    ("FungibleVault", "lock_fungible_amount") => 77750,
                    ("FungibleVault", "put") => 134713,
                    ("FungibleVault", "recall") => 266062,
                    ("FungibleVault", "take") => 203677,
                    ("FungibleVault", "unlock_fungible_amount") => 132108,
                    ("GenesisHelper", "init") => 4567874,
                    ("Identity", "create") => 567462,
                    ("Identity", "create_advanced") => 199867,
                    ("Identity", "securify") => 468393,
                    ("Metadata", "create") => 18465,
                    ("Metadata", "create_with_data") => 18385,
                    ("Metadata", "get") => 20114,
                    ("Metadata", "remove") => 37509,
                    ("Metadata", "set") => 36472,
                    ("NonFungibleResourceManager", "burn") => 240244,
                    ("NonFungibleResourceManager", "create") => 278588,
                    ("NonFungibleResourceManager", "create_bucket") => 222832,
                    ("NonFungibleResourceManager", "create_non_fungible_with_address") => 248723,
                    (
                        "NonFungibleResourceManager",
                        "create_uuid_non_fungible_with_initial_supply",
                    ) => 371965,
                    ("NonFungibleResourceManager", "create_vault") => 290686,
                    ("NonFungibleResourceManager", "create_with_initial_supply") => 348024,
                    ("NonFungibleResourceManager", "get_non_fungible") => 161527,
                    ("NonFungibleResourceManager", "get_resource_type") => 157541,
                    ("NonFungibleResourceManager", "get_total_supply") => 161935,
                    ("NonFungibleResourceManager", "mint") => 370268,
                    ("NonFungibleResourceManager", "mint_single_uuid") => 304487,
                    ("NonFungibleResourceManager", "mint_uuid") => 304126,
                    ("NonFungibleResourceManager", "non_fungible_exists") => 161135,
                    ("NonFungibleResourceManager", "update_non_fungible_data") => 237009,
                    ("NonFungibleVault", "create_proof_of_all") => 212614,
                    ("NonFungibleVault", "create_proof_of_amount") => 206574,
                    ("NonFungibleVault", "create_proof_of_non_fungibles") => 208925,
                    ("NonFungibleVault", "get_amount") => 78340,
                    ("NonFungibleVault", "get_non_fungible_local_ids") => 79115,
                    ("NonFungibleVault", "lock_non_fungibles") => 140379,
                    ("NonFungibleVault", "put") => 142043,
                    ("NonFungibleVault", "recall") => 267245,
                    ("NonFungibleVault", "take") => 209884,
                    ("NonFungibleVault", "take_non_fungibles") => 210402,
                    ("NonFungibleVault", "unlock_non_fungibles") => 140611,
                    ("Package", "PackageRoyalty_claim_royalty") => 460102,
                    ("Package", "PackageRoyalty_set_royalty_config") => 217143,
                    ("Package", "publish_wasm") => 458988,
                    ("Proof", "Proof_drop") => 198758,
                    ("Proof", "Proof_get_amount") => 69914,
                    ("Proof", "Proof_get_non_fungible_local_ids") => 71120,
                    ("Proof", "Proof_get_resource_address") => 67936,
                    ("Proof", "clone") => 205434,
                    ("Radiswap", "instantiate_pool") => 10609872,
                    ("Radiswap", "swap") => 3181336,
                    ("TransactionProcessor", "run") => 1770226,
                    ("Validator", "claim_xrd") => 899795,
                    ("Validator", "register") => 280382,
                    ("Validator", "stake") => 1113263,
                    ("Validator", "unregister") => 239553,
                    ("Validator", "unstake") => 1432558,
                    ("Validator", "update_accept_delegated_stake") => 256401,
                    ("Validator", "update_key") => 312758,
                    ("Worktop", "Worktop_drain") => 69013,
                    ("Worktop", "Worktop_drop") => 66830,
                    ("Worktop", "Worktop_put") => 212750,
                    ("Worktop", "Worktop_take") => 277865,
                    ("Worktop", "Worktop_take_all") => 68365,
                    ("Worktop", "Worktop_take_non_fungibles") => 146198,
                    ("Package", "publish_native") => (input_size * 13 + 10910) >> 2, // calculated using linear regression on gathered data
                    ("Package", "publish_wasm_advanced") => input_size * 22 + 289492, // calculated using linear regression on gathered data
                    _ => 411524, // average of above values without Package::publish_native and Package::publish_wasm_advanced
                },
                FnIdent::System(value) => {
                    match (identifier.blueprint.blueprint_name.as_str(), value) {
                        ("Identity", 0) => 252633,
                        ("Account", 0) => 220211,
                        _ => 236422, // average of above values
                    }
                }
            },
            CostingEntry::LockSubstate {
                node_id,
                module_id,
                substate_key: _,
            } => match (module_id, node_id.entity_type()) {
                (SysModuleId::AccessRules, Some(EntityType::GlobalAccessController)) => 2822,
                (SysModuleId::AccessRules, Some(EntityType::GlobalAccount)) => 1564,
                (SysModuleId::AccessRules, Some(EntityType::GlobalClock)) => 1283,
                (SysModuleId::AccessRules, Some(EntityType::GlobalEpochManager)) => 1644,
                (SysModuleId::AccessRules, Some(EntityType::GlobalFungibleResource)) => 327,
                (SysModuleId::AccessRules, Some(EntityType::GlobalGenericComponent)) => 968,
                (SysModuleId::AccessRules, Some(EntityType::GlobalIdentity)) => 1257,
                (SysModuleId::AccessRules, Some(EntityType::GlobalNonFungibleResource)) => 310,
                (SysModuleId::AccessRules, Some(EntityType::GlobalPackage)) => 1920,
                (SysModuleId::AccessRules, Some(EntityType::GlobalValidator)) => 1626,
                (SysModuleId::AccessRules, Some(EntityType::GlobalVirtualEcdsaAccount)) => 328,
                (SysModuleId::AccessRules, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 321,
                (SysModuleId::Metadata, Some(EntityType::GlobalAccount)) => 787,
                (SysModuleId::Metadata, Some(EntityType::GlobalFungibleResource)) => 600,
                (SysModuleId::Metadata, Some(EntityType::GlobalGenericComponent)) => 632,
                (SysModuleId::Metadata, Some(EntityType::GlobalIdentity)) => 700,
                (SysModuleId::Metadata, Some(EntityType::GlobalPackage)) => 622,
                (SysModuleId::Metadata, Some(EntityType::GlobalValidator)) => 689,
                (SysModuleId::Metadata, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 707,
                (SysModuleId::Virtualized, Some(EntityType::InternalGenericComponent)) => 650,
                (SysModuleId::Virtualized, Some(EntityType::InternalKeyValueStore)) => 671,
                (SysModuleId::Object, Some(EntityType::GlobalAccessController)) => 1562,
                (SysModuleId::Object, Some(EntityType::GlobalAccount)) => 356,
                (SysModuleId::Object, Some(EntityType::GlobalClock)) => 290,
                (SysModuleId::Object, Some(EntityType::GlobalEpochManager)) => 675,
                (SysModuleId::Object, Some(EntityType::GlobalFungibleResource)) => 303,
                (SysModuleId::Object, Some(EntityType::GlobalGenericComponent)) => 413,
                (SysModuleId::Object, Some(EntityType::GlobalNonFungibleResource)) => 355,
                (SysModuleId::Object, Some(EntityType::GlobalPackage)) => 290,
                (SysModuleId::Object, Some(EntityType::GlobalValidator)) => 1568,
                (SysModuleId::Object, Some(EntityType::GlobalVirtualEcdsaAccount)) => 355,
                (SysModuleId::Object, Some(EntityType::InternalFungibleVault)) => 577,
                (SysModuleId::Object, Some(EntityType::InternalGenericComponent)) => 177,
                (SysModuleId::Object, Some(EntityType::InternalNonFungibleVault)) => 290,
                (SysModuleId::Royalty, Some(EntityType::GlobalAccessController)) => 609,
                (SysModuleId::Royalty, Some(EntityType::GlobalAccount)) => 593,
                (SysModuleId::Royalty, Some(EntityType::GlobalClock)) => 596,
                (SysModuleId::Royalty, Some(EntityType::GlobalEpochManager)) => 605,
                (SysModuleId::Royalty, Some(EntityType::GlobalGenericComponent)) => 585,
                (SysModuleId::Royalty, Some(EntityType::GlobalIdentity)) => 605,
                (SysModuleId::Royalty, Some(EntityType::GlobalValidator)) => 604,
                (SysModuleId::Royalty, Some(EntityType::GlobalVirtualEcdsaAccount)) => 577,
                (SysModuleId::Royalty, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 290,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalAccessController)) => 309,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalAccount)) => 311,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalClock)) => 324,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalEpochManager)) => 333,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalFungibleResource)) => 313,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalGenericComponent)) => 318,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalIdentity)) => 307,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalNonFungibleResource)) => 311,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalPackage)) => 315,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalValidator)) => 312,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalVirtualEcdsaAccount)) => 318,
                (SysModuleId::TypeInfo, Some(EntityType::GlobalVirtualEcdsaIdentity)) => 323,
                (SysModuleId::TypeInfo, Some(EntityType::InternalAccount)) => 461,
                (SysModuleId::TypeInfo, Some(EntityType::InternalFungibleVault)) => 321,
                (SysModuleId::TypeInfo, Some(EntityType::InternalGenericComponent)) => 173,
                (SysModuleId::TypeInfo, Some(EntityType::InternalKeyValueStore)) => 290,
                (SysModuleId::TypeInfo, Some(EntityType::InternalNonFungibleVault)) => 207,
                _ => 632, // average of above values
            },
            CostingEntry::LockSubstateFirstTime => 100, // todo: determine correct value
            CostingEntry::ReadSubstate { size: _ } => 174,
            CostingEntry::WriteSubstate { size: _ } => 126,
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
