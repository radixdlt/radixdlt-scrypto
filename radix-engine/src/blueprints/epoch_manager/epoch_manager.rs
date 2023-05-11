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
use radix_engine_interface::blueprints::resource::AccessRule::DenyAll;
use radix_engine_interface::blueprints::resource::*;
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

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct CurrentProposalStatisticSubstate {
    /// A proposal statistic of each validator from the current validator set, in the iteration
    /// order of [`CurrentValidatorSetSubstate.validator_set`].
    pub validator_statistics: Vec<ProposalStatistic>,
}

impl CurrentProposalStatisticSubstate {
    /// Gets a mutable reference to a proposal statistic tracker of an individual validator.
    pub fn get_mut_proposal_statistic(
        &mut self,
        validator_index: ValidatorIndex,
    ) -> Result<&mut ProposalStatistic, RuntimeError> {
        let validator_count = self.validator_statistics.len();
        self.validator_statistics
            .get_mut(validator_index as usize)
            .ok_or_else(|| {
                RuntimeError::ApplicationError(ApplicationError::EpochManagerError(
                    EpochManagerError::InvalidaValidatorIndex {
                        index: validator_index as usize,
                        count: validator_count,
                    },
                ))
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, ScryptoSbor)]
pub struct ProposalStatistic {
    /// A counter of successful proposals made by a specific validator.
    pub made: u64,
    /// A counter of missed proposals (caused both by gap rounds or fallback rounds).
    pub missed: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Sbor)]
pub enum EpochManagerError {
    InvalidRoundUpdate {
        from: u64,
        to: u64,
    },
    InconsistentGapRounds {
        gap_rounds: u64,
        progressed_rounds: u64,
    },
    InvalidaValidatorIndex {
        index: usize,
        count: usize,
    },
}

pub const EPOCH_MANAGER_REGISTERED_VALIDATORS_BY_STAKE_INDEX: CollectionIndex = 0u8;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct EpochRegisteredValidatorByStakeEntry {
    component_address: ComponentAddress,
    validator: Validator,
}

pub struct EpochManagerBlueprint;

impl EpochManagerBlueprint {
    pub(crate) fn create<Y>(
        validator_token_address: [u8; NodeId::LENGTH], // TODO: Clean this up
        component_address: [u8; NodeId::LENGTH],       // TODO: Clean this up
        initial_epoch: u64,
        initial_configuration: EpochManagerInitialConfiguration,
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
                max_validators: initial_configuration.max_validators,
                rounds_per_epoch: initial_configuration.rounds_per_epoch,
                num_unstake_epochs: initial_configuration.num_unstake_epochs,
            };
            let epoch_manager = EpochManagerSubstate {
                epoch: initial_epoch,
                round: 0,
            };
            let current_validator_set = CurrentValidatorSetSubstate {
                validator_set: BTreeMap::new(),
            };
            let current_proposal_statistic = CurrentProposalStatisticSubstate {
                validator_statistics: Vec::new(),
            };

            api.new_simple_object(
                EPOCH_MANAGER_BLUEPRINT,
                vec![
                    scrypto_encode(&config).unwrap(),
                    scrypto_encode(&epoch_manager).unwrap(),
                    scrypto_encode(&current_validator_set).unwrap(),
                    scrypto_encode(&current_proposal_statistic).unwrap(),
                ],
            )?
        };

