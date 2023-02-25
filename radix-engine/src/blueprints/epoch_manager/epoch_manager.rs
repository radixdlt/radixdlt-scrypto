use super::ValidatorCreator;
use crate::errors::RuntimeError;
use crate::errors::{ApplicationError, InterpreterError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::access_rules::MethodAccessRulesChainSubstate;
use crate::types::*;
use native_sdk::resource::{ResourceManager, SysBucket};
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::account::{AccountDepositInput, ACCOUNT_DEPOSIT_IDENT};
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;
use radix_engine_interface::rule;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct EpochManagerSubstate {
    pub address: ComponentAddress, // TODO: Does it make sense for this to be stored here?
    pub epoch: u64,
    pub round: u64,

    // TODO: Move configuration to an immutable substate
    pub rounds_per_epoch: u64,
    pub num_unstake_epochs: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, ScryptoSbor)]
pub struct Validator {
    pub key: EcdsaSecp256k1PublicKey,
    pub stake: Decimal,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct ValidatorSetSubstate {
    pub validator_set: BTreeMap<ComponentAddress, Validator>,
    pub epoch: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub enum EpochManagerError {
    InvalidRoundUpdate { from: u64, to: u64 },
}

pub struct EpochManagerBlueprint;

impl EpochManagerBlueprint {
    pub(crate) fn create<Y>(
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: EpochManagerCreateInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let underlying_node_id = api.kernel_allocate_node_id(RENodeType::EpochManager)?;
        let address = ComponentAddress::EpochManager(input.component_address);

        let epoch_manager = EpochManagerSubstate {
            address,
            epoch: input.initial_epoch,
            round: 0,
            rounds_per_epoch: input.rounds_per_epoch,
            num_unstake_epochs: input.num_unstake_epochs,
        };

        let mut olympia_validator_token_resman: ResourceManager = {
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

            let result = api.call_function(
                RESOURCE_MANAGER_PACKAGE,
                RESOURCE_MANAGER_BLUEPRINT,
                RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_ADDRESS_IDENT,
                scrypto_encode(&ResourceManagerCreateNonFungibleWithAddressInput {
                    id_type: NonFungibleIdType::Bytes,
                    metadata,
                    access_rules,
                    resource_address: input.olympia_validator_token_address,
                })
                .unwrap(),
            )?;
            let resource_address: ResourceAddress = scrypto_decode(result.as_slice()).unwrap();
            ResourceManager(resource_address)
        };

        let mut validator_set = BTreeMap::new();

        for (key, validator_init) in input.validator_set {
            let local_id = NonFungibleLocalId::bytes(key.to_vec()).unwrap();
            let global_id =
                NonFungibleGlobalId::new(olympia_validator_token_resman.0, local_id.clone());
            let owner_token_bucket =
                olympia_validator_token_resman.mint_non_fungible(local_id, api)?;
            api.call_method(
                RENodeId::GlobalComponent(validator_init.validator_account_address.into()),
                ACCOUNT_DEPOSIT_IDENT,
                scrypto_encode(&AccountDepositInput {
                    bucket: owner_token_bucket,
                })
                .unwrap(),
            )?;

            let stake = validator_init.initial_stake.sys_amount(api)?;
            let (address, lp_bucket) = ValidatorCreator::create_with_initial_stake(
                address,
                key,
                rule!(require(global_id)),
                validator_init.initial_stake,
                true,
                api,
            )?;
            let validator = Validator { key, stake };
            validator_set.insert(address, validator);

            api.call_method(
                RENodeId::GlobalComponent(validator_init.stake_account_address.into()),
                ACCOUNT_DEPOSIT_IDENT,
                scrypto_encode(&AccountDepositInput { bucket: lp_bucket }).unwrap(),
            )?;
        }

        let current_validator_set = ValidatorSetSubstate {
            epoch: input.initial_epoch,
            validator_set: validator_set.clone(),
        };

        let preparing_validator_set = ValidatorSetSubstate {
            epoch: input.initial_epoch + 1,
            validator_set,
        };

        let mut access_rules = AccessRules::new();
        access_rules.set_method_access_rule(
            MethodKey::new(
                NodeModuleId::SELF,
                EPOCH_MANAGER_NEXT_ROUND_IDENT.to_string(),
            ),
            rule!(require(AuthAddresses::validator_role())),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(
                NodeModuleId::SELF,
                EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT.to_string(),
            ),
            rule!(allow_all),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(
                NodeModuleId::SELF,
                EPOCH_MANAGER_CREATE_VALIDATOR_IDENT.to_string(),
            ),
            rule!(allow_all),
        );
        let non_fungible_local_id =
            NonFungibleLocalId::bytes(scrypto_encode(&EPOCH_MANAGER_PACKAGE).unwrap()).unwrap();
        let non_fungible_global_id = NonFungibleGlobalId::new(PACKAGE_TOKEN, non_fungible_local_id);
        access_rules.set_method_access_rule(
            MethodKey::new(
                NodeModuleId::SELF,
                EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT.to_string(),
            ),
            rule!(require(non_fungible_global_id)),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(
                NodeModuleId::SELF,
                EPOCH_MANAGER_SET_EPOCH_IDENT.to_string(),
            ),
            rule!(require(AuthAddresses::system_role())), // Set epoch only used for debugging
        );

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::ObjectAccessRulesChain(MethodAccessRulesChainSubstate {
                access_rules_chain: vec![access_rules],
            }),
        );

        api.kernel_create_node(
            underlying_node_id,
            RENodeInit::EpochManager(
                epoch_manager,
                current_validator_set,
                preparing_validator_set,
            ),
            node_modules,
        )?;

        api.globalize_with_address(underlying_node_id, AccessRules::new(), address.into())?;

        Ok(IndexedScryptoValue::from_typed(&address))
    }

    pub(crate) fn get_current_epoch<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let _input: EpochManagerGetCurrentEpochInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
            LockFlags::read_only(),
        )?;

