use radix_engine_interface::api::blueprints::resource::*;
use radix_engine_interface::api::node_modules::auth::*;
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::api::{EngineApi, Invokable};
use radix_engine_interface::data::{ScryptoCategorize, ScryptoDecode};
use radix_engine_interface::math::Decimal;
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
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.invoke(AuthZoneDrainInvocation {
            receiver: node_id.into(),
        })
    }

    pub fn sys_clear<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(env: &mut Y) -> Result<(), E>
    where
        Y: EngineApi<E> + Invokable<AuthZoneClearInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.invoke(AuthZoneClearInvocation {
            receiver: node_id.into(),
        })
    }

    pub fn sys_pop<Y, E: Debug + ScryptoCategorize + ScryptoDecode>(env: &mut Y) -> Result<Proof, E>
    where
        Y: EngineApi<E> + Invokable<AuthZonePopInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
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
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
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
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
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
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
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
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");

        let proof: Proof = proof.into();

        env.invoke(AuthZonePushInvocation {
            receiver: node_id.into(),
            proof,
        })
    }
}
