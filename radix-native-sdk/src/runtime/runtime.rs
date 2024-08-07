use radix_common::constants::CONSENSUS_MANAGER;
use radix_common::data::scrypto::*;
use radix_common::time::*;
use radix_common::traits::ScryptoEvent;
use radix_common::types::{NodeId, PackageAddress};
use radix_engine_interface::api::actor_api::EventFlags;
use radix_engine_interface::api::*;
use radix_engine_interface::blueprints::consensus_manager::*;
use radix_engine_interface::blueprints::resource::{
    AccessRule, AuthZoneAssertAccessRuleInput, AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT,
};
use radix_engine_interface::types::Epoch;
use sbor::rust::prelude::*;

#[derive(Debug)]
pub struct Runtime {}

impl Runtime {
    pub fn emit_event<
        Y: SystemApi<E>,
        E: SystemApiError,
        T: ScryptoEncode + ScryptoDescribe + ScryptoEvent,
    >(
        api: &mut Y,
        event: T,
    ) -> Result<(), E> {
        api.actor_emit_event(
            T::EVENT_NAME.to_string(),
            scrypto_encode(&event).unwrap(),
            EventFlags::empty(),
        )
    }

    pub fn emit_event_no_revert<
        Y: SystemApi<E>,
        E: SystemApiError,
        T: ScryptoEncode + ScryptoDescribe + ScryptoEvent,
    >(
        api: &mut Y,
        event: T,
    ) -> Result<(), E> {
        api.actor_emit_event(
            T::EVENT_NAME.to_string(),
            scrypto_encode(&event).unwrap(),
            EventFlags::FORCE_WRITE,
        )
    }

    pub fn current_epoch<Y: SystemObjectApi<E>, E: SystemApiError>(
        api: &mut Y,
    ) -> Result<Epoch, E> {
        let rtn = api.call_method(
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_GET_CURRENT_EPOCH_IDENT,
            scrypto_encode(&ConsensusManagerGetCurrentEpochInput).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn current_time<Y: SystemObjectApi<E>, E: SystemApiError>(
        precision: TimePrecision,
        api: &mut Y,
    ) -> Result<Instant, E> {
        let rtn = api.call_method(
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT,
            scrypto_encode(&ConsensusManagerGetCurrentTimeInputV2 { precision }).unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn compare_against_current_time<Y: SystemObjectApi<E>, E: SystemApiError>(
        instant: Instant,
        precision: TimePrecision,
        operator: TimeComparisonOperator,
        api: &mut Y,
    ) -> Result<bool, E> {
        let rtn = api.call_method(
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT,
            scrypto_encode(&ConsensusManagerCompareCurrentTimeInputV2 {
                precision,
                instant,
                operator,
            })
            .unwrap(),
        )?;

        Ok(scrypto_decode(&rtn).unwrap())
    }

    pub fn generate_ruid<Y: SystemApi<E>, E: SystemApiError>(api: &mut Y) -> Result<[u8; 32], E> {
        api.generate_ruid()
    }

    pub fn assert_access_rule<Y: SystemApi<E>, E: SystemApiError>(
        rule: AccessRule,
        api: &mut Y,
    ) -> Result<(), E> {
        let auth_zone = api.actor_get_node_id(ACTOR_REF_AUTH_ZONE)?;
        let _rtn = api.call_method(
            &auth_zone,
            AUTH_ZONE_ASSERT_ACCESS_RULE_IDENT,
            scrypto_encode(&AuthZoneAssertAccessRuleInput { rule }).unwrap(),
        )?;

        Ok(())
    }

    pub fn get_node_id<Y: SystemApi<E>, E: SystemApiError>(api: &mut Y) -> Result<NodeId, E> {
        api.actor_get_node_id(ACTOR_REF_SELF)
    }

    pub fn package_address<Y: SystemApi<E>, E: SystemApiError>(
        api: &mut Y,
    ) -> Result<PackageAddress, E> {
        api.actor_get_blueprint_id().map(|x| x.package_address)
    }
}
