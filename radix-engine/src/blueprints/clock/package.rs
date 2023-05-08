use crate::errors::{RuntimeError, SystemUpstreamError};
use crate::system::system_modules::costing::{FIXED_HIGH_FEE, FIXED_LOW_FEE};
use crate::types::*;
use native_sdk::modules::access_rules::AccessRules;
use native_sdk::modules::metadata::Metadata;
use native_sdk::modules::royalty::ComponentRoyalty;
use radix_engine_interface::api::field_lock_api::LockFlags;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::object_api::ObjectModuleId;
use radix_engine_interface::api::{ClientApi, OBJECT_HANDLE_SELF};
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

        let mut fields = Vec::new();
        fields.push(aggregator.add_child_type_and_descendents::<ClockSubstate>());

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
                    outer_blueprint: None,
                    schema,
                    fields,
                    kv_stores: vec![],
                    indices: vec![],
                    sorted_indices: vec![],
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
        Y: ClientApi<RuntimeError>,
    {
        match export_name {
            CLOCK_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                if receiver.is_some() {
                    return Err(RuntimeError::SystemUpstreamError(
                        SystemUpstreamError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                Self::create(input, api)
            }
            CLOCK_GET_CURRENT_TIME_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::get_current_time(input, api)
            }
            CLOCK_SET_CURRENT_TIME_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunNative)?;

                Self::set_current_time(input, api)
            }
            CLOCK_COMPARE_CURRENT_TIME_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunNative)?;

                Self::compare_current_time(input, api)
            }
            _ => Err(RuntimeError::SystemUpstreamError(
                SystemUpstreamError::NativeExportDoesNotExist(export_name.to_string()),
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
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let clock_id = api.new_simple_object(
            CLOCK_BLUEPRINT,
            vec![scrypto_encode(&ClockSubstate {
                current_time_rounded_to_minutes_ms: 0,
            })
            .unwrap()],
        )?;

        let mut access_rules = AccessRulesConfig::new();
        access_rules.set_method_access_rule(
            MethodKey::new(ObjectModuleId::Main, CLOCK_SET_CURRENT_TIME_IDENT),
            rule!(require(AuthAddresses::validator_role())),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(ObjectModuleId::Main, CLOCK_GET_CURRENT_TIME_IDENT),
            rule!(allow_all),
        );
        access_rules.set_method_access_rule(
            MethodKey::new(ObjectModuleId::Main, CLOCK_COMPARE_CURRENT_TIME_IDENT),
            rule!(allow_all),
        );
        let access_rules = AccessRules::sys_new(access_rules, btreemap!(), api)?.0;
        let metadata = Metadata::sys_create(api)?;
        let royalty = ComponentRoyalty::sys_create(RoyaltyConfig::default(), api)?;

        let address = ComponentAddress::new_or_panic(input.component_address);
        api.globalize_with_address(
            btreemap!(
                ObjectModuleId::Main => clock_id,
                ObjectModuleId::AccessRules => access_rules.0,
                ObjectModuleId::Metadata => metadata.0,
                ObjectModuleId::Royalty => royalty.0,
            ),
            address.into(),
        )?;

        Ok(IndexedScryptoValue::from_typed(&address))
    }

    fn set_current_time<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: ClockSetCurrentTimeInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        let current_time_ms = input.current_time_ms;
        let current_time_rounded_to_minutes =
            (current_time_ms / MINUTES_TO_MS_FACTOR) * MINUTES_TO_MS_FACTOR;

        let handle = api.actor_lock_field(
            OBJECT_HANDLE_SELF,
            ClockField::CurrentTimeRoundedToMinutes.into(),
            LockFlags::MUTABLE,
        )?;
        let mut substate: ClockSubstate = api.field_lock_read_typed(handle)?;
        substate.current_time_rounded_to_minutes_ms = current_time_rounded_to_minutes;
        api.field_lock_write_typed(handle, &substate)?;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn get_current_time<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: ClockGetCurrentTimeInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        match input.precision {
            TimePrecision::Minute => {
                let handle = api.actor_lock_field(
                    OBJECT_HANDLE_SELF,
                    ClockField::CurrentTimeRoundedToMinutes.into(),
                    LockFlags::read_only(),
                )?;
                let substate: ClockSubstate = api.field_lock_read_typed(handle)?;
                let instant = Instant::new(
                    substate.current_time_rounded_to_minutes_ms / SECONDS_TO_MS_FACTOR,
                );
                Ok(IndexedScryptoValue::from_typed(&instant))
            }
        }
    }

    fn compare_current_time<Y>(
        input: &IndexedScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: ClientApi<RuntimeError>,
    {
        let input: ClockCompareCurrentTimeInput = input.as_typed().map_err(|e| {
            RuntimeError::SystemUpstreamError(SystemUpstreamError::InputDecodeError(e))
        })?;

        match input.precision {
            TimePrecision::Minute => {
                let handle = api.actor_lock_field(
                    OBJECT_HANDLE_SELF,
                    ClockField::CurrentTimeRoundedToMinutes.into(),
                    LockFlags::read_only(),
                )?;
                let substate: ClockSubstate = api.field_lock_read_typed(handle)?;
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
