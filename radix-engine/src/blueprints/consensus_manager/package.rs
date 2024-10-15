use crate::blueprints::consensus_manager::{ConsensusManagerBlueprint, ValidatorBlueprint};
use crate::errors::{ApplicationError, RuntimeError};
use crate::internal_prelude::*;
use radix_engine_interface::api::SystemApi;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::package::PackageDefinition;

pub const VALIDATOR_ROLE: &str = "validator";

pub struct ConsensusManagerNativePackage;

impl ConsensusManagerNativePackage {
    pub fn definition() -> PackageDefinition {
        let blueprints = indexmap!(
            CONSENSUS_MANAGER_BLUEPRINT.to_string() => ConsensusManagerBlueprint::definition(),
            VALIDATOR_BLUEPRINT.to_string() => ValidatorBlueprint::definition(),
        );

        PackageDefinition { blueprints }
    }

    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            CONSENSUS_MANAGER_CREATE_IDENT => {
                let input: ConsensusManagerCreateInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ConsensusManagerBlueprint::create(
                    input.validator_owner_token_address,
                    input.component_address,
                    input.initial_epoch,
                    input.initial_config,
                    input.initial_time_ms,
                    input.initial_current_leader,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT => {
                let _input: ConsensusManagerGetCurrentEpochInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;

                let rtn = ConsensusManagerBlueprint::get_current_epoch(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_START_IDENT => {
                let _input: ConsensusManagerStartInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ConsensusManagerBlueprint::start(api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT => {
                let input: ConsensusManagerGetCurrentTimeInputV1 =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ConsensusManagerBlueprint::get_current_time_v1(input.precision, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT => {
                let input: ConsensusManagerCompareCurrentTimeInputV1 =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ConsensusManagerBlueprint::compare_current_time_v1(
                    input.instant,
                    input.precision,
                    input.operator,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_NEXT_ROUND_IDENT => {
                let input: ConsensusManagerNextRoundInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ConsensusManagerBlueprint::next_round(
                    input.round,
                    input.proposer_timestamp_ms,
                    input.leader_proposal_history,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_CREATE_VALIDATOR_IDENT => {
                let input: ConsensusManagerCreateValidatorInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ConsensusManagerBlueprint::create_validator(
                    input.key,
                    input.fee_factor,
                    input.xrd_payment,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_REGISTER_IDENT => {
                let _input: ValidatorRegisterInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::register(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UNREGISTER_IDENT => {
                let _input: ValidatorUnregisterInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::unregister(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_STAKE_AS_OWNER_IDENT => {
                let input: ValidatorStakeAsOwnerInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::stake_as_owner(input.stake, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_STAKE_IDENT => {
                let input: ValidatorStakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::stake(input.stake, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UNSTAKE_IDENT => {
                let input: ValidatorUnstakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::unstake(input.stake_unit_bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_CLAIM_XRD_IDENT => {
                let input: ValidatorClaimXrdInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::claim_xrd(input.bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UPDATE_KEY_IDENT => {
                let input: ValidatorUpdateKeyInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::update_key(input.key, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UPDATE_FEE_IDENT => {
                let input: ValidatorUpdateFeeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::update_fee(input.new_fee_factor, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT => {
                let input: ValidatorUpdateAcceptDelegatedStakeInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::update_accept_delegated_stake(
                    input.accept_delegated_stake,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_ACCEPTS_DELEGATED_STAKE_IDENT => {
                let _: ValidatorAcceptsDelegatedStakeInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::accepts_delegated_stake(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_TOTAL_STAKE_XRD_AMOUNT_IDENT => {
                let _: ValidatorTotalStakeXrdAmountInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::total_stake_xrd_amount(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_TOTAL_STAKE_UNIT_SUPPLY_IDENT => {
                let _: ValidatorTotalStakeUnitSupplyInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::total_stake_unit_supply(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_GET_REDEMPTION_VALUE_IDENT => {
                let input: ValidatorGetRedemptionValueInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn =
                    ValidatorBlueprint::get_redemption_value(input.amount_of_stake_units, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_SIGNAL_PROTOCOL_UPDATE_READINESS_IDENT => {
                let input: ValidatorSignalProtocolUpdateReadinessInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::signal_protocol_update_readiness(input.vote, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_GET_PROTOCOL_UPDATE_READINESS_IDENT => {
                let _input: ValidatorGetProtocolUpdateReadinessInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::get_protocol_update_readiness(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_LOCK_OWNER_STAKE_UNITS_IDENT => {
                let input: ValidatorLockOwnerStakeUnitsInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::lock_owner_stake_units(input.stake_unit_bucket, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_START_UNLOCK_OWNER_STAKE_UNITS_IDENT => {
                let input: ValidatorStartUnlockOwnerStakeUnitsInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::start_unlock_owner_stake_units(
                    input.requested_stake_unit_amount,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_FINISH_UNLOCK_OWNER_STAKE_UNITS_IDENT => {
                let _input: ValidatorFinishUnlockOwnerStakeUnitsInput =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ValidatorBlueprint::finish_unlock_owner_stake_units(api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_APPLY_EMISSION_IDENT => {
                let input: ValidatorApplyEmissionInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::apply_emission(
                    input.xrd_bucket,
                    input.epoch,
                    input.proposals_made,
                    input.proposals_missed,
                    api,
                )?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            VALIDATOR_APPLY_REWARD_IDENT => {
                let input: ValidatorApplyRewardInput = input.as_typed().map_err(|e| {
                    RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                })?;
                let rtn = ValidatorBlueprint::apply_reward(input.xrd_bucket, input.epoch, api)?;
                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}

pub struct ConsensusManagerSecondsPrecisionNativeCode;

impl ConsensusManagerSecondsPrecisionNativeCode {
    pub fn invoke_export<Y: SystemApi<RuntimeError>>(
        export_name: &str,
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError> {
        match export_name {
            CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT => {
                let input: ConsensusManagerGetCurrentTimeInputV2 =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ConsensusManagerBlueprint::get_current_time_v2(input.precision, api)?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT => {
                let input: ConsensusManagerCompareCurrentTimeInputV2 =
                    input.as_typed().map_err(|e| {
                        RuntimeError::ApplicationError(ApplicationError::InputDecodeError(e))
                    })?;
                let rtn = ConsensusManagerBlueprint::compare_current_time_v2(
                    input.instant,
                    input.precision,
                    input.operator,
                    api,
                )?;

                Ok(IndexedScryptoValue::from_typed(&rtn))
            }
            _ => Err(RuntimeError::ApplicationError(
                ApplicationError::ExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
