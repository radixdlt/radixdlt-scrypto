use crate::blueprints::resource::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_interface::api::node_modules::auth::RoleDefinition;
use radix_engine_interface::api::ObjectModuleId;

pub const TRACK_TOTAL_SUPPLY_FEATURE: &str = "track_total_supply";
pub const VAULT_FREEZE_FEATURE: &str = "vault_freeze";
pub const VAULT_RECALL_FEATURE: &str = "vault_recall";
pub const MINT_FEATURE: &str = "mint";
pub const BURN_FEATURE: &str = "burn";

// Meta-roles
pub const RESOURCE_PACKAGE_ROLE: &str = "resource_package";

// Main roles
pub const MINTER_ROLE: &str = "minter";
pub const MINTER_UPDATER_ROLE: &str = "minter_updater";
pub const BURNER_ROLE: &str = "burner";
pub const BURNER_UPDATER_ROLE: &str = "burner_updater";
pub const WITHDRAWER_ROLE: &str = "withdrawer";
pub const WITHDRAWER_UPDATER_ROLE: &str = "withdrawer_updater";
pub const DEPOSITOR_ROLE: &str = "depositor";
pub const DEPOSITOR_UPDATER_ROLE: &str = "depositor_updater";
pub const RECALLER_ROLE: &str = "recaller";
pub const RECALLER_UPDATER_ROLE: &str = "recaller_updater";
pub const FREEZER_ROLE: &str = "freezer";
pub const FREEZER_UPDATER_ROLE: &str = "freezer_updater";
pub const NON_FUNGIBLE_DATA_UPDATER_ROLE: &str = "non_fungible_data_updater";
pub const NON_FUNGIBLE_DATA_UPDATER_UPDATER_ROLE: &str = "non_fungible_data_updater_updater";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor, ManifestSbor)]
pub enum ResourceAction {
    Mint,
    Burn,
    Recall,
    Freeze,
}

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct ResourceActionRoleInit {
    pub actor: RoleDefinition,
    pub updater: RoleDefinition,
}

impl ResourceActionRoleInit {
    pub fn locked(actor: AccessRule) -> Self {
        Self {
            actor: RoleDefinition::locked(actor),
            updater: RoleDefinition::locked(AccessRule::DenyAll),
        }
    }
}

pub const RESOURCE_MANAGER_BURN_IDENT: &str = "burn";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerBurnInput {
    pub bucket: Bucket,
}

pub type ResourceManagerBurnOutput = ();

pub const RESOURCE_MANAGER_PACKAGE_BURN_IDENT: &str = "package_burn";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerPackageBurnInput {
    pub bucket: Bucket,
}

pub type ResourceManagerPackageBurnOutput = ();

pub const RESOURCE_MANAGER_CREATE_EMPTY_VAULT_IDENT: &str = "create_empty_vault";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateEmptyVaultInput {}

pub type ResourceManagerCreateEmptyVaultOutput = Vault;

pub const RESOURCE_MANAGER_CREATE_EMPTY_BUCKET_IDENT: &str = "create_empty_bucket";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerCreateEmptyBucketInput {}

pub type ResourceManagerCreateEmptyBucketOutput = Bucket;

pub const RESOURCE_MANAGER_DROP_EMPTY_BUCKET_IDENT: &str = "drop_empty_bucket";

#[derive(Debug, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerDropEmptyBucketInput {
    pub bucket: Bucket,
}

pub type ResourceManagerDropEmptyBucketOutput = ();

pub const RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT: &str = "get_resource_type";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetResourceTypeInput {}

pub type ResourceManagerGetResourceTypeOutput = ResourceType;

pub const RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT: &str = "get_total_supply";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetTotalSupplyInput {}

pub type ResourceManagerGetTotalSupplyOutput = Option<Decimal>;
