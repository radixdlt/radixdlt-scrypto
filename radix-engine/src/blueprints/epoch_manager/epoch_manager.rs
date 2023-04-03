use super::{EpochChangeEvent, RoundChangeEvent, ValidatorCreator};
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use native_sdk::account::Account;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::{ResourceManager, SysBucket};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::rule;
use radix_engine_interface::schema::{IterableMapSchema, KeyValueStoreSchema};

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct EpochManagerSubstate {
    pub epoch: u64,
    pub round: u64,

    // TODO: Move configuration to an immutable substate
    pub max_validators: u32,
    pub rounds_per_epoch: u64,
    pub num_unstake_epochs: u64,
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct RegisteredValidatorsSubstate {
    pub validators: Own,
    pub index: Own,
}

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub enum EpochManagerError {
    InvalidRoundUpdate { from: u64, to: u64 },
}

pub struct EpochManagerBlueprint;

impl EpochManagerBlueprint {
    fn to_index_key(stake: Decimal, address: ComponentAddress) -> Vec<u8> {
        let reverse_stake = Decimal::MAX - stake;
        let mut index_key = reverse_stake.to_be_bytes();
        index_key.extend(scrypto_encode(&address).unwrap());
        index_key
    }

    pub(crate) fn create<Y>(
        validator_token_address: [u8; 26], // TODO: Clean this up
        component_address: [u8; 26],       // TODO: Clean this up
        validator_set: BTreeMap<EcdsaSecp256k1PublicKey, ValidatorInit>,
        initial_epoch: u64,
        max_validators: u32,
        rounds_per_epoch: u64,
        num_unstake_epochs: u64,
        api: &mut Y,
    ) -> Result<ComponentAddress, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let address = ComponentAddress::EpochManager(component_address);

        {
            let metadata: BTreeMap<String, String> = BTreeMap::new();
            let mut access_rules = BTreeMap::new();

            // TODO: remove mint and premint all tokens
            {
                let non_fungible_local_id =
                    NonFungibleLocalId::bytes(scrypto_encode(&EPOCH_MANAGER_PACKAGE).unwrap())
                        .unwrap();
                let global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id);
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
        }

        let mut validators = BTreeMap::new();

        let registered_validators = api.new_key_value_store(KeyValueStoreSchema::new::<ComponentAddress, Validator>(false))?;

        let index_id = api.new_iterable_map(IterableMapSchema::new::<(ComponentAddress, Validator)>())?;

        for (key, validator_init) in validator_set {
            let stake = validator_init.initial_stake.sys_amount(api)?;
            let (address, lp_bucket, owner_token_bucket) =
                ValidatorCreator::create_with_initial_stake(
                    address,
                    key,
                    validator_init.initial_stake,
                    true,
                    api,
                )?;

            let validator = Validator { key, stake };
            validators.insert(address, validator);

            {
                let lock_handle = api.sys_lock_substate(
                    RENodeId::KeyValueStore(registered_validators),
                    SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(scrypto_encode(&address).unwrap())),
                    LockFlags::MUTABLE,
                )?;
                let validator = Validator { key, stake };
                api.sys_write_typed_substate(lock_handle, Some(validator))?;
                api.sys_drop_lock(lock_handle)?;
            }

            {
                let index_key = Self::to_index_key(stake, address);
                let entry_value = (address, Validator {
                    key,
                    stake,
                });
                api.insert_into_iterable_map(
                    RENodeId::KeyValueStore(index_id),
                    index_key,
                    scrypto_encode(&entry_value).unwrap(),
                )?;
            }

            Account(validator_init.validator_account_address).deposit(owner_token_bucket, api)?;
            Account(validator_init.stake_account_address).deposit(lp_bucket, api)?;
        }

        let current_validator_set = {
            let next_validator_set: Vec<(ComponentAddress, Validator)> = api.first_typed_in_iterable_map(
                RENodeId::KeyValueStore(index_id),
                max_validators,
            )?;
            let next_validator_set: BTreeMap<ComponentAddress, Validator> = next_validator_set.into_iter().collect();
            Runtime::emit_event(
                api,
                EpochChangeEvent {
                    epoch: initial_epoch,
                    validators: next_validator_set.clone(),
                },
            )?;

            CurrentValidatorSetSubstate {
                validator_set: next_validator_set,
            }
        };

        let epoch_manager_id = {
            let epoch_manager = EpochManagerSubstate {
                epoch: initial_epoch,
                round: 0,
                max_validators,
                rounds_per_epoch,
                num_unstake_epochs,
            };

            let preparing_validator_set = RegisteredValidatorsSubstate {
                validators: Own::KeyValueStore(registered_validators),
                index: Own::KeyValueStore(index_id),
            };

            api.new_object(
                EPOCH_MANAGER_BLUEPRINT,
                vec![
                    scrypto_encode(&epoch_manager).unwrap(),
                    scrypto_encode(&current_validator_set).unwrap(),
                    scrypto_encode(&preparing_validator_set).unwrap(),
                ],
            )?
        };

        let mut access_rules = AccessRulesConfig::new();
        access_rules.set_method_access_rule(
            MethodKey::new(NodeModuleId::SELF, EPOCH_MANAGER_NEXT_ROUND_IDENT),
            rule!(require(AuthAddresses::validator_role())),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(NodeModuleId::SELF, EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT),
            rule!(allow_all),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(NodeModuleId::SELF, EPOCH_MANAGER_CREATE_VALIDATOR_IDENT),
            rule!(allow_all),
        );
        let non_fungible_local_id =
            NonFungibleLocalId::bytes(scrypto_encode(&EPOCH_MANAGER_PACKAGE).unwrap()).unwrap();
        let non_fungible_global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id);
        access_rules.set_method_access_rule(
            MethodKey::new(NodeModuleId::SELF, EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT),
            rule!(require(non_fungible_global_id)),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(NodeModuleId::SELF, EPOCH_MANAGER_SET_EPOCH_IDENT),
            rule!(require(AuthAddresses::system_role())), // Set epoch only used for debugging
        );

        let access_rules = AccessRules::sys_new(access_rules, btreemap!(), api)?.0;
        let metadata = Metadata::sys_create(api)?;
        let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

        api.globalize_with_address(
            RENodeId::Object(epoch_manager_id),
            btreemap!(
                NodeModuleId::AccessRules => access_rules.id(),
                NodeModuleId::Metadata => metadata.id(),
                NodeModuleId::ComponentRoyalty => royalty.id(),
            ),
            address.into(),
        )?;

        Ok(address)
    }

    pub(crate) fn get_current_epoch<Y>(
        receiver: &RENodeId,
        api: &mut Y,
    ) -> Result<u64, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
            LockFlags::read_only(),
        )?;

        let epoch_manager: &EpochManagerSubstate = api.kernel_get_substate_ref(handle)?;

        Ok(epoch_manager.epoch)
    }

    pub(crate) fn next_round<Y>(
        receiver: &RENodeId,
        round: u64,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let mgr_handle = api.sys_lock_substate(receiver.clone(), offset, LockFlags::MUTABLE)?;
        let epoch_manager: &mut EpochManagerSubstate =
            api.kernel_get_substate_ref_mut(mgr_handle)?;

        let max_validators = epoch_manager.max_validators;

        if round <= epoch_manager.round {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::EpochManagerError(EpochManagerError::InvalidRoundUpdate {
                    from: epoch_manager.round,
                    to: round,
                }),
            ));
        }

        if round >= epoch_manager.rounds_per_epoch {
            let next_epoch = epoch_manager.epoch + 1;

            let offset = SubstateOffset::EpochManager(EpochManagerOffset::RegisteredValidators);
            let handle = api.sys_lock_substate(receiver.clone(), offset, LockFlags::read_only())?;
            let validators: &RegisteredValidatorsSubstate = api.kernel_get_substate_ref(handle)?;
            let index_id = validators.index.id();

            let next_validator_set: Vec<(ComponentAddress, Validator)> = api.first_typed_in_iterable_map(
                RENodeId::KeyValueStore(index_id),
                max_validators,
            )?;
            let next_validator_set: BTreeMap<ComponentAddress, Validator> = next_validator_set.into_iter().collect();

            let epoch_manager: &mut EpochManagerSubstate =
                api.kernel_get_substate_ref_mut(mgr_handle)?;
            epoch_manager.epoch = next_epoch;
            epoch_manager.round = 0;

            let handle = api.sys_lock_substate(
                receiver.clone(),
                SubstateOffset::EpochManager(EpochManagerOffset::CurrentValidatorSet),
                LockFlags::MUTABLE,
            )?;
            let validator_set: &mut CurrentValidatorSetSubstate =
                api.kernel_get_substate_ref_mut(handle)?;
            validator_set.validator_set = next_validator_set.clone();

            Runtime::emit_event(
                api,
                EpochChangeEvent {
                    epoch: next_epoch,
                    validators: next_validator_set,
                },
            )?;
        } else {
            epoch_manager.round = round;

            Runtime::emit_event(api, RoundChangeEvent { round })?;
        }

        Ok(())
    }

    pub(crate) fn set_epoch<Y>(
        receiver: &RENodeId,
        epoch: u64,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
            LockFlags::MUTABLE,
        )?;

        let epoch_manager: &mut EpochManagerSubstate = api.kernel_get_substate_ref_mut(handle)?;
        epoch_manager.epoch = epoch;

        Ok(())
    }

    pub(crate) fn create_validator<Y>(
        _receiver: &RENodeId,
        key: EcdsaSecp256k1PublicKey,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let manager_address: ComponentAddress = api.get_global_address()?.into();

        // TODO: Validator as a child blueprint of EpochManager
        let (validator_address, owner_token_bucket) =
            ValidatorCreator::create(manager_address, key, false, api)?;

        Ok((validator_address, owner_token_bucket))
    }

    pub(crate) fn update_validator<Y>(
        receiver: &RENodeId,
        validator_address: ComponentAddress,
        update: UpdateValidator,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let handle = api.sys_lock_substate(
            receiver.clone(),
            SubstateOffset::EpochManager(EpochManagerOffset::RegisteredValidators),
            LockFlags::MUTABLE,
        )?;
        let registered: &mut RegisteredValidatorsSubstate = api.kernel_get_substate_ref_mut(handle)?;
        let kv_id = registered.validators.id();
        let index_node = RENodeId::KeyValueStore(registered.index.id());

        let lock_handle = api.sys_lock_substate(
            RENodeId::KeyValueStore(kv_id),
            SubstateOffset::KeyValueStore(KeyValueStoreOffset::Entry(scrypto_encode(&validator_address).unwrap())),
            LockFlags::MUTABLE,
        )?;
        let previous: Option<Validator> = api.sys_read_typed_substate(lock_handle)?;

        match update {
            UpdateValidator::Register(key, stake) => {
                api.sys_write_typed_substate(lock_handle, Some(Validator {
                    key, stake,
                }))?;
            }
            UpdateValidator::Unregister => {
                api.sys_write_typed_substate(lock_handle, Option::<Validator>::None)?;
            }
        }

        if let Some(previous) = previous {
            let index_key = Self::to_index_key(previous.stake, validator_address);
            api.remove_from_iterable_map(
                index_node,
                index_key,
            );
        }
        match update {
            UpdateValidator::Register(key, stake) => {
                let index_key = Self::to_index_key(stake, validator_address);
                let entry_value = (validator_address, Validator {
                    key, stake,
                });

                api.insert_into_iterable_map(
                    index_node,
                    index_key,
                    scrypto_encode(&entry_value).unwrap(),
                )?;
            }
            UpdateValidator::Unregister => {}
        }

        Ok(())
    }
}
