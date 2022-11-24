use radix_engine_interface::api::api::{EngineApi, SysNativeInvokable};
use radix_engine_interface::api::types::RENodeId;
use radix_engine_interface::data::{ScryptoDecode, ScryptoTypeId};
use radix_engine_interface::math::Decimal;
use radix_engine_interface::model::*;
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use scrypto::engine::scrypto_env::ScryptoEnv;

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct ComponentAuthZone {}

impl ComponentAuthZone {
    pub fn sys_drain<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        env: &mut Y,
    ) -> Result<Vec<Proof>, E>
    where
        Y: EngineApi<E> + SysNativeInvokable<AuthZoneDrainInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.sys_invoke(AuthZoneDrainInvocation {
            receiver: node_id.into(),
        })
    }

    pub fn sys_clear<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(env: &mut Y) -> Result<(), E>
    where
        Y: EngineApi<E> + SysNativeInvokable<AuthZoneClearInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.sys_invoke(AuthZoneClearInvocation {
            receiver: node_id.into(),
        })
    }

    pub fn sys_pop<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(env: &mut Y) -> Result<Proof, E>
    where
        Y: EngineApi<E> + SysNativeInvokable<AuthZonePopInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.sys_invoke(AuthZonePopInvocation {
            receiver: node_id.into(),
        })
    }

    pub fn sys_create_proof<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: EngineApi<E> + SysNativeInvokable<AuthZoneCreateProofInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.sys_invoke(AuthZoneCreateProofInvocation {
            receiver: node_id.into(),
            resource_address,
        })
    }

    pub fn sys_create_proof_by_amount<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        amount: Decimal,
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: EngineApi<E> + SysNativeInvokable<AuthZoneCreateProofByAmountInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.sys_invoke(AuthZoneCreateProofByAmountInvocation {
            receiver: node_id.into(),
            amount,
            resource_address,
        })
    }

    pub fn sys_create_proof_by_ids<Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: EngineApi<E> + SysNativeInvokable<AuthZoneCreateProofByIdsInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        env.sys_invoke(AuthZoneCreateProofByIdsInvocation {
            receiver: node_id.into(),
            ids: ids.clone(),
            resource_address,
        })
    }

    pub fn sys_push<P: Into<Proof>, Y, E: Debug + ScryptoTypeId + ScryptoDecode>(
        proof: P,
        env: &mut Y,
    ) -> Result<(), E>
    where
        Y: EngineApi<E> + SysNativeInvokable<AuthZonePushInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");

        let proof: Proof = proof.into();

        env.sys_invoke(AuthZonePushInvocation {
            receiver: node_id.into(),
            proof,
        })
    }

    pub fn pop() -> Proof {
        Self::sys_pop(&mut ScryptoEnv).unwrap()
    }

    pub fn create_proof(resource_address: ResourceAddress) -> Proof {
        Self::sys_create_proof(resource_address, &mut ScryptoEnv).unwrap()
    }

    pub fn create_proof_by_amount(amount: Decimal, resource_address: ResourceAddress) -> Proof {
        Self::sys_create_proof_by_amount(amount, resource_address, &mut ScryptoEnv).unwrap()
    }

    pub fn create_proof_by_ids(
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> Proof {
        Self::sys_create_proof_by_ids(ids, resource_address, &mut ScryptoEnv).unwrap()
    }

    pub fn push<P: Into<Proof>>(proof: P) {
        Self::sys_push(proof, &mut ScryptoEnv).unwrap()
    }
}
