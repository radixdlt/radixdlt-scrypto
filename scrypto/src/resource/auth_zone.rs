use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::engine::{api::*, types::*, utils::*};
use crate::math::Decimal;
use crate::resource::*;
use crate::scrypto;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZonePopInvocation {
    pub receiver: AuthZoneId,
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZonePushInvocation {
    pub receiver: AuthZoneId,
    pub proof: Proof,
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofInvocation {
    pub receiver: AuthZoneId,
    pub resource_address: ResourceAddress,
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByAmountInvocation {
    pub receiver: AuthZoneId,
    pub amount: Decimal,
    pub resource_address: ResourceAddress,
}

#[derive(Debug)]
#[scrypto(TypeId, Encode, Decode)]
pub struct AuthZoneCreateProofByIdsInvocation {
    pub receiver: AuthZoneId,
    pub ids: BTreeSet<NonFungibleId>,
    pub resource_address: ResourceAddress,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneClearInvocation {
    pub receiver: AuthZoneId,
}

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZoneDrainInvocation {
    pub receiver: AuthZoneId,
}

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct ComponentAuthZone {}

impl ComponentAuthZone {
    pub fn pop() -> Proof {
        let input = RadixEngineInput::GetVisibleNodeIds();
        let owned_node_ids: Vec<RENodeId> = call_engine(input);
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");

        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::AuthZone(AuthZoneMethod::Pop),
            scrypto::buffer::scrypto_encode(
                &(AuthZonePopInvocation {
                    receiver: node_id.into(),
                }),
            ),
        );
        call_engine(input)
    }

    pub fn create_proof(resource_address: ResourceAddress) -> Proof {
        let input = RadixEngineInput::GetVisibleNodeIds();
        let owned_node_ids: Vec<RENodeId> = call_engine(input);
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");

        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::AuthZone(AuthZoneMethod::CreateProof),
            scrypto::buffer::scrypto_encode(
                &(AuthZoneCreateProofInvocation {
                    receiver: node_id.into(),
                    resource_address,
                }),
            ),
        );
        call_engine(input)
    }

    pub fn create_proof_by_amount(amount: Decimal, resource_address: ResourceAddress) -> Proof {
        let input = RadixEngineInput::GetVisibleNodeIds();
        let owned_node_ids: Vec<RENodeId> = call_engine(input);
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");

        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::AuthZone(AuthZoneMethod::CreateProofByAmount),
            scrypto::buffer::scrypto_encode(
                &(AuthZoneCreateProofByAmountInvocation {
                    amount,
                    receiver: node_id.into(),
                    resource_address,
                }),
            ),
        );
        call_engine(input)
    }

    pub fn create_proof_by_ids(
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> Proof {
        let input = RadixEngineInput::GetVisibleNodeIds();
        let owned_node_ids: Vec<RENodeId> = call_engine(input);
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");

        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::AuthZone(AuthZoneMethod::CreateProofByIds),
            scrypto::buffer::scrypto_encode(
                &(AuthZoneCreateProofByIdsInvocation {
                    ids: ids.clone(),
                    receiver: node_id.into(),
                    resource_address,
                }),
            ),
        );
        call_engine(input)
    }

    pub fn push<P: Into<Proof>>(proof: P) {
        let input = RadixEngineInput::GetVisibleNodeIds();
        let owned_node_ids: Vec<RENodeId> = call_engine(input);
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");

        let proof: Proof = proof.into();
        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::AuthZone(AuthZoneMethod::Push),
            scrypto::buffer::scrypto_encode(
                &(AuthZonePushInvocation {
                    proof,
                    receiver: node_id.into(),
                }),
            ),
        );
        call_engine(input)
    }
}
