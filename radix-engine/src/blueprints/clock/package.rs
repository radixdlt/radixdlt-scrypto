use crate::errors::{InterpreterError, RuntimeError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::{FIXED_HIGH_FEE, FIXED_LOW_FEE};
use crate::types::*;
use native_sdk::access_rules::AccessRulesObject;
use native_sdk::metadata::Metadata;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::clock::ClockCreateInput;
use radix_engine_interface::blueprints::clock::TimePrecision;
use radix_engine_interface::blueprints::clock::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::rule;
use radix_engine_interface::schema::{BlueprintSchema, FunctionSchema, PackageSchema};
use radix_engine_interface::time::*;

#[derive(Debug, Clone, Sbor, PartialEq, Eq)]
pub struct CurrentTimeRoundedToMinutesSubstate {
    pub current_time_rounded_to_minutes_ms: i64,
}

const SECONDS_TO_MS_FACTOR: i64 = 1000;
const MINUTES_TO_SECONDS_FACTOR: i64 = 60;
const MINUTES_TO_MS_FACTOR: i64 = SECONDS_TO_MS_FACTOR * MINUTES_TO_SECONDS_FACTOR;

pub struct ClockNativePackage;
impl ClockNativePackage {
    pub fn schema() -> PackageSchema {
        let mut aggregator = TypeAggregator::<ScryptoCustomTypeKind>::new();

        let mut substates = Vec::new();
        substates.push(aggregator.add_child_type_and_descendents::<AccessControllerSubstate>());

        let mut functions = BTreeMap::new();
        functions.insert(
            ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator
                    .add_child_type_and_descendents::<AccessControllerCreateGlobalInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<AccessControllerCreateGlobalOutput>(),
                export_name: ACCESS_CONTROLLER_CREATE_GLOBAL_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        PackageSchema {
            blueprints: btreemap!(
                CLOCK_BLUEPRINT.to_string() => BlueprintSchema {
                    schema,
                    substates,
                    functions
                }
            ),
        }
    }

    pub fn package_access_rules() -> BTreeMap<FnKey, AccessRule> {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            FnKey::new(CLOCK_BLUEPRINT.to_string(), CLOCK_CREATE_IDENT.to_string()),
            rule!(require(AuthAddresses::system_role())),
        );
        access_rules
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            CLOCK_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                Self::create(input, api)
            }
            CLOCK_GET_CURRENT_TIME_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::get_current_time(receiver, input, api)
            }
            CLOCK_SET_CURRENT_TIME_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::set_current_time(receiver, input, api)
            }
            CLOCK_COMPARE_CURRENT_TIME_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::compare_current_time(receiver, input, api)
            }
            _ => Err(RuntimeError::InterpreterError(
                InterpreterError::NativeExportDoesNotExist(export_name.to_string()),
            )),
        }
    }

    fn create<Y>(
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: ClockCreateInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let clock_id = api.new_object(
            CLOCK_BLUEPRINT,
            vec![scrypto_encode(&CurrentTimeRoundedToMinutesSubstate {
                current_time_rounded_to_minutes_ms: 0,
            })
            .unwrap()],
        )?;

        let mut access_rules = AccessRules::new();
        access_rules.set_method_access_rule(
            MethodKey::new(NodeModuleId::SELF, CLOCK_SET_CURRENT_TIME_IDENT.to_string()),
            rule!(require(AuthAddresses::validator_role())),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(NodeModuleId::SELF, CLOCK_GET_CURRENT_TIME_IDENT.to_string()),
            rule!(allow_all),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(
                NodeModuleId::SELF,
                CLOCK_COMPARE_CURRENT_TIME_IDENT.to_string(),
            ),
            rule!(allow_all),
        );
        let access_rules = AccessRulesObject::sys_new(access_rules, api)?;
        let metadata = Metadata::sys_create(api)?;
        let address = ComponentAddress::Clock(input.component_address);
        api.globalize_with_address(
            RENodeId::Object(clock_id),
            btreemap!(
                NodeModuleId::AccessRules => access_rules.id(),
                NodeModuleId::Metadata => metadata.id(),
            ),
            address.into(),
        )?;

        Ok(IndexedScryptoValue::from_typed(&address))
    }

    fn set_current_time<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: ClockSetCurrentTimeInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        let current_time_ms = input.current_time_ms;
        let current_time_rounded_to_minutes =
            (current_time_ms / MINUTES_TO_MS_FACTOR) * MINUTES_TO_MS_FACTOR;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes),
            LockFlags::MUTABLE,
        )?;
        let current_time_rounded_to_minutes_substate: &mut CurrentTimeRoundedToMinutesSubstate =
            api.kernel_get_substate_ref_mut(handle)?;
        current_time_rounded_to_minutes_substate.current_time_rounded_to_minutes_ms =
            current_time_rounded_to_minutes;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn get_current_time<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: ClockGetCurrentTimeInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        match input.precision {
            TimePrecision::Minute => {
                let handle = api.sys_lock_substate(
                    receiver,
                    SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes),
                    LockFlags::read_only(),
                )?;
                let substate: &CurrentTimeRoundedToMinutesSubstate =
                    api.kernel_get_substate_ref(handle)?;
                let instant = Instant::new(
                    substate.current_time_rounded_to_minutes_ms / SECONDS_TO_MS_FACTOR,
                );
                Ok(IndexedScryptoValue::from_typed(&instant))
            }
        }
    }

    fn compare_current_time<Y>(
        receiver: RENodeId,
        input: IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: ClockCompareCurrentTimeInput = input.as_typed().map_err(|e| {
            RuntimeError::InterpreterError(InterpreterError::ScryptoInputDecodeError(e))
        })?;

        match input.precision {
            TimePrecision::Minute => {
                let handle = api.sys_lock_substate(
                    receiver,
                    SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes),
                    LockFlags::read_only(),
                )?;
                let substate: &CurrentTimeRoundedToMinutesSubstate =
                    api.kernel_get_substate_ref(handle)?;
                let current_time_instant = Instant::new(
                    substate.current_time_rounded_to_minutes_ms / SECONDS_TO_MS_FACTOR,
                );
                let other_instant_rounded = Instant::new(
                    (input.instant.seconds_since_unix_epoch / MINUTES_TO_SECONDS_FACTOR)
                        * MINUTES_TO_SECONDS_FACTOR,
                );
                let result = current_time_instant.compare(other_instant_rounded, input.operator);
                Ok(IndexedScryptoValue::from_typed(&result))
            }
        }
    }
}
