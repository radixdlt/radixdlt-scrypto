use sbor::rust::collections::BTreeSet;
use sbor::rust::vec::Vec;
use sbor::*;

use crate::engine::{api::*, types::*, utils::*};
use crate::math::Decimal;
use crate::resource::*;

#[derive(Debug, TypeId, Encode, Decode)]
pub struct AuthZonePopInput {
    pub auth_zone_id: AuthZoneId,
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
    pub fn pop() -> Proof {
        let input = RadixEngineInput::GetVisibleNodeIds();
        let owned_node_ids: Vec<RENodeId> = call_engine(input);
        let node_id = owned_node_ids
            .into_iter()
            .find(|n| matches!(n, RENodeId::AuthZoneStack(..)))
            .expect("AuthZone does not exist");

        let input = RadixEngineInput::InvokeNativeMethod(
            NativeMethod::AuthZone(AuthZoneMethod::Pop),
            node_id,
            scrypto::buffer::scrypto_encode(
                &(AuthZonePopInput {
                    auth_zone_id: node_id.into(),
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
            node_id,
            scrypto::buffer::scrypto_encode(
                &(AuthZoneCreateProofInput {
                    auth_zone_id: node_id.into(),
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
            node_id,
            scrypto::buffer::scrypto_encode(
                &(AuthZoneCreateProofByAmountInput {
                    amount,
                    auth_zone_id: node_id.into(),
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
            node_id,
            scrypto::buffer::scrypto_encode(
                &(AuthZoneCreateProofByIdsInput {
                    ids: ids.clone(),
                    auth_zone_id: node_id.into(),
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
            node_id,
            scrypto::buffer::scrypto_encode(
                &(AuthZonePushInput {
                    proof,
                    auth_zone_id: node_id.into(),
                }),
            ),
        );
        call_engine(input)
    }
}
