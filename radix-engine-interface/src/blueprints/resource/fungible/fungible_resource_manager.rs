use crate::blueprints::resource::*;
use crate::internal_prelude::*;
#[cfg(feature = "fuzzing")]
use arbitrary::Arbitrary;
use radix_common::data::manifest::model::ManifestAddressReservation;
use radix_common::types::*;
use radix_engine_interface::object_modules::metadata::MetadataInit;
use radix_engine_interface::object_modules::ModuleConfig;

pub const FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT: &str = "FungibleResourceManager";

pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT: &str = "create";

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Default, Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleResourceRoles {
    pub mint_roles: Option<MintRoles<RoleDefinition>>,
    pub burn_roles: Option<BurnRoles<RoleDefinition>>,
    pub freeze_roles: Option<FreezeRoles<RoleDefinition>>,
    pub recall_roles: Option<RecallRoles<RoleDefinition>>,
    pub withdraw_roles: Option<WithdrawRoles<RoleDefinition>>,
    pub deposit_roles: Option<DepositRoles<RoleDefinition>>,
}

impl FungibleResourceRoles {
    pub fn default_with_owner_mint_burn() -> Self {
        Self {
            mint_roles: mint_roles! {
                minter => OWNER;
                minter_updater => rule!(deny_all);
            },
            burn_roles: burn_roles! {
                burner => OWNER;
                burner_updater => rule!(deny_all);
            },
            ..Default::default()
        }
    }

    pub fn single_locked_rule(access_rule: AccessRule) -> Self {
        Self {
            mint_roles: mint_roles! {
                minter => access_rule.clone();
                minter_updater => rule!(deny_all);
            },
            burn_roles: burn_roles! {
                burner => access_rule.clone();
                burner_updater => rule!(deny_all);
            },
            freeze_roles: freeze_roles! {
                freezer => access_rule.clone();
                freezer_updater => rule!(deny_all);
            },
            recall_roles: recall_roles! {
                recaller => access_rule.clone();
                recaller_updater => rule!(deny_all);
            },
            withdraw_roles: withdraw_roles! {
                withdrawer => access_rule.clone();
                withdrawer_updater => rule!(deny_all);
            },
            deposit_roles: deposit_roles! {
                depositor => access_rule;
                depositor_updater => rule!(deny_all);
            },
        }
    }
}

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleResourceManagerCreateInput {
    pub owner_role: OwnerRole,
    pub track_total_supply: bool,
    pub divisibility: u8,
    pub resource_roles: FungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<GlobalAddressReservation>,
}

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct FungibleResourceManagerCreateManifestInput {
    pub owner_role: OwnerRole,
    pub track_total_supply: bool,
    pub divisibility: u8,
    pub resource_roles: FungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<ManifestAddressReservation>,
}

pub type FungibleResourceManagerCreateOutput = ResourceAddress;

pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_WITH_INITIAL_SUPPLY_IDENT: &str =
    "create_with_initial_supply";

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleResourceManagerCreateWithInitialSupplyInput {
    pub owner_role: OwnerRole,
    pub track_total_supply: bool,
    pub divisibility: u8,
    pub initial_supply: Decimal,
    pub resource_roles: FungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<GlobalAddressReservation>,
}

#[cfg_attr(feature = "fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct FungibleResourceManagerCreateWithInitialSupplyManifestInput {
    pub owner_role: OwnerRole,
    pub track_total_supply: bool,
    pub divisibility: u8,
    pub initial_supply: Decimal,
    pub resource_roles: FungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<ManifestAddressReservation>,
}

pub type FungibleResourceManagerCreateWithInitialSupplyOutput = (ResourceAddress, Bucket);

pub const FUNGIBLE_RESOURCE_MANAGER_MINT_IDENT: &str = "mint";

#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct FungibleResourceManagerMintInput {
    pub amount: Decimal,
}

pub type FungibleResourceManagerMintManifestInput = FungibleResourceManagerMintInput;

pub type FungibleResourceManagerMintOutput = Bucket;

pub type FungibleResourceManagerCreateEmptyBucketInput = ResourceManagerCreateEmptyBucketInput;
pub type FungibleResourceManagerCreateEmptyBucketManifestInput =
    FungibleResourceManagerCreateEmptyBucketInput;

pub type FungibleResourceManagerBurnInput = ResourceManagerBurnInput;
pub type FungibleResourceManagerBurnManifestInput = FungibleResourceManagerBurnInput;

pub type FungibleResourceManagerPackageBurnInput = ResourceManagerPackageBurnInput;
pub type FungibleResourceManagerPackageBurnManifestInput = FungibleResourceManagerPackageBurnInput;

pub type FungibleResourceManagerCreateEmptyVaultInput = ResourceManagerCreateEmptyVaultInput;
pub type FungibleResourceManagerCreateEmptyVaultManifestInput =
    FungibleResourceManagerCreateEmptyVaultInput;

pub type FungibleResourceManagerGetResourceTypeInput = ResourceManagerGetResourceTypeInput;
pub type FungibleResourceManagerGetResourceTypeManifestInput =
    FungibleResourceManagerGetResourceTypeInput;

pub type FungibleResourceManagerGetTotalSupplyInput = ResourceManagerGetTotalSupplyInput;
pub type FungibleResourceManagerGetTotalSupplyManifestInput =
    FungibleResourceManagerGetTotalSupplyInput;

pub type FungibleResourceManagerAmountForWithdrawalInput =
    ResourceManagerGetAmountForWithdrawalInput;
pub type FungibleResourceManagerAmountForWithdrawalManifestInput =
    FungibleResourceManagerAmountForWithdrawalInput;

pub type FungibleResourceManagerDropEmptyBucketInput = ResourceManagerDropEmptyBucketInput;
pub type FungibleResourceManagerDropEmptyBucketManifestInput =
    FungibleResourceManagerDropEmptyBucketInput;
