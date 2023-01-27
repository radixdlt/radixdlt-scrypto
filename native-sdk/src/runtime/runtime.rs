use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::{EngineApi, Invokable};
use radix_engine_interface::constants::{CLOCK, EPOCH_MANAGER};
use radix_engine_interface::data::{ScryptoCategorize, ScryptoDecode};
use radix_engine_interface::model::*;
use radix_engine_interface::time::{Instant, TimeComparisonOperator};
use sbor::rust::fmt::Debug;

#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    pub fn sys_current_epoch<Y, E>(api: &mut Y) -> Result<u64, E>
    where
        Y: Invokable<EpochManagerGetCurrentEpochInvocation, E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.invoke(EpochManagerGetCurrentEpochInvocation {
            receiver: EPOCH_MANAGER,
        })
    }

    pub fn sys_current_time<Y, E>(api: &mut Y, precision: TimePrecision) -> Result<Instant, E>
    where
        Y: Invokable<ClockGetCurrentTimeInvocation, E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.invoke(ClockGetCurrentTimeInvocation {
            receiver: CLOCK,
            precision,
        })
    }

    pub fn sys_compare_against_current_time<Y, E>(
        api: &mut Y,
        instant: Instant,
        precision: TimePrecision,
        operator: TimeComparisonOperator,
    ) -> Result<bool, E>
    where
        Y: Invokable<ClockCompareCurrentTimeInvocation, E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.invoke(ClockCompareCurrentTimeInvocation {
            receiver: CLOCK,
            precision,
            instant,
            operator,
        })
    }

    /// Generates a UUID.
    pub fn generate_uuid<Y, E>(api: &mut Y) -> Result<u128, E>
    where
        Y: EngineApi<E> + Invokable<TransactionRuntimeGenerateUuidInvocation, E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let visible_node_ids = api.sys_get_visible_nodes()?;
        let node_id = visible_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::TransactionRuntime(..)))
            .expect("TransactionHash does not exist");

        api.invoke(TransactionRuntimeGenerateUuidInvocation {
            receiver: node_id.into(),
        })
    }
}
