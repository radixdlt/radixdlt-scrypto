use crate::args;
use sbor::*;
use crate::buffer::scrypto_decode;
use crate::core::SNodeRef;
use crate::engine::{api::*, call_engine};
use crate::math::Decimal;
use crate::resource::*;
use crate::rust::collections::BTreeSet;
use crate::rust::string::ToString;


#[derive(Debug, TypeId, Encode, Decode)]
pub enum AuthZoneMethod {
    Push(Proof),
    Pop(),
    Clear(),
    CreateProof(ResourceAddress),
    CreateProofByAmount(Decimal, ResourceAddress),
    CreateProofByIds(BTreeSet<NonFungibleId>, ResourceAddress),
}

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct AuthZone {}

impl AuthZone {
    /// Pushes a proof to the auth zone.
    pub fn push(proof: Proof) {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::AuthZoneRef,
            function: "main".to_string(),
            args: args![AuthZoneMethod::Push(proof)],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Pops the most recently added proof from the auth zone.
    pub fn pop() -> Proof {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::AuthZoneRef,
            function: "main".to_string(),
            args: args![AuthZoneMethod::Pop()],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn create_proof(resource_address: ResourceAddress) -> Proof {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::AuthZoneRef,
            function: "main".to_string(),
            args: args![AuthZoneMethod::CreateProof(resource_address)],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn create_proof_by_amount(amount: Decimal, resource_address: ResourceAddress) -> Proof {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::AuthZoneRef,
            function: "main".to_string(),
            args: args![AuthZoneMethod::CreateProofByAmount(amount, resource_address)],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn create_proof_by_ids(
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> Proof {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::AuthZoneRef,
            function: "main".to_string(),
            args: args![AuthZoneMethod::CreateProofByIds(ids.clone(), resource_address)],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }
}
