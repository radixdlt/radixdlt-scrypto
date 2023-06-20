use radix_engine_common::types::NodeId;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::resource::AccessRule;
use radix_engine_interface::constants::CONSENSUS_MANAGER;
use radix_engine_interface::data::scrypto::*;
use radix_engine_interface::time::*;
use radix_engine_interface::traits::ScryptoEvent;
use radix_engine_interface::types::Epoch;
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
        Y: ClientTransactionRuntimeApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.emit_event(T::event_name().to_string(), scrypto_encode(&event).unwrap())
    }

    pub fn current_epoch<Y, E>(api: &mut Y) -> Result<Epoch, E>
    where
        Y: ClientObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
            scrypto_encode(&ConsensusManagerGetCurrentEpochInput).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn current_time<Y, E>(api: &mut Y, precision: TimePrecision) -> Result<Instant, E>
    where
        Y: ClientObjectApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let rtn = api.call_method(
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT,
            scrypto_encode(&ConsensusManagerGetCurrentTimeInput { precision }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn compare_against_current_time<Y, E>(
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
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT,
            scrypto_encode(&ConsensusManagerCompareCurrentTimeInput {
                precision,
                instant,
                operator,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn generate_ruid<Y, E>(api: &mut Y) -> Result<[u8; 32], E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.generate_ruid()
    }

    pub fn assert_access_rule<Y, E>(access_rule: AccessRule, api: &mut Y) -> Result<(), E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.assert_access_rule(access_rule)
    }

    pub fn get_node_id<Y, E>(api: &mut Y) -> Result<NodeId, E>
    where
        Y: ClientApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        api.actor_get_node_id()
    }
}
