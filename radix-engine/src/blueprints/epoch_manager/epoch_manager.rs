use super::{EpochChangeEvent, RoundChangeEvent, ValidatorCreator};
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelNodeApi;
use crate::types::*;
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::ResourceManager;
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::{ClientApi, CollectionIndex, OBJECT_HANDLE_SELF};
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::blueprints::resource::AccessRule::DenyAll;
use radix_engine_interface::rule;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct EpochManagerConfigSubstate {
    pub max_validators: u32,
    pub rounds_per_epoch: u64,
    pub num_unstake_epochs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct EpochManagerSubstate {
    pub epoch: u64,
    pub round: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, ScryptoSbor)]
pub struct Validator {
    pub key: EcdsaSecp256k1PublicKey,
    pub stake: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct CurrentValidatorSetSubstate {
    pub validator_set: BTreeMap<ComponentAddress, Validator>,
}

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub enum EpochManagerError {
    InvalidRoundUpdate { from: u64, to: u64 },
}

pub const EPOCH_MANAGER_SECONDARY_INDEX: CollectionIndex = 0u8;

pub struct EpochManagerBlueprint;

impl EpochManagerBlueprint {
    pub(crate) fn create<Y>(
        validator_token_address: [u8; NodeId::LENGTH], // TODO: Clean this up
        component_address: [u8; NodeId::LENGTH],       // TODO: Clean this up
        initial_epoch: u64,
        max_validators: u32,
        rounds_per_epoch: u64,
        num_unstake_epochs: u64,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let address = ComponentAddress::new_or_panic(component_address);

        {
            let metadata: BTreeMap<String, String> = BTreeMap::new();
            let mut access_rules = BTreeMap::new();

            // TODO: remove mint and premint all tokens
            {
                let global_id =
                    NonFungibleGlobalId::package_of_direct_caller_badge(EPOCH_MANAGER_PACKAGE);
                access_rules.insert(Mint, (rule!(require(global_id)), rule!(deny_all)));
            }

            access_rules.insert(Withdraw, (rule!(allow_all), rule!(deny_all)));

            ResourceManager::new_non_fungible_with_address::<(), Y, RuntimeError>(
                NonFungibleIdType::UUID,
                metadata,
                access_rules,
                validator_token_address,
                api,
            )?;
        };

        let epoch_manager_id = {
            let config = EpochManagerConfigSubstate {
                max_validators,
                rounds_per_epoch,
                num_unstake_epochs,
            };
            let epoch_manager = EpochManagerSubstate {
                epoch: initial_epoch,
                round: 0,
            };
            let current_validator_set = CurrentValidatorSetSubstate {
                validator_set: BTreeMap::new(),
            };

            api.new_simple_object(
                EPOCH_MANAGER_BLUEPRINT,
                vec![
                    scrypto_encode(&config).unwrap(),
                    scrypto_encode(&epoch_manager).unwrap(),
                    scrypto_encode(&current_validator_set).unwrap(),
                ],
            )?
        };

        let mut access_rules = AccessRulesConfig::new();
        access_rules.set_group_access_rule_and_mutability(
            "start",
            rule!(require(package_of_direct_caller(EPOCH_MANAGER_PACKAGE))),
            rule!(require(package_of_direct_caller(EPOCH_MANAGER_PACKAGE))),
        );
        access_rules.set_group_access_rule_and_mutability(
            "validator",
            rule!(require(AuthAddresses::validator_role())),
            DenyAll
        );
        access_rules.set_group_access_rule_and_mutability(
            "system",
            rule!(require(AuthAddresses::system_role())), // Set epoch only used for debugging
            DenyAll
        );

        access_rules.set_group(
            MethodKey::new(ObjectModuleId::Main, EPOCH_MANAGER_START_IDENT),
            "start",
        );
        access_rules.set_group(
            MethodKey::new(ObjectModuleId::Main, EPOCH_MANAGER_NEXT_ROUND_IDENT),
            "validator",
        );
        access_rules.set_group(
            MethodKey::new(ObjectModuleId::Main, EPOCH_MANAGER_SET_EPOCH_IDENT),
            "system",
        );
        access_rules.set_public(
            MethodKey::new(ObjectModuleId::Main, EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT),
        );
        access_rules.set_public(
            MethodKey::new(ObjectModuleId::Main, EPOCH_MANAGER_CREATE_VALIDATOR_IDENT),
        );

        let validator_access_rules =
            AccessRulesConfig::new().default(AccessRule::AllowAll, AccessRule::DenyAll);

        let access_rules = AccessRules::sys_new(
            access_rules,
            btreemap!(
                VALIDATOR_BLUEPRINT.to_string() => validator_access_rules
            ),
            api,
        )?
        .0;
        let metadata = Metadata::sys_create(api)?;
        let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

        api.globalize_with_address(
            btreemap!(
                ObjectModuleId::Main => epoch_manager_id,
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            address.into(),
        )?;

        Ok(())
    }

    pub(crate) fn get_current_epoch<Y>(api: &mut Y) -> Result<u64, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            EpochManagerField::EpochManager.into(),
            LockFlags::read_only(),
        )?;

