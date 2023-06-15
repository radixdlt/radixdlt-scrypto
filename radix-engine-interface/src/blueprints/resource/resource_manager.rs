use crate::blueprints::resource::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;

pub const TRACK_TOTAL_SUPPLY_FEATURE: &str = "track_total_supply";

// Meta-roles
pub const RESOURCE_PACKAGE_ROLE: &str = "resource_package";

// Main roles
pub const MINT_ROLE: &str = "mint";
pub const MINT_UPDATE_ROLE: &str = "mint_update";
pub const BURN_ROLE: &str = "burn";
pub const BURN_UPDATE_ROLE: &str = "burn_update";
pub const WITHDRAW_ROLE: &str = "withdraw";
pub const WITHDRAW_UPDATE_ROLE: &str = "withdraw_update";
pub const DEPOSIT_ROLE: &str = "deposit";
pub const DEPOSIT_UPDATE_ROLE: &str = "deposit_update";
pub const RECALL_ROLE: &str = "recall";
pub const RECALL_UPDATE_ROLE: &str = "recall_update";
pub const FREEZE_ROLE: &str = "freeze";
pub const FREEZE_UPDATE_ROLE: &str = "freeze_update";
pub const UNFREEZE_ROLE: &str = "unfreeze";
pub const UNFREEZE_UPDATE_ROLE: &str = "unfreeze_update";
pub const UPDATE_NON_FUNGIBLE_DATA_ROLE: &str = "update_non_fungible_data";
pub const UPDATE_NON_FUNGIBLE_DATA_UPDATE_ROLE: &str = "update_non_fungible_data_update";
pub const SET_METADATA_ROLE: &str = "set_metadata";
pub const SET_METADATA_UPDATE_ROLE: &str = "set_metadata_update";

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
    Unfreeze,
}

pub const ALL_RESOURCE_AUTH_KEYS: [ResourceMethodAuthKey; 9] = [
    ResourceMethodAuthKey::Mint,
    ResourceMethodAuthKey::Burn,
    ResourceMethodAuthKey::UpdateNonFungibleData,
    ResourceMethodAuthKey::UpdateMetadata,
    ResourceMethodAuthKey::Withdraw,
    ResourceMethodAuthKey::Deposit,
    ResourceMethodAuthKey::Recall,
    ResourceMethodAuthKey::Freeze,
    ResourceMethodAuthKey::Unfreeze,
];

impl ResourceMethodAuthKey {
    pub fn action_role_key(&self) -> RoleKey {
        match self {
            Self::Mint => RoleKey::new(MINT_ROLE),
            Self::Burn => RoleKey::new(BURN_ROLE),
            Self::UpdateNonFungibleData => RoleKey::new(UPDATE_NON_FUNGIBLE_DATA_ROLE),
            Self::UpdateMetadata => RoleKey::new(SET_METADATA_ROLE),
            Self::Withdraw => RoleKey::new(WITHDRAW_ROLE),
            Self::Deposit => RoleKey::new(DEPOSIT_ROLE),
            Self::Recall => RoleKey::new(RECALL_ROLE),
            Self::Freeze => RoleKey::new(FREEZE_ROLE),
            Self::Unfreeze => RoleKey::new(UNFREEZE_ROLE),
        }
    }

    pub fn updater_role_key(&self) -> RoleKey {
        match self {
            Self::Mint => RoleKey::new(MINT_UPDATE_ROLE),
            Self::Burn => RoleKey::new(BURN_UPDATE_ROLE),
            Self::UpdateNonFungibleData => RoleKey::new(UPDATE_NON_FUNGIBLE_DATA_UPDATE_ROLE),
            Self::UpdateMetadata => RoleKey::new(SET_METADATA_UPDATE_ROLE),
            Self::Withdraw => RoleKey::new(WITHDRAW_UPDATE_ROLE),
            Self::Deposit => RoleKey::new(DEPOSIT_UPDATE_ROLE),
            Self::Recall => RoleKey::new(RECALL_UPDATE_ROLE),
            Self::Freeze => RoleKey::new(FREEZE_UPDATE_ROLE),
            Self::Unfreeze => RoleKey::new(UNFREEZE_UPDATE_ROLE),
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