        let mut access_rules = AccessRulesConfig::new();
        access_rules.set_authority(
            "start",
            rule!(require(package_of_direct_caller(EPOCH_MANAGER_PACKAGE))),
            rule!(require(package_of_direct_caller(EPOCH_MANAGER_PACKAGE))),
        );
        access_rules.set_authority(
            "validator",
            rule!(require(AuthAddresses::validator_role())),
            DenyAll,
        );
        access_rules.set_authority(
            "system",
            rule!(require(AuthAddresses::system_role())), // Set epoch only used for debugging
            DenyAll,
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
        access_rules.set_public(MethodKey::new(
            ObjectModuleId::Main,
            EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT,
        ));
        access_rules.set_public(MethodKey::new(
            ObjectModuleId::Main,
            EPOCH_MANAGER_CREATE_VALIDATOR_IDENT,
        ));

        let validator_access_rules = AccessRulesConfig::new();

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

    pub(crate) fn next_round<Y>(
        round: u64,
        proposal_history: LeaderProposalHistory,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
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

        let progressed_rounds = round as i128 - epoch_manager.round as i128;
        if progressed_rounds <= 0 {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::EpochManagerError(EpochManagerError::InvalidRoundUpdate {
                    from: epoch_manager.round,
                    to: round,
                }),
            ));
        }

        Self::update_proposal_statistics(progressed_rounds as u64, proposal_history, api)?;

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

    fn update_proposal_statistics<Y>(
        progressed_rounds: u64,
        proposal_history: LeaderProposalHistory,
        api: &mut Y,
    ) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        if proposal_history.gap_round_leaders.len() as u64 != progressed_rounds - 1 {
            return Err(RuntimeError::ApplicationError(
                ApplicationError::EpochManagerError(EpochManagerError::InconsistentGapRounds {
                    gap_rounds: proposal_history.gap_round_leaders.len() as u64,
                    progressed_rounds,
                }),
            ));
        }

        let statistic_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            EpochManagerField::CurrentProposalStatistic.into(),
            LockFlags::MUTABLE,
        )?;
        let mut statistic: CurrentProposalStatisticSubstate =
            api.field_lock_read_typed(statistic_handle)?;
        for gap_round_leader in proposal_history.gap_round_leaders {
            let mut gap_round_statistic = statistic.get_mut_proposal_statistic(gap_round_leader)?;
            gap_round_statistic.missed += 1;
        }
        let mut current_round_statistic =
            statistic.get_mut_proposal_statistic(proposal_history.current_leader)?;
        if proposal_history.is_fallback {
            current_round_statistic.missed += 1;
        } else {
            current_round_statistic.made += 1;
        }
        api.field_lock_write_typed(statistic_handle, statistic)?;
        api.field_lock_release(statistic_handle)?;

        Ok(())
    }

    fn epoch_change<Y>(epoch: u64, max_validators: u32, api: &mut Y) -> Result<(), RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let validators: Vec<EpochRegisteredValidatorByStakeEntry> = api
            .actor_sorted_index_scan_typed(
                OBJECT_HANDLE_SELF,
                EPOCH_MANAGER_REGISTERED_VALIDATORS_BY_STAKE_INDEX,
                max_validators,
            )?;
        let next_validator_set: BTreeMap<ComponentAddress, Validator> = validators
            .into_iter()
            .map(|entry| (entry.component_address, entry.validator))
            .collect();

        let validator_set_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            EpochManagerField::CurrentValidatorSet.into(),
            LockFlags::MUTABLE,
        )?;
        api.field_lock_write_typed(
            validator_set_handle,
            CurrentValidatorSetSubstate {
                validator_set: next_validator_set.clone(),
            },
        )?;
        api.field_lock_release(validator_set_handle)?;

        let statistic_handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            EpochManagerField::CurrentProposalStatistic.into(),
            LockFlags::MUTABLE,
        )?;
        let mut statistic: CurrentProposalStatisticSubstate =
            api.field_lock_read_typed(statistic_handle)?;
        // TODO(emissions): In some next "emissions" PR, capture the concluded epoch's validator
        // statistics (to be used for unreliability penalty calculation); at the moment we only
        // reset it.
        statistic.validator_statistics = (0..next_validator_set.len())
            .map(|_index| ProposalStatistic::default())
            .collect();
        api.field_lock_write_typed(statistic_handle, statistic)?;
        api.field_lock_release(statistic_handle)?;

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
