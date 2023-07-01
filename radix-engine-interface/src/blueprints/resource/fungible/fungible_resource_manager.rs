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
pub struct FungibleResourceFeatures {
    pub mintable: Option<MintableRoles<RoleDefinition>>,
    pub burnable: Option<BurnableRoles<RoleDefinition>>,
    pub freezable: Option<FreezableRoles<RoleDefinition>>,
    pub recallable: Option<RecallableRoles<RoleDefinition>>,
    pub restrict_withdraw: Option<WithdrawableRoles<RoleDefinition>>,
    pub restrict_deposit: Option<DepositableRoles<RoleDefinition>>,
}

impl FungibleResourceFeatures {
    pub fn single_locked_rule(access_rule: AccessRule) -> Self {
        Self {
            mintable: mintable! {
                minter => access_rule.clone(), locked;
                minter_updater => rule!(deny_all), locked;
            },
            burnable: burnable! {
                burner => access_rule.clone(), locked;
                burner_updater => rule!(deny_all), locked;
            },
            freezable: freezable! {
                freezer => access_rule.clone(), locked;
                freezer_updater => rule!(deny_all), locked;
            },
            recallable: recallable! {
                recaller => access_rule.clone(), locked;
                recaller_updater => rule!(deny_all), locked;
            },
            restrict_withdraw: restrict_withdraw! {
                withdrawer => access_rule.clone(), locked;
                withdrawer_updater => rule!(deny_all), locked;
            },
            restrict_deposit: restrict_deposit! {
                depositor => access_rule.clone(), locked;
                depositor_updater => rule!(deny_all), locked;
            },
        }
    }

    pub fn to_features_and_roles(self) -> (Vec<&'static str>, RolesInit) {
        let mut features = Vec::new();
        let mut roles = RolesInit::new();

        if let Some(mintable) = &self.mintable {
            if mintable
                .minter
                .ne(&RoleDefinition::locked(AccessRule::DenyAll))
            {
                features.push(MINT_FEATURE);
            }
        }

        if let Some(burnable) = &self.burnable {
            if burnable
                .burner
                .ne(&RoleDefinition::locked(AccessRule::DenyAll))
            {
                features.push(BURN_FEATURE);
            }
        }

        if let Some(freezable) = &self.freezable {
            if freezable
                .freezer
                .ne(&RoleDefinition::locked(AccessRule::DenyAll))
            {
                features.push(VAULT_FREEZE_FEATURE);
            }
        }

        if let Some(recallable) = &self.recallable {
            if recallable
                .recaller
                .ne(&RoleDefinition::locked(AccessRule::DenyAll))
            {
                features.push(VAULT_RECALL_FEATURE);
            }
        }

        roles
            .data
            .extend(self.mintable.unwrap_or_default().to_role_init().data);
        roles
            .data
            .extend(self.burnable.unwrap_or_default().to_role_init().data);
        roles
            .data
            .extend(self.recallable.unwrap_or_default().to_role_init().data);
        roles
            .data
            .extend(self.freezable.unwrap_or_default().to_role_init().data);
        roles.data.extend(
            self.restrict_deposit
                .unwrap_or_default()
                .to_role_init()
                .data,
        );
        roles.data.extend(
            self.restrict_withdraw
                .unwrap_or_default()
                .to_role_init()
                .data,
        );

        (features, roles)
    }
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor)]
pub struct FungibleResourceManagerCreateInput {
    pub owner_role: OwnerRole,
    pub track_total_supply: bool,
    pub divisibility: u8,
    pub resource_features: FungibleResourceFeatures,
    pub metadata: ModuleConfig<MetadataInit>,
    pub address_reservation: Option<GlobalAddressReservation>,
}

#[cfg_attr(feature = "radix_engine_fuzzing", derive(Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor)]
pub struct FungibleResourceManagerCreateManifestInput {
    pub owner_role: OwnerRole,
    pub track_total_supply: bool,
    pub divisibility: u8,
    pub resource_features: FungibleResourceFeatures,
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
    pub resource_features: FungibleResourceFeatures,
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
    pub resource_features: FungibleResourceFeatures,
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
