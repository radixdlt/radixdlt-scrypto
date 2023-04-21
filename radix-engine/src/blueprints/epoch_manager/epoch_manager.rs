use super::{EpochChangeEvent, RoundChangeEvent, ValidatorCreator};
use crate::errors::ApplicationError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::types::*;
use native_sdk::account::Account;
use native_sdk::modules::access_rules::{AccessRules, AccessRulesObject, AttachedAccessRules};
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use native_sdk::resource::{ResourceManager, SysBucket};
use native_sdk::runtime::Runtime;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::substate_lock_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::rule;

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

pub type SecondaryIndexSubstate = Own;

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub enum EpochManagerError {
    InvalidRoundUpdate { from: u64, to: u64 },
}

pub struct EpochManagerBlueprint;

impl EpochManagerBlueprint {
    pub(crate) fn create<Y>(
        validator_token_address: [u8; NodeId::LENGTH], // TODO: Clean this up
        component_address: [u8; NodeId::LENGTH],       // TODO: Clean this up
        validator_set: Vec<(EcdsaSecp256k1PublicKey, ComponentAddress, Bucket)>,
        initial_epoch: u64,
        max_validators: u32,
        rounds_per_epoch: u64,
        num_unstake_epochs: u64,
        api: &mut Y,
    ) -> Result<Vec<Bucket>, RuntimeError>
    where
        Y: KernelNodeApi + ClientApi<RuntimeError>,
    {
        let address = ComponentAddress::new_unchecked(component_address);

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
        };

        let registered_validators = {
            let sorted_validators = api.new_sorted_index()?;
            Own(sorted_validators)
        };

        let epoch_manager_id = {
            let epoch_manager = EpochManagerSubstate {
                epoch: initial_epoch,
                round: 0,
                max_validators,
                rounds_per_epoch,
                num_unstake_epochs,
            };
            let current_validator_set = CurrentValidatorSetSubstate {
                validator_set: BTreeMap::new(),
            };

            api.new_object(
                EPOCH_MANAGER_BLUEPRINT,
                vec![
                    scrypto_encode(&epoch_manager).unwrap(),
                    scrypto_encode(&current_validator_set).unwrap(),
                    scrypto_encode(&registered_validators).unwrap(),
                ],
            )?
        };