        let epoch_manager: &EpochManagerSubstate = api.kernel_get_substate_ref(handle)?;

        Ok(IndexedScryptoValue::from_typed(&epoch_manager.epoch))
    }

    pub(crate) fn next_round<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: EpochManagerNextRoundInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let offset = SubstateOffset::EpochManager(EpochManagerOffset::EpochManager);
        let mgr_handle = api.sys_lock_substate(receiver, offset, LockFlags::MUTABLE)?;
        let epoch_manager: &mut EpochManagerSubstate =
            api.kernel_get_substate_ref_mut(mgr_handle)?;

        if input.round <= epoch_manager.round {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::EpochManagerError(EpochManagerError::InvalidRoundUpdate {
                    from: epoch_manager.round,
                    to: input.round,
                }),
            ));
        }

        if input.round >= epoch_manager.rounds_per_epoch {
            let offset = SubstateOffset::EpochManager(EpochManagerOffset::PreparingValidatorSet);
            let handle = api.sys_lock_substate(receiver, offset, LockFlags::MUTABLE)?;
            let preparing_validator_set: &mut ValidatorSetSubstate =
                api.kernel_get_substate_ref_mut(handle)?;
            let prepared_epoch = preparing_validator_set.epoch;
            let next_validator_set = preparing_validator_set.validator_set.clone();
            preparing_validator_set.epoch = prepared_epoch + 1;

            let epoch_manager: &mut EpochManagerSubstate =
                api.kernel_get_substate_ref_mut(mgr_handle)?;
            epoch_manager.epoch = prepared_epoch;
            epoch_manager.round = 0;

            let handle = api.sys_lock_substate(
                receiver,
                SubstateOffset::EpochManager(EpochManagerOffset::CurrentValidatorSet),
                LockFlags::MUTABLE,
            )?;
            let validator_set: &mut ValidatorSetSubstate =
                api.kernel_get_substate_ref_mut(handle)?;
            validator_set.epoch = prepared_epoch;
            validator_set.validator_set = next_validator_set;
        } else {
            epoch_manager.round = input.round;
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn set_epoch<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: EpochManagerSetEpochInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
            LockFlags::MUTABLE,
        )?;

        let epoch_manager: &mut EpochManagerSubstate = api.kernel_get_substate_ref_mut(handle)?;
        epoch_manager.epoch = input.epoch;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    pub(crate) fn create_validator<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: EpochManagerCreateValidatorInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::EpochManager(EpochManagerOffset::EpochManager),
            LockFlags::read_only(),
        )?;
        let epoch_manager: &EpochManagerSubstate = api.kernel_get_substate_ref(handle)?;
        let manager = epoch_manager.address;
        let validator_address =
            ValidatorCreator::create(manager, input.key, input.owner_access_rule, false, api)?;

        Ok(IndexedScryptoValue::from_typed(&validator_address))
    }

    pub(crate) fn update_validator<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: EpochManagerUpdateValidatorInput =
            scrypto_decode(&scrypto_encode(&input).unwrap())
                .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::EpochManager(EpochManagerOffset::PreparingValidatorSet),
            LockFlags::MUTABLE,
        )?;
        let validator_set: &mut ValidatorSetSubstate = api.kernel_get_substate_ref_mut(handle)?;
        match input.update {
            UpdateValidator::Register(key, stake) => {
                validator_set
                    .validator_set
                    .insert(input.validator_address, Validator { key, stake });
            }
            UpdateValidator::Unregister => {
                validator_set.validator_set.remove(&input.validator_address);
            }
        }

        Ok(IndexedScryptoValue::from_typed(&()))
    }
}
