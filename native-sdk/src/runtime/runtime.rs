use radix_engine_interface::api::types::{RENodeId, ScryptoReceiver};
use radix_engine_interface::api::{ClientComponentApi, ClientNativeInvokeApi, ClientNodeApi, ClientSubstateApi};
use radix_engine_interface::blueprints::clock::*;
use radix_engine_interface::blueprints::epoch_manager::*;
use radix_engine_interface::blueprints::transaction_runtime::*;
use radix_engine_interface::constants::{CLOCK, EPOCH_MANAGER};
use radix_engine_interface::data::{scrypto_decode, scrypto_encode, ScryptoCategorize, ScryptoDecode};
use radix_engine_interface::time::*;
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    pub fn sys_current_epoch<Y, E>(api: &mut Y) -> Result<u64, E>
    where
        Y: ClientComponentApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            ScryptoReceiver::Global(EPOCH_MANAGER),
            EPOCH_MANAGER_GET_CURRENT_EPOCH_IDENT,
            scrypto_encode(&EpochManagerGetCurrentEpochInput).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn sys_current_time<Y, E>(api: &mut Y, precision: TimePrecision) -> Result<Instant, E>
    where
        Y: ClientComponentApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            ScryptoReceiver::Global(CLOCK),
            CLOCK_GET_CURRENT_TIME_IDENT,
            scrypto_encode(&ClockGetCurrentTimeInput {
                precision
            }).unwrap()
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
        Y: ClientComponentApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            ScryptoReceiver::Global(CLOCK),
            CLOCK_COMPARE_CURRENT_TIME_IDENT,
            scrypto_encode(&ClockCompareCurrentTimeInput {
                precision,
                instant,
                operator,
            }).unwrap()
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    /// Generates a UUID.
    pub fn generate_uuid<Y, E>(api: &mut Y) -> Result<u128, E>
    where
        Y: ClientNodeApi<E> + ClientSubstateApi<E> + ClientNativeInvokeApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.call_native(TransactionRuntimeGenerateUuidInvocation {
            receiver: RENodeId::TransactionRuntime.into(),
        })
    }
}
