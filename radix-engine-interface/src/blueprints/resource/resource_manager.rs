use crate::blueprints::resource::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_interface::api::node_modules::metadata::{
    METADATA_ADMIN_ROLE, METADATA_ADMIN_UPDATER_ROLE,
};
use radix_engine_interface::api::ObjectModuleId;

pub const TRACK_TOTAL_SUPPLY_FEATURE: &str = "track_total_supply";
pub const VAULT_FREEZE_FEATURE: &str = "vault_freeze";
pub const VAULT_RECALL_FEATURE: &str = "vault_recall";
pub const MINT_FEATURE: &str = "mint";
pub const BURN_FEATURE: &str = "burn";

// Meta-roles
pub const RESOURCE_PACKAGE_ROLE: &str = "resource_package";

// Main roles
pub const MINT_ROLE: &str = "minter";
pub const MINT_UPDATE_ROLE: &str = "minter_updater";
pub const BURN_ROLE: &str = "burner";
pub const BURN_UPDATE_ROLE: &str = "burner_updater";
pub const WITHDRAW_ROLE: &str = "withdrawer";
pub const WITHDRAW_UPDATE_ROLE: &str = "withdrawer_updater";
pub const DEPOSIT_ROLE: &str = "depositor";
pub const DEPOSIT_UPDATE_ROLE: &str = "depositor_updater";
pub const RECALL_ROLE: &str = "recaller";
pub const RECALL_UPDATE_ROLE: &str = "recaller_updater";
pub const FREEZE_ROLE: &str = "freezer";
pub const FREEZE_UPDATER_ROLE: &str = "freezer_updater";
pub const UPDATE_NON_FUNGIBLE_DATA_ROLE: &str = "non_fungible_data_updater";
pub const UPDATE_NON_FUNGIBLE_DATA_UPDATE_ROLE: &str = "non_fungible_data_updater_updater";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, ScryptoSbor, ManifestSbor)]
pub enum ResourceMethodAuthKey {
    Mint,
    Burn,
    UpdateNonFungibleData,
    UpdateMetadata,
    Withdraw,
    Deposit,
    Recall,
    Freeze,
}

pub const ALL_RESOURCE_AUTH_KEYS: [ResourceMethodAuthKey; 8] = [
    ResourceMethodAuthKey::Mint,
    ResourceMethodAuthKey::Burn,
    ResourceMethodAuthKey::UpdateNonFungibleData,
    ResourceMethodAuthKey::UpdateMetadata,
    ResourceMethodAuthKey::Withdraw,
    ResourceMethodAuthKey::Deposit,
    ResourceMethodAuthKey::Recall,
    ResourceMethodAuthKey::Freeze,
];

impl ResourceMethodAuthKey {
    pub fn action_role_key(&self) -> (ObjectModuleId, RoleKey) {
        match self {
            Self::Mint => (ObjectModuleId::Main, RoleKey::new(MINT_ROLE)),
            Self::Burn => (ObjectModuleId::Main, RoleKey::new(BURN_ROLE)),
            Self::UpdateNonFungibleData => (
                ObjectModuleId::Main,
                RoleKey::new(UPDATE_NON_FUNGIBLE_DATA_ROLE),
            ),
            Self::Withdraw => (ObjectModuleId::Main, RoleKey::new(WITHDRAW_ROLE)),
            Self::Deposit => (ObjectModuleId::Main, RoleKey::new(DEPOSIT_ROLE)),
            Self::Recall => (ObjectModuleId::Main, RoleKey::new(RECALL_ROLE)),
            Self::Freeze => (ObjectModuleId::Main, RoleKey::new(FREEZE_ROLE)),

            Self::UpdateMetadata => (ObjectModuleId::Metadata, RoleKey::new(METADATA_ADMIN_ROLE)),
        }
    }

    pub fn updater_role_key(&self) -> (ObjectModuleId, RoleKey) {
        match self {
            Self::Mint => (ObjectModuleId::Main, RoleKey::new(MINT_UPDATE_ROLE)),
            Self::Burn => (ObjectModuleId::Main, RoleKey::new(BURN_UPDATE_ROLE)),
            Self::UpdateNonFungibleData => (
                ObjectModuleId::Main,
                RoleKey::new(UPDATE_NON_FUNGIBLE_DATA_UPDATE_ROLE),
            ),
            Self::Withdraw => (ObjectModuleId::Main, RoleKey::new(WITHDRAW_UPDATE_ROLE)),
            Self::Deposit => (ObjectModuleId::Main, RoleKey::new(DEPOSIT_UPDATE_ROLE)),
            Self::Recall => (ObjectModuleId::Main, RoleKey::new(RECALL_UPDATE_ROLE)),
            Self::Freeze => (
                ObjectModuleId::Main,
                RoleKey::new(FREEZE_UPDATER_ROLE),
            ),

            Self::UpdateMetadata => (
                ObjectModuleId::Metadata,
                RoleKey::new(METADATA_ADMIN_UPDATER_ROLE),
            ),
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
