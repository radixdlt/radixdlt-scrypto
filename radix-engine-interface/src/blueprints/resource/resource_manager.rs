use crate::blueprints::resource::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;

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
pub enum ResourceFeature {
    Mint,
    Burn,
    Recall,
    Freeze,
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

pub const RESOURCE_MANAGER_GET_AMOUNT_FOR_WITHDRAWAL_IDENT: &str = "amount_for_withdrawal";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct ResourceManagerGetAmountForWithdrawalInput {
    pub request_amount: Decimal,
    pub withdraw_strategy: WithdrawStrategy,
}

pub type ResourceManagerGetAmountForWithdrawalOutput = Decimal;
