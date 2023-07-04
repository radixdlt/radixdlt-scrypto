use crate::blueprints::resource::*;
use crate::*;
#[cfg(feature = "radix_engine_fuzzing")]
use arbitrary::Arbitrary;
use radix_engine_common::data::manifest::model::ManifestAddressReservation;
use radix_engine_common::types::*;
use radix_engine_interface::api::node_modules::auth::ToRoleEntry;
use radix_engine_interface::api::node_modules::metadata::MetadataInit;
use radix_engine_interface::api::node_modules::ModuleConfig;

pub const FUNGIBLE_RESOURCE_MANAGER_BLUEPRINT: &str = "FungibleResourceManager";

pub const FUNGIBLE_RESOURCE_MANAGER_CREATE_IDENT: &str = "create";

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
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
    pub fn single_locked_rule(access_rule: AccessRule) -> Self {
        Self {
            mint_roles: mint_roles! {
                minter => access_rule.clone(), locked;
                minter_updater => rule!(deny_all), locked;
            },
            burn_roles: burn_roles! {
                burner => access_rule.clone(), locked;
                burner_updater => rule!(deny_all), locked;
            },
            freeze_roles: freeze_roles! {
                freezer => access_rule.clone(), locked;
                freezer_updater => rule!(deny_all), locked;
            },
            recall_roles: recall_roles! {
                recaller => access_rule.clone(), locked;
                recaller_updater => rule!(deny_all), locked;
            },
            withdraw_roles: withdraw_roles! {
                withdrawer => access_rule.clone(), locked;
                withdrawer_updater => rule!(deny_all), locked;
            },
            deposit_roles: deposit_roles! {
                depositor => access_rule.clone(), locked;
                depositor_updater => rule!(deny_all), locked;
            },
        }
    }

    pub fn to_features_and_roles(self) -> (Vec<&'static str>, RolesInit) {
        let mut features = Vec::new();
        let mut roles = RolesInit::new();

        if self.mint_roles.is_some() {
            features.push(MINT_FEATURE);
        }

        if self.burn_roles.is_some() {
            features.push(BURN_FEATURE);
        }

        if self.freeze_roles.is_some() {
            features.push(VAULT_FREEZE_FEATURE);
        }

        if self.recall_roles.is_some() {
            features.push(VAULT_RECALL_FEATURE);
        }

        roles
            .data
            .extend(self.mint_roles.unwrap_or_default().to_role_init().data);
        roles
            .data
            .extend(self.burn_roles.unwrap_or_default().to_role_init().data);
        roles
            .data
            .extend(self.recall_roles.unwrap_or_default().to_role_init().data);
        roles
            .data
            .extend(self.freeze_roles.unwrap_or_default().to_role_init().data);
        roles
            .data
            .extend(self.deposit_roles.unwrap_or_default().to_role_init().data);
        roles
            .data
            .extend(self.withdraw_roles.unwrap_or_default().to_role_init().data);

        (features, roles)
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleResourceManagerCreateInput {
    pub owner_role: OwnerRole,
    pub track_total_supply: bool,
    pub divisibility: u8,
    pub resource_roles: FungibleResourceRoles,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<GlobalAddressReservation>,
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
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

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
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

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
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

pub type FungibleResourceManagerMintOutput = Bucket;
