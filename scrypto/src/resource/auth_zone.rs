use sbor::rust::collections::BTreeSet;
use sbor::rust::fmt::Debug;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::engine::{api::*, types::*, utils::*};
use crate::math::Decimal;
use crate::native_methods;
use crate::resource::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZonePopInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZonePushInput {
    pub proof: Proof,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofInput {
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByAmountInput {
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByIdsInput {
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneClearInput {}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneDrainInput {}

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
        Y: ScryptoSyscalls<E>,
    {
        let owned_node_ids = sys_calls.sys_get_visible_nodes()?;
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");
        sys_calls.sys_invoke_native_method(
            NativeMethod::AuthZone(AuthZoneMethod::Pop),
            node_id,
            scrypto::buffer::scrypto_encode(&(AuthZonePopInput {})),
        )
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
            node_id,
            scrypto::buffer::scrypto_encode(&(AuthZoneCreateProofInput { resource_address })),
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
            node_id,
            scrypto::buffer::scrypto_encode(
                &(AuthZoneCreateProofByAmountInput {
                    amount,
                    resource_address,
                }),
            ),
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
            node_id,
            scrypto::buffer::scrypto_encode(
                &(AuthZoneCreateProofByIdsInput {
                    ids: ids.clone(),
                    resource_address,
                }),
            ),
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
            node_id,
            scrypto::buffer::scrypto_encode(&(AuthZonePushInput { proof })),
        )
    }
}
