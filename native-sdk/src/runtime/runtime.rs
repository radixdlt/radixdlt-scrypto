use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::clock::*;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::resource::AccessRule;
use radix_engine_interface::constants::{CLOCK, EPOCH_MANAGER};
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::time::*;
use radix_engine_interface::traits::ScryptoEvent;
use sbor::rust::prelude::*;

#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    /// Emits an application event
    pub fn emit_event<T: ScryptoEncode + ScryptoDescribe + ScryptoEvent, Y, E>(
        api: &mut Y,
        event: T,
    ) -> Result<(), E>
    where
        Y: ClientEventApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.emit_event(T::event_name().to_string(), scrypto_encode(&event).unwrap())
    }

    pub fn sys_current_epoch<Y, E>(api: &mut Y) -> Result<u64, E>
    where
        Y: ClientObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            EPOCH_MANAGER.as_node_id(),
            EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT,
            scrypto_encode(&EpochManagerGetCurrentEpochInput).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_current_time<Y, E>(api: &mut Y, precision: TimePrecision) -> Result<Instant, E>
    where
        Y: ClientObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            CLOCK.as_node_id(),
            CLOCK_GET_CURRENT_TIME_IDENT,
            scrypto_encode(&ClockGetCurrentTimeInput { precision }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_compare_against_current_time<Y, E>(
        api: &mut Y,
        instant: Instant,
        precision: TimePrecision,
        operator: TimeComparisonOperator,
    ) -> Result<bool, E>
    where
        Y: ClientObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            CLOCK.as_node_id(),
            CLOCK_COMPARE_CURRENT_TIME_IDENT,
            scrypto_encode(&ClockCompareCurrentTimeInput {
                precision,
                instant,
                operator,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    /// Generates a UUID.
    pub fn generate_uuid<Y, E>(api: &mut Y) -> Result<u128, E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.generate_uuid()
    }

    pub fn assert_access_rule<Y, E>(access_rule: AccessRule, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.assert_access_rule(access_rule)
    }
}