        let non_fungible_local_id =
            NonFungibleLocalId::bytes(scrypto_encode(&EPOCH_MANAGER_PACKAGE).unwrap()).unwrap();
        let this_package_token = NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id);

        let mut access_rules = AccessRulesConfig::new();
        access_rules.set_method_access_rule_and_mutability(
            MethodKey::new(ObjectModuleId::SELF, EPOCH_MANAGER_START_IDENT),
            rule!(require(this_package_token.clone())),
            rule!(require(this_package_token.clone())),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(ObjectModuleId::SELF, EPOCH_MANAGER_NEXT_ROUND_IDENT),
            rule!(require(AuthAddresses::validator_role())),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(ObjectModuleId::SELF, EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT),
            rule!(allow_all),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(ObjectModuleId::SELF, EPOCH_MANAGER_CREATE_VALIDATOR_IDENT),
            rule!(allow_all),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(
                ObjectModuleId::SELF,
                EPOCH_MANAGER_CREATE_VALIDATOR_WITH_STAKE_IDENT,
            ),
            rule!(allow_all),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(ObjectModuleId::SELF, EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT),
            rule!(require(this_package_token)),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(ObjectModuleId::SELF, EPOCH_MANAGER_SET_EPOCH_IDENT),
            rule!(require(AuthAddresses::system_role())), // Set epoch only used for debugging
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
                ObjectModuleId::SELF => epoch_manager_id,
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            address.into(),
        )?;

        // Initialize validators
        let lp_buckets = {
            let mut lp_buckets = vec![];
            for (key, component_address, initial_stake) in validator_set {
                let rtn = api.call_method(
                    address.as_node_id(),
                    EPOCH_MANAGER_CREATE_VALIDATOR_WITH_STAKE_IDENT,
                    scrypto_encode(&EpochManagerCreateValidatorWithStakeInput {
                        key,
                        xrd_stake: initial_stake,
                        register: true,
                    })
                    .unwrap(),
                )?;
                let rtn: EpochManagerCreateValidatorWithStakeOutput = scrypto_decode(&rtn).unwrap();
                let (_address, lp_bucket, owner_token_bucket) = rtn;

                Account(component_address).deposit(owner_token_bucket, api)?;
                lp_buckets.push(lp_bucket);
            }
            lp_buckets
        };

        api.call_method(
            address.as_node_id(),
            EPOCH_MANAGER_START_IDENT,
            scrypto_encode(&()).unwrap(),
        )?;

        Ok(lp_buckets)
    }

    pub(crate) fn get_current_epoch<Y>(api: &mut Y) -> Result<u64, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            EpochManagerOffset::EpochManager.into(),
            LockFlags::read_only(),
        )?;

        let epoch_manager: EpochManagerSubstate = api.sys_read_substate_typed(handle)?;

        Ok(epoch_manager.epoch)
    }

    pub(crate) fn start<Y>(receiver: &NodeId, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let mgr_handle =
            api.lock_field(EpochManagerOffset::EpochManager.into(), LockFlags::MUTABLE)?;
        let epoch_manager: EpochManagerSubstate = api.sys_read_substate_typed(mgr_handle)?;
        Self::epoch_change(epoch_manager.epoch, epoch_manager.max_validators, api)?;

        let access_rules = AttachedAccessRules(*receiver);
        access_rules.set_method_access_rule_and_mutability(
            MethodKey::new(ObjectModuleId::SELF, EPOCH_MANAGER_START_IDENT),
            AccessRuleEntry::AccessRule(AccessRule::DenyAll),
            AccessRuleEntry::AccessRule(AccessRule::DenyAll),
            api,
        )?;

        Ok(())
    }

    pub(crate) fn next_round<Y>(round: u64, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let mgr_handle =
            api.lock_field(EpochManagerOffset::EpochManager.into(), LockFlags::MUTABLE)?;
        let mut epoch_manager: EpochManagerSubstate = api.sys_read_substate_typed(mgr_handle)?;

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
            let max_validators = epoch_manager.max_validators;
            Self::epoch_change(next_epoch, max_validators, api)?;
            epoch_manager.epoch = next_epoch;
            epoch_manager.round = 0;
        } else {
            Runtime::emit_event(api, RoundChangeEvent { round })?;
            epoch_manager.round = round;
        }

        api.sys_write_substate_typed(mgr_handle, &epoch_manager)?;
        api.sys_drop_lock(mgr_handle)?;

        Ok(())
    }

    pub(crate) fn set_epoch<Y>(epoch: u64, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(EpochManagerOffset::EpochManager.into(), LockFlags::MUTABLE)?;

        let mut epoch_manager: EpochManagerSubstate = api.sys_read_substate_typed(handle)?;
        epoch_manager.epoch = epoch;
        api.sys_write_substate_typed(handle, &epoch_manager)?;

        Ok(())
    }

    pub(crate) fn create_validator<Y>(
        _receiver: &NodeId,
        key: EcdsaSecp256k1PublicKey,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let (validator_address, owner_token_bucket) = ValidatorCreator::create(key, false, api)?;

        Ok((validator_address, owner_token_bucket))
    }

    pub(crate) fn create_validator_with_stake<Y>(
        key: EcdsaSecp256k1PublicKey,
        xrd_stake: Bucket,
        register: bool,
        api: &mut Y,
    ) -> Result<(ComponentAddress, Bucket, Bucket), RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let stake_amount = xrd_stake.sys_amount(api)?;

        let (validator_address, liquidity_token_bucket, owner_token_bucket, index_key) =
            ValidatorCreator::create_with_stake(key, xrd_stake, register, api)?;

        if let Some(index_key) = index_key {
            Self::update_validator(
                UpdateSecondaryIndex::Create {
                    primary: validator_address,
                    stake: stake_amount,
                    index_key,
                    key,
                },
                api,
            )?;
        }

        Ok((
            validator_address,
            liquidity_token_bucket,
            owner_token_bucket,
        ))
    }

    pub(crate) fn update_validator<Y>(
        update: UpdateSecondaryIndex,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            EpochManagerOffset::RegisteredValidatorSet.into(),
            LockFlags::read_only(),
        )?;
        let secondary_index: SecondaryIndexSubstate = api.sys_read_substate_typed(handle)?;

        match update {
            UpdateSecondaryIndex::Create {
                index_key,
                primary: address,
                key,
                stake,
            } => {
                api.insert_typed_into_sorted_index(
                    secondary_index.as_node_id(),
                    index_key,
                    (address, Validator { key, stake }),
                )?;
            }
            UpdateSecondaryIndex::UpdatePublicKey { index_key, key } => {
                let (address, mut validator) = api
                    .remove_typed_from_sorted_index::<(ComponentAddress, Validator)>(
                        secondary_index.as_node_id(),
                        &index_key,
                    )?
                    .unwrap();
                validator.key = key;
                api.insert_typed_into_sorted_index(
                    secondary_index.as_node_id(),
                    index_key,
                    (address, validator),
                )?;
            }
            UpdateSecondaryIndex::UpdateStake {
                index_key,
                new_index_key,
                new_stake_amount,
            } => {
                let (address, mut validator) = api
                    .remove_typed_from_sorted_index::<(ComponentAddress, Validator)>(
                        secondary_index.as_node_id(),
                        &index_key,
                    )?
                    .unwrap();
                validator.stake = new_stake_amount;
                api.insert_typed_into_sorted_index(
                    secondary_index.as_node_id(),
                    new_index_key,
                    (address, validator),
                )?;
            }
            UpdateSecondaryIndex::Remove { index_key } => {
                api.remove_from_sorted_index(secondary_index.as_node_id(), &index_key)?;
            }
        }

        Ok(())
    }

    fn epoch_change<Y>(epoch: u64, max_validators: u32, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let handle = api.lock_field(
            EpochManagerOffset::RegisteredValidatorSet.into(),
            LockFlags::MUTABLE,
        )?;

        let secondary_index: SecondaryIndexSubstate =
            api.sys_read_substate_typed(handle)?;

        let validators: Vec<(ComponentAddress, Validator)> =
            api.scap_typed_sorted_index(secondary_index.as_node_id(), max_validators)?;
        let next_validator_set: BTreeMap<ComponentAddress, Validator> =
            validators.into_iter().collect();

        let handle = api.lock_field(
            EpochManagerOffset::CurrentValidatorSet.into(),
            LockFlags::MUTABLE,
        )?;
        let mut validator_set: CurrentValidatorSetSubstate = api.sys_read_substate_typed(handle)?;
        validator_set.validator_set = next_validator_set.clone();
        api.sys_write_substate_typed(handle, &validator_set)?;
        api.sys_drop_lock(handle)?;

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
