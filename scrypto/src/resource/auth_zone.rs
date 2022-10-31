use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::engine::{api::*, types::*, utils::*};
use crate::math::Decimal;
use crate::resource::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZonePopInput {
    pub auth_zone_id: AuthZoneId,
}

impl SysInvocation for AuthZonePopInput {
    type Output = scrypto::resource::Proof;

    fn native_method() -> NativeMethod {
        NativeMethod::AuthZone(AuthZoneMethod::Pop)
    }
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZonePushInput {
    pub auth_zone_id: AuthZoneId,
    pub proof: Proof,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofInput {
    pub auth_zone_id: AuthZoneId,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByAmountInput {
    pub auth_zone_id: AuthZoneId,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByIdsInput {
    pub auth_zone_id: AuthZoneId,
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneClearInput {
    pub auth_zone_id: AuthZoneId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneDrainInput {
    pub auth_zone_id: AuthZoneId,
}

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct ComponentAuthZone {}

impl ComponentAuthZone {
    #[cfg(target_arch = "wasm32")]
    pub fn pop() -> Proof {
        Self::sys_pop(&mut Syscalls).unwrap()
    }

    pub fn sys_pop<Y, E: Debug + TypeId + Decode>(sys_calls: &mut Y) -> Result<Proof, E>
    where
        Y: ScryptoSyscalls<E> + SysInvokable<AuthZonePopInput, E>,
    {
        let owned_node_ids = sys_calls.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        sys_calls.sys_invoke(AuthZonePopInput {
            auth_zone_id: node_id.into(),
        })
    }

    #[cfg(target_arch = "wasm32")]
    pub fn create_proof(resource_address: ResourceAddress) -> Proof {
        Self::sys_create_proof(resource_address, &mut Syscalls).unwrap()
    }

    pub fn sys_create_proof<Y, E: Debug + TypeId + Decode>(
        resource_address: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ScryptoSyscalls<E>,
    {
        let owned_node_ids = sys_calls.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        sys_calls.sys_invoke_native_method(
            NativeMethod::AuthZone(AuthZoneMethod::CreateProof),
            &AuthZoneCreateProofInput {
                auth_zone_id: node_id.into(),
                resource_address,
            },
        )
    }

    #[cfg(target_arch = "wasm32")]
    pub fn create_proof_by_amount(amount: Decimal, resource_address: ResourceAddress) -> Proof {
        Self::sys_create_proof_by_amount(amount, resource_address, &mut Syscalls).unwrap()
    }

    pub fn sys_create_proof_by_amount<Y, E: Debug + TypeId + Decode>(
        amount: Decimal,
        resource_address: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ScryptoSyscalls<E>,
    {
        let owned_node_ids = sys_calls.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        sys_calls.sys_invoke_native_method(
            NativeMethod::AuthZone(AuthZoneMethod::CreateProofByAmount),
            &AuthZoneCreateProofByAmountInput {
                auth_zone_id: node_id.into(),
                amount,
                resource_address,
            },
        )
    }

    #[cfg(target_arch = "wasm32")]
    pub fn create_proof_by_ids(
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> Proof {
        Self::sys_create_proof_by_ids(ids, resource_address, &mut Syscalls).unwrap()
    }

    pub fn sys_create_proof_by_ids<Y, E: Debug + TypeId + Decode>(
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
        sys_calls: &mut Y,
    ) -> Result<Proof, E>
    where
        Y: ScryptoSyscalls<E>,
    {
        let owned_node_ids = sys_calls.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        sys_calls.sys_invoke_native_method(
            NativeMethod::AuthZone(AuthZoneMethod::CreateProofByIds),
            &AuthZoneCreateProofByIdsInput {
                auth_zone_id: node_id.into(),
                ids: ids.clone(),
                resource_address,
            },
        )
    }

    #[cfg(target_arch = "wasm32")]
    pub fn push<P: Into<Proof>>(proof: P) {
        Self::sys_push(proof, &mut Syscalls).unwrap()
    }

    pub fn sys_push<P: Into<Proof>, Y, E: Debug + TypeId + Decode>(
        proof: P,
        sys_calls: &mut Y,
    ) -> Result<(), E>
    where
        Y: ScryptoSyscalls<E>,
    {
        let owned_node_ids = sys_calls.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");

        let proof: Proof = proof.into();

        sys_calls.sys_invoke_native_method(
            NativeMethod::AuthZone(AuthZoneMethod::Push),
            &AuthZonePushInput {
                auth_zone_id: node_id.into(),
                proof,
            },
        )
    }
}
