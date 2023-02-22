use crate::errors::{InterpreterError, RuntimeError};
use crate::kernel::kernel_api::{KernelNodeApi, KernelSubstateApi};
use crate::system::global::GlobalSubstate;
use crate::system::kernel_modules::costing::{FIXED_HIGH_FEE, FIXED_LOW_FEE};
use crate::system::node::RENodeInit;
use crate::system::node::RENodeModuleInit;
use crate::system::node_modules::access_rules::ObjectAccessRulesChainSubstate;
use crate::types::*;
use radix_engine_interface::api::node_modules::auth::AuthAddresses;
use radix_engine_interface::api::substate_api::LockFlags;
use radix_engine_interface::api::types::*;
use radix_engine_interface::api::types::{Address, ClockOffset, RENodeId, SubstateOffset};
use radix_engine_interface::api::unsafe_api::ClientCostingReason;
use radix_engine_interface::api::ClientApi;
use radix_engine_interface::api::ClientSubstateApi;
use radix_engine_interface::blueprints::clock::ClockCreateInput;
use radix_engine_interface::blueprints::clock::TimePrecision;
use radix_engine_interface::blueprints::clock::*;
use radix_engine_interface::blueprints::resource::AccessRuleKey;
use radix_engine_interface::blueprints::resource::AccessRules;
use radix_engine_interface::blueprints::resource::{require, AccessRule};
use radix_engine_interface::data::ScryptoValue;
use radix_engine_interface::rule;
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
    pub fn package_access_rules() -> BTreeMap<(String, String), AccessRule> {
        let mut access_rules = BTreeMap::new();
        access_rules.insert(
            (CLOCK_BLUEPRINT.to_string(), CLOCK_CREATE_IDENT.to_string()),
            rule!(require(AuthAddresses::system_role())),
        );
        access_rules
    }

    pub fn invoke_export<Y>(
        export_name: &str,
        receiver: Option<RENodeId>,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        match export_name {
            CLOCK_CREATE_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                if receiver.is_some() {
                    return Err(RuntimeError::InterpreterError(
                        InterpreterError::NativeUnexpectedReceiver(export_name.to_string()),
                    ));
                }
                Self::create(input, api)
            }
            CLOCK_GET_CURRENT_TIME_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::get_current_time(receiver, input, api)
            }
            CLOCK_SET_CURRENT_TIME_IDENT => {
                api.consume_cost_units(FIXED_LOW_FEE, ClientCostingReason::RunPrecompiled)?;

                let receiver = receiver.ok_or(RuntimeError::InterpreterError(
                    InterpreterError::NativeExpectedReceiver(export_name.to_string()),
                ))?;
                Self::set_current_time(receiver, input, api)
            }
            CLOCK_COMPARE_CURRENT_TIME_IDENT => {
                api.consume_cost_units(FIXED_HIGH_FEE, ClientCostingReason::RunPrecompiled)?;

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

    fn create<Y>(input: ScryptoValue, api: &mut Y) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientSubstateApi<RuntimeError>,
    {
        // TODO: Remove decode/encode mess
        let input: ClockCreateInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let underlying_node_id = api.kernel_allocate_node_id(RENodeType::Clock)?;

        let mut access_rules = AccessRules::new();
        access_rules.set_method_access_rule(
            AccessRuleKey::new(NodeModuleId::SELF, CLOCK_SET_CURRENT_TIME_IDENT.to_string()),
            rule!(require(AuthAddresses::validator_role())),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::new(NodeModuleId::SELF, CLOCK_GET_CURRENT_TIME_IDENT.to_string()),
            rule!(allow_all),
        );
        access_rules.set_method_access_rule(
            AccessRuleKey::new(
                NodeModuleId::SELF,
                CLOCK_COMPARE_CURRENT_TIME_IDENT.to_string(),
            ),
            rule!(allow_all),
        );

        let mut node_modules = BTreeMap::new();
        node_modules.insert(
            NodeModuleId::AccessRules,
            RENodeModuleInit::ObjectAccessRulesChain(ObjectAccessRulesChainSubstate {
                access_rules_chain: vec![access_rules],
            }),
        );

        api.kernel_create_node(
            underlying_node_id,
            RENodeInit::Clock(CurrentTimeRoundedToMinutesSubstate {
                current_time_rounded_to_minutes_ms: 0,
            }),
            node_modules,
        )?;

        let global_node_id = RENodeId::Global(Address::Component(ComponentAddress::Clock(
            input.component_address,
        )));
        api.kernel_create_node(
            global_node_id,
            RENodeInit::Global(GlobalSubstate::Clock(underlying_node_id.into())),
            BTreeMap::new(),
        )?;

        let address: ComponentAddress = global_node_id.into();

        Ok(IndexedScryptoValue::from_typed(&address))
    }

    fn set_current_time<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: ClockSetCurrentTimeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

        let current_time_ms = input.current_time_ms;
        let current_time_rounded_to_minutes =
            (current_time_ms / MINUTES_TO_MS_FACTOR) * MINUTES_TO_MS_FACTOR;

        let handle = api.sys_lock_substate(
            receiver,
            SubstateOffset::Clock(ClockOffset::CurrentTimeRoundedToMinutes),
            LockFlags::MUTABLE,
        )?;
        let mut substate_ref = api.kernel_get_substate_ref_mut(handle)?;
        substate_ref
            .current_time_rounded_to_minutes()
            .current_time_rounded_to_minutes_ms = current_time_rounded_to_minutes;

        Ok(IndexedScryptoValue::from_typed(&()))
    }

    fn get_current_time<Y>(
        receiver: RENodeId,
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: ClockGetCurrentTimeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

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
        input: ScryptoValue,
        api: &mut Y,
    ) -> Result<IndexedScryptoValue, RuntimeError>
    where
        Y: KernelNodeApi + KernelSubstateApi + ClientApi<RuntimeError>,
    {
        let input: ClockCompareCurrentTimeInput = scrypto_decode(&scrypto_encode(&input).unwrap())
            .map_err(|_| RuntimeError::InterpreterError(InterpreterError::InvalidInvocation))?;

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
