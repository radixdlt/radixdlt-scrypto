use core::fmt::Debug;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::{EngineApi, Invokable};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use sbor::rust::collections::BTreeSet;
use scrypto::engine::scrypto_env::ScryptoEnv;

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct ComponentAuthZone {}

impl ComponentAuthZone {
    pub fn push<P: Into<Proof>>(proof: P) {
        let mut env = ScryptoEnv;
        let node_id = Self::auth_zone_node_id(&mut env).expect("Auth Zone doesn't exist");

        let proof: Proof = proof.into();

        env.invoke(AuthZonePushInvocation {
            receiver: node_id.into(),
            proof,
        })
        .unwrap();
    }

    pub fn pop() -> Proof {
        let mut env = ScryptoEnv;
        let node_id = Self::auth_zone_node_id(&mut env).expect("Auth Zone doesn't exist");
        env.invoke(AuthZonePopInvocation {
            receiver: node_id.into(),
        })
        .unwrap()
    }

    pub fn create_proof(resource_address: ResourceAddress) -> Proof {
        let mut env = ScryptoEnv;
        let node_id = Self::auth_zone_node_id(&mut env).expect("Auth Zone doesn't exist");
        env.invoke(AuthZoneCreateProofInvocation {
            receiver: node_id.into(),
            resource_address,
        })
        .unwrap()
    }

    pub fn create_proof_by_amount(amount: Decimal, resource_address: ResourceAddress) -> Proof {
        let mut env = ScryptoEnv;
        let node_id = Self::auth_zone_node_id(&mut env).expect("Auth Zone doesn't exist");
        env.invoke(AuthZoneCreateProofByAmountInvocation {
            receiver: node_id.into(),
            amount,
            resource_address,
        })
        .unwrap()
    }

    pub fn create_proof_by_ids(
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
    ) -> Proof {
        let mut env = ScryptoEnv;
        let node_id = Self::auth_zone_node_id(&mut env).expect("Auth Zone doesn't exist");
        env.invoke(AuthZoneCreateProofByIdsInvocation {
            receiver: node_id.into(),
            ids: ids.clone(),
            resource_address,
        })
        .unwrap()
    }

    pub fn assert_access_rule(access_rule: AccessRule) {
        let mut env = ScryptoEnv;
        let node_id = Self::auth_zone_node_id(&mut env).expect("Auth Zone doesn't exist");
        env.invoke(AuthZoneAssertAccessRuleInvocation {
            receiver: node_id.into(),
            access_rule,
        })
        .unwrap()
    }

    fn auth_zone_node_id<Y, E>(api: &mut Y) -> Option<RENodeId>
    where
        Y: EngineApi<E>,
        E: Debug,
    {
        let owned_node_ids = api.sys_get_visible_nodes().unwrap();
        owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
    }
}
