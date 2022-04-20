use crate::args;
use crate::buffer::scrypto_decode;
use crate::core::SNodeRef;
use crate::engine::{api::*, call_engine};
use crate::math::Decimal;
use crate::resource::*;
use crate::rust::collections::BTreeSet;
use crate::rust::string::ToString;

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct ComponentAuthZone {}

impl ComponentAuthZone {
    /// Pushes a proof to the auth zone.
    pub fn push(proof: Proof) {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::AuthZone,
            function: "push".to_string(),
            args: args![proof],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    /// Pops the most recently added proof from the auth zone.
    pub fn pop() -> Proof {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::AuthZone,
            function: "pop".to_string(),
            args: args![],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn create_proof(resource_address: ResourceAddress) -> Proof {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::AuthZone,
            function: "create_proof".to_string(),
            args: args![resource_address],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn create_proof_by_amount(amount: Decimal, resource_address: ResourceAddress) -> Proof {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::AuthZone,
            function: "create_proof_by_amount".to_string(),
            args: args![amount, resource_address],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }

    pub fn create_proof_by_ids(
        ids: &BTreeSet<NonFungibleId>,
        resource_address: ResourceAddress,
    ) -> Proof {
        let input = InvokeSNodeInput {
            snode_ref: SNodeRef::AuthZone,
            function: "create_proof_by_ids".to_string(),
            args: args![ids.clone(), resource_address],
        };
        let output: InvokeSNodeOutput = call_engine(INVOKE_SNODE, input);
        scrypto_decode(&output.rtn).unwrap()
    }
}
