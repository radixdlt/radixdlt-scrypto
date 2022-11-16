use radix_engine_lib::data::ScryptoCustomTypeId;
use radix_engine_lib::engine::api::{SysNativeInvokable, Syscalls};
use radix_engine_lib::engine::types::RENodeId;
use radix_engine_lib::math::Decimal;
use radix_engine_lib::resource::{
    AuthZoneClearInvocation, AuthZoneCreateProofByAmountInvocation,
    AuthZoneCreateProofByIdsInvocation, AuthZoneCreateProofInvocation, AuthZoneDrainInvocation,
    AuthZonePopInvocation, AuthZonePushInvocation, NonFungibleId, ResourceAddress,
};
use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;
use scrypto::engine::scrypto_env::ScryptoEnv;

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct ComponentAuthZone {}

impl ComponentAuthZone {
    pub fn sys_drain<Y, E: Debug + TypeId<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>>(
        env: &mut Y,
    ) -> Result<Vec<radix_engine_lib::resource::Proof>, E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZoneDrainInvocation, E>,
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

    pub fn sys_clear<Y, E: Debug + TypeId<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>>(
        env: &mut Y,
    ) -> Result<(), E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZoneClearInvocation, E>,
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

    pub fn sys_pop<Y, E: Debug + TypeId<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>>(
        env: &mut Y,
    ) -> Result<radix_engine_lib::resource::Proof, E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZonePopInvocation, E>,
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

    pub fn sys_create_proof<
        Y,
        E: Debug + TypeId<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    >(
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<radix_engine_lib::resource::Proof, E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZoneCreateProofInvocation, E>,
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

    pub fn sys_create_proof_by_amount<
        Y,
        E: Debug + TypeId<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    >(
        amount: Decimal,
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<radix_engine_lib::resource::Proof, E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZoneCreateProofByAmountInvocation, E>,
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

    pub fn sys_create_proof_by_ids<
        Y,
        E: Debug + TypeId<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    >(
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        env: &mut Y,
    ) -> Result<radix_engine_lib::resource::Proof, E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZoneCreateProofByIdsInvocation, E>,
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

    pub fn sys_push<
        P: Into<radix_engine_lib::resource::Proof>,
        Y,
        E: Debug + TypeId<ScryptoCustomTypeId> + Decode<ScryptoCustomTypeId>,
    >(
        proof: P,
        env: &mut Y,
    ) -> Result<(), E>
    where
        Y: Syscalls<E> + SysNativeInvokable<AuthZonePushInvocation, E>,
    {
        let owned_node_ids = env.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");

        let proof: radix_engine_lib::resource::Proof = proof.into();

        env.sys_invoke(AuthZonePushInvocation {
            receiver: node_id.into(),
            proof,
        })
    }

    pub fn pop() -> radix_engine_lib::resource::Proof {
        Self::sys_pop(&mut ScryptoEnv).unwrap()
    }

    pub fn create_proof(resource_address: ResourceAddress) -> radix_engine_lib::resource::Proof {
        Self::sys_create_proof(resource_address, &mut ScryptoEnv).unwrap()
    }

    pub fn create_proof_by_amount(
        amount: Decimal,
        resource_address: ResourceAddress,
    ) -> radix_engine_lib::resource::Proof {
        Self::sys_create_proof_by_amount(amount, resource_address, &mut ScryptoEnv).unwrap()
    }

    pub fn create_proof_by_ids(
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> radix_engine_lib::resource::Proof {
        Self::sys_create_proof_by_ids(ids, resource_address, &mut ScryptoEnv).unwrap()
    }

    pub fn push<P: Into<radix_engine_lib::resource::Proof>>(proof: P) {
        Self::sys_push(proof, &mut ScryptoEnv).unwrap()
    }
}