        let epoch_manager: EpochManagerSubstate = api.field_lock_read_typed(handle)?;

        Ok(epoch_manager.epoch)
    }

    pub(crate) fn start<Y>(receiver: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let config_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            EpochManagerField::Config.into(),
            LockFlags::read_only(),
        )?;
        let config: EpochManagerConfigSubstate = api.field_lock_read_typed(config_handle)?;

        let mgr_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            EpochManagerField::EpochManager.into(),
            LockFlags::read_only(),
        )?;
        let mgr: EpochManagerSubstate = api.field_lock_read_typed(mgr_handle)?;

        Self::epoch_change(mgr.epoch, config.max_validators, api)?;

        let access_rules = AttachedAccessRules(*receiver);
        access_rules.set_group_access_rule_and_mutability(
            "start",
            AccessRule::DenyAll,
            AccessRule::DenyAll,
            api,
        )?;

        Ok(())
    }

    pub(crate) fn next_round<Y>(round: u64, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let config_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            EpochManagerField::Config.into(),
            LockFlags::read_only(),
        )?;
        let config: EpochManagerConfigSubstate = api.field_lock_read_typed(config_handle)?;
        let mgr_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            EpochManagerField::EpochManager.into(),
            LockFlags::MUTABLE,
        )?;
        let mut epoch_manager: EpochManagerSubstate = api.field_lock_read_typed(mgr_handle)?;

        if round <= epoch_manager.round {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::EpochManagerError(EpochManagerError::InvalidRoundUpdate {
                    from: epoch_manager.round,
                    to: round,
                }),
            ));
        }

        if round >= config.rounds_per_epoch {
            let next_epoch = epoch_manager.epoch + 1;
            let max_validators = config.max_validators;
            Self::epoch_change(next_epoch, max_validators, api)?;
            epoch_manager.epoch = next_epoch;
            epoch_manager.round = 0;
        } else {
            Runtime::emit_event(api, RoundChangeEvent { round })?;
            epoch_manager.round = round;
        }

        api.field_lock_write_typed(mgr_handle, &epoch_manager)?;
        api.field_lock_release(mgr_handle)?;

        Ok(())
    }

    pub(crate) fn set_epoch<Y>(epoch: u64, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            EpochManagerField::EpochManager.into(),
            LockFlags::MUTABLE,
        )?;

        let mut epoch_manager: EpochManagerSubstate = api.field_lock_read_typed(handle)?;
        epoch_manager.epoch = epoch;
        api.field_lock_write_typed(handle, &epoch_manager)?;

        Ok(())
    }

    pub(crate) fn create_validator<Y>(
        key: EcdsaSecp256k1PublicKey,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let (validator_address, owner_token_bucket) = ValidatorCreator::create(key, false, api)?;

        Ok((validator_address, owner_token_bucket))
    }

    fn epoch_change<Y>(epoch: u64, max_validators: u32, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let validators: Vec<(ComponentAddress, Validator)> = api.actor_sorted_index_scan_typed(
            OBJECT_HANDLE_SELF,
            EPOCH_MANAGER_SECONDARY_INDEX,
            max_validators,
        )?;
        let next_validator_set: BTreeMap<ComponentAddress, Validator> =
            validators.into_iter().collect();

        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            EpochManagerField::CurrentValidatorSet.into(),
            LockFlags::MUTABLE,
        )?;
        let mut validator_set: CurrentValidatorSetSubstate = api.field_lock_read_typed(handle)?;
        validator_set.validator_set = next_validator_set.clone();
        api.field_lock_write_typed(handle, &validator_set)?;
        api.field_lock_release(handle)?;

        Runtime::emit_event(
            api,
            EpochChangeEvent {
                epoch,
                validators: next_validator_set,
            },
        )?;

        Ok(())
    }
}
