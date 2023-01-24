use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::{EngineApi, Invokable};
use radix_engine_interface::data::{ScryptoCategorize, ScryptoDecode};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;

pub struct ComponentAuthZone {}

impl ComponentAuthZone {
    pub fn sys_drain<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        env: &mut Y,
    ) -> Result<Vec<Proof>, E>
    where
        Y: EngineApi<E> + Invokable<AuthZoneDrainInvocation, E>,
    {
        let node_id = Self::auth_zone_node_id(env).expect("Auth Zone doesn't exist");
        env.invoke(AuthZoneDrainInvocation {
            receiver: node_id.into(),
        })
    }

    pub fn sys_clear<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(env: &mut Y) -> Result<(), E>
    where
        Y: EngineApi<E> + Invokable<AuthZoneClearInvocation, E>,
    {
        let node_id = Self::auth_zone_node_id(env).expect("Auth Zone doesn't exist");
        env.invoke(AuthZoneClearInvocation {
            receiver: node_id.into(),
        })
    }

    pub fn sys_pop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(env: &mut Y) -> Result<Proof, E>
    where
        Y: EngineApi<E> + Invokable<AuthZonePopInvocation, E>,
    {
        let node_id = Self::auth_zone_node_id(env).expect("Auth Zone doesn't exist");
        env.invoke(AuthZonePopInvocation {
            receiver: node_id.into(),
        })
    }

    pub fn sys_create_proof<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: EngineApi<E> + Invokable<AuthZoneCreateProofInvocation, E>,
    {
        let node_id = Self::auth_zone_node_id(env).expect("Auth Zone doesn't exist");
        env.invoke(AuthZoneCreateProofInvocation {
            receiver: node_id.into(),
            resource_address,
        })
    }

    pub fn sys_create_proof_by_amount<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        amount: Decimal,
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: EngineApi<E> + Invokable<AuthZoneCreateProofByAmountInvocation, E>,
    {
        let node_id = Self::auth_zone_node_id(env).expect("Auth Zone doesn't exist");
        env.invoke(AuthZoneCreateProofByAmountInvocation {
            receiver: node_id.into(),
            amount,
            resource_address,
        })
    }

    pub fn sys_create_proof_by_ids<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        ids: &BTreeSet<NonFungibleLocalId>,
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: EngineApi<E> + Invokable<AuthZoneCreateProofByIdsInvocation, E>,
    {
        let node_id = Self::auth_zone_node_id(env).expect("Auth Zone doesn't exist");
        env.invoke(AuthZoneCreateProofByIdsInvocation {
            receiver: node_id.into(),
            ids: ids.clone(),
            resource_address,
        })
    }

    pub fn sys_push<P: Into<Proof>, Y, E: Debug + ScryptoCategorize + ScryptoDecode>(
        proof: P,
        env: &mut Y,
    ) -> Result<(), E>
    where
        Y: EngineApi<E> + Invokable<AuthZonePushInvocation, E>,
    {
        let node_id = Self::auth_zone_node_id(env).expect("Auth Zone doesn't exist");
        let proof: Proof = proof.into();

        env.invoke(AuthZonePushInvocation {
            receiver: node_id.into(),
            proof,
        })
    }

    pub fn sys_assert_access_rule<Y, E>(access_rule: AccessRule, env: &mut Y) -> Result<(), E>
    where
        Y: EngineApi<E> + Invokable<AuthZoneAssertAccessRuleInvocation, E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let node_id = Self::auth_zone_node_id(env).expect("Auth Zone doesn't exist");
        env.invoke(AuthZoneAssertAccessRuleInvocation {
            receiver: node_id.into(),
            access_rule,
        })
    }

    fn auth_zone_node_id<Y, E>(api: &mut Y) -> Option<RENodeId>
    where
        Y: EngineApi<E>,
        E: Debug + ScryptoCategorize + ScryptoDecode,
    {
        let owned_node_ids = api.sys_get_visible_nodes().unwrap();
        owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
    }
}
