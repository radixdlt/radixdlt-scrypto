use crate::blueprints::resource::*;
use crate::errors::InterpreterError;
use crate::errors::RuntimeError;
use crate::kernel::kernel_api::KernelSubstateApi;
use crate::kernel::KernelNodeApi;
use crate::types::*;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::ClientNativeInvokeApi;
use radix_engine_interface::api::ClientSubstateApi;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::data::ScryptoValue;

pub struct ResourceManagerNativePackage;

impl ResourceManagerNativePackage {
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<ResourceManagerId>,
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
            RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_non_fungible(input, api)
            }
            RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_ADDRESS_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_non_fungible_with_address(input, api)
            }
            RESOURCE_MANAGER_CREATE_NON_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_non_fungible_with_initial_supply(input, api)
            }
            RESOURCE_MANAGER_CREATE_UUID_NON_FUNGIBLE_WITH_INITIAL_SUPPLY => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_uuid_non_fungible_with_initial_supply(input, api)
            }
            RESOURCE_MANAGER_CREATE_FUNGIBLE_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_fungible(input, api)
            }
            RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_fungible_with_initial_supply(input, api)
            }
            RESOURCE_MANAGER_CREATE_FUNGIBLE_WITH_INITIAL_SUPPLY_AND_ADDRESS_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::create_fungible_with_initial_supply_and_address(
                    input, api,
                )
            }
            RESOURCE_MANAGER_BURN_BUCKET_IDENT => {
                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                ResourceManagerBlueprint::burn_bucket(input, api)
            }
            RESOURCE_MANAGER_MINT_NON_FUNGIBLE => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::mint_non_fungible(receiver, input, api)
            }
            RESOURCE_MANAGER_MINT_UUID_NON_FUNGIBLE => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::mint_uuid_non_fungible(receiver, input, api)
            }
            RESOURCE_MANAGER_MINT_FUNGIBLE => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::mint_fungible(receiver, input, api)
            }
            RESOURCE_MANAGER_BURN_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::burn(receiver, input, api)
            }
            RESOURCE_MANAGER_CREATE_BUCKET_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::create_bucket(receiver, input, api)
            }
            RESOURCE_MANAGER_CREATE_VAULT_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::create_vault(receiver, input, api)
            }
            RESOURCE_MANAGER_UPDATE_VAULT_AUTH_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::update_vault_auth(receiver, input, api)
            }
            RESOURCE_MANAGER_SET_VAULT_AUTH_MUTABILITY_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::set_vault_auth_mutability(receiver, input, api)
            }
            RESOURCE_MANAGER_UPDATE_NON_FUNGIBLE_DATA_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::update_non_fungible_data(receiver, input, api)
            }
            RESOURCE_MANAGER_NON_FUNGIBLE_EXISTS_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::non_fungible_exists(receiver, input, api)
            }
            RESOURCE_MANAGER_GET_RESOURCE_TYPE_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::get_resource_type(receiver, input, api)
            }
            RESOURCE_MANAGER_GET_TOTAL_SUPPLY_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::get_total_supply(receiver, input, api)
            }
            RESOURCE_MANAGER_GET_NON_FUNGIBLE_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                ResourceManagerBlueprint::get_non_fungible(receiver, input, api)
            }
            VAULT_LOCK_FEE_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::lock_fee(receiver, input, api)
            }
            VAULT_TAKE_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::take(receiver, input, api)
            }
            VAULT_TAKE_NON_FUNGIBLES_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::take_non_fungibles(receiver, input, api)
            }
            VAULT_RECALL_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::recall(receiver, input, api)
            }
            VAULT_RECALL_NON_FUNGIBLES_IDENT => {
                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                VaultBlueprint::recall_non_fungibles(receiver, input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }
}
