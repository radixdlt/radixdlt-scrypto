use crate::errors::{InterpreterError, RuntimeError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::kernel_modules::costing::{FIXED_HIGH_FEE, FIXED_LOW_FEE};
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::blueprints::clock::ClockCreateInput;
use radix_engine_interface::blueprints::clock::TimePrecision;
use radix_engine_interface::blueprints::clock::*;
use radix_engine_interface::blueprints::resource::*;
use radix_engine_interface::rule;
use radix_engine_interface::schema::{BlueprintSchema, FunctionSchema, PackageSchema, Receiver};
use radix_engine_interface::time::*;
use resources_tracker_macro::trace_resources;

#[derive(Debug, Clone, Sbor, PartialEq, Eq)]
pub struct ClockSubstate {
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
        substates.push(aggregator.add_child_type_and_descendents::<ClockSubstate>());

        let mut functions = BTreeMap::new();
        functions.insert(
            CLOCK_CREATE_IDENT.to_string(),
            FunctionSchema {
                receiver: None,
                input: aggregator.add_child_type_and_descendents::<ClockCreateInput>(),
                output: aggregator.add_child_type_and_descendents::<ClockCreateOutput>(),
                export_name: CLOCK_CREATE_IDENT.to_string(),
            },
        );
        functions.insert(
            CLOCK_GET_CURRENT_TIME_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRef),
                input: aggregator.add_child_type_and_descendents::<ClockGetCurrentTimeInput>(),
                output: aggregator.add_child_type_and_descendents::<ClockGetCurrentTimeOutput>(),
                export_name: CLOCK_GET_CURRENT_TIME_IDENT.to_string(),
            },
        );
        functions.insert(
            CLOCK_SET_CURRENT_TIME_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<ClockSetCurrentTimeInput>(),
                output: aggregator.add_child_type_and_descendents::<ClockSetCurrentTimeOutput>(),
                export_name: CLOCK_SET_CURRENT_TIME_IDENT.to_string(),
            },
        );
        functions.insert(
            CLOCK_COMPARE_CURRENT_TIME_IDENT.to_string(),
            FunctionSchema {
                receiver: Some(Receiver::SelfRefMut),
                input: aggregator.add_child_type_and_descendents::<ClockCompareCurrentTimeInput>(),
                output: aggregator
                    .add_child_type_and_descendents::<ClockCompareCurrentTimeOutput>(),
                export_name: CLOCK_COMPARE_CURRENT_TIME_IDENT.to_string(),
            },
        );

        let schema = generate_full_schema(aggregator);
        PackageSchema {
            blueprints: btreemap!(
                CLOCK_BLUEPRINT.to_string() => BlueprintSchema {
                    parent: None,
                    schema,
                    substates,
                    functions,
                    virtual_lazy_load_functions: btreemap!(),
                    event_schema: [].into()
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

    #[trace_resources(log=export_name)]
    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<&NodeId>,
        input: &IndexedScryptoValue,
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
        input: &IndexedScryptoValue,
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
            vec![scrypto_encode(&ClockSubstate {
                current_time_rounded_to_minutes_ms: 0,
            })
            .unwrap()],
        )?;

        let mut access_rules = AccessRulesConfig::new();
        access_rules.set_method_access_rule(
            MethodKey::new(TypedModuleId::ObjectState, CLOCK_SET_CURRENT_TIME_IDENT),
            rule!(require(AuthAddresses::validator_role())),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(TypedModuleId::ObjectState, CLOCK_GET_CURRENT_TIME_IDENT),
            rule!(allow_all),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(TypedModuleId::ObjectState, CLOCK_COMPARE_CURRENT_TIME_IDENT),
            rule!(allow_all),
        );
        let access_rules = AccessRules::sys_new(access_rules, btreemap!(), api)?.0;
        let metadata = Metadata::sys_create(api)?;
        let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

        let address = ComponentAddress::new_unchecked(input.component_address);
        api.globalize_with_address(
            clock_id,
            btreemap!(
                TypedModuleId::AccessRules => access_rules.0,
                TypedModuleId::Metadata => metadata.0,
                TypedModuleId::Royalty => royalty.0,
            ),
            address.into(),
        )?;

        Ok(IndexedScryptoValue::from_typed(&address))
    }

    fn set_current_time<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
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
            &ClockOffset::CurrentTimeRoundedToMinutes.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: ClockSubstate = api.sys_read_substate_typed(handle)?;
        substate.current_time_rounded_to_minutes_ms = current_time_rounded_to_minutes;
        api.sys_write_substate_typed(handle, &substate)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn get_current_time<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
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
                    &ClockOffset::CurrentTimeRoundedToMinutes.into(),
                    LockFlags::read_only(),
                )?;
                let substate: ClockSubstate = api.sys_read_substate_typed(handle)?;
                let instant = Instant::new(
                    substate.current_time_rounded_to_minutes_ms / SECONDS_TO_MS_FACTOR,
                );
                Ok(IndexedScryptoValue::from_typed(&instant))
            }
        }
    }

    fn compare_current_time<Y>(
        receiver: &NodeId,
        input: &IndexedScryptoValue,
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
                    &ClockOffset::CurrentTimeRoundedToMinutes.into(),
                    LockFlags::read_only(),
                )?;
                let substate: ClockSubstate = api.sys_read_substate_typed(handle)?;
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
