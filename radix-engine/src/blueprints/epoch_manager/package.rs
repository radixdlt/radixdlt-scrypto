use crate::blueprints::epoch_manager::{EpochManagerBlueprint, ValidatorBlueprint};
use crate::errors::InterpreterError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::auth::{
    HardAuthRule, HardProofRule, HardResourceOrNonFungible, MethodAuthorization,
};
use crate::types::*;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientNativeInvokeApi;
use radix_engine_interface::api::{ClientApi, ClientSubstateApi};
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::data::ScryptoValue;

pub struct EpochManagerNativePackage;

impl EpochManagerNativePackage {
    pub fn create_auth() -> Vec<MethodAuthorization> {
        vec![MethodAuthorization::Protected(HardAuthRule::ProofRule(
            HardProofRule::Require(HardResourceOrNonFungible::NonFungible(
                AuthAddresses::system_role(),
            )),
        ))]
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<ComponentId>,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi
            + KernelSubstateApi
            + ClientSubstateApi<RuntimeError>
            + ClientApi<RuntimeError>
            + ClientNativeInvokeApi<RuntimeError>,
    {
        match export_name {
            EPOCH_MANAGER_CREATE_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                EpochManagerBlueprint::create(input, api)
            }
            EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                EpochManagerBlueprint::get_current_epoch(receiver, input, api)
            }
            EPOCH_MANAGER_SET_EPOCH_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                EpochManagerBlueprint::set_epoch(receiver, input, api)
            }
            EPOCH_MANAGER_NEXT_ROUND_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                EpochManagerBlueprint::next_round(receiver, input, api)
            }
            EPOCH_MANAGER_CREATE_VALIDATOR_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                EpochManagerBlueprint::create_validator(receiver, input, api)
            }
            EPOCH_MANAGER_UPDATE_VALIDATOR_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                EpochManagerBlueprint::update_validator(receiver, input, api)
            }
            VALIDATOR_REGISTER_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::register(receiver, input, api)
            }
            VALIDATOR_UNREGISTER_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::unregister(receiver, input, api)
            }
            VALIDATOR_STAKE_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::stake(receiver, input, api)
            }
            VALIDATOR_UNSTAKE_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::unstake(receiver, input, api)
            }
            VALIDATOR_CLAIM_XRD_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::claim_xrd(receiver, input, api)
            }
            VALIDATOR_UPDATE_KEY_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::update_key(receiver, input, api)
            }
            VALIDATOR_UPDATE_ACCEPT_DELEGATED_STAKE_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ValidatorBlueprint::update_accept_delegated_stake(receiver, input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
