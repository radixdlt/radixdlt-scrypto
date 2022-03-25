use crate::engine::{api::*, call_engine};
use crate::math::Decimal;
use crate::resource::*;
use crate::rust::collections::BTreeSet;

/// Represents the auth zone, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
pub struct AuthZone {}

impl AuthZone {
    /// Pushes a proof to the auth zone.
    pub fn push(proof: Proof) {
        let input = PushOntoAuthZoneInput { proof_id: proof.0 };
        let _: PushOntoAuthZoneOutput = call_engine(PUSH_ONTO_AUTH_ZONE, input);
    }

    /// Pops the most recently added proof from the auth zone.
    pub fn pop() -> Proof {
        let input = PopFromAuthZoneInput {};
        let output: PopFromAuthZoneOutput = call_engine(POP_FROM_AUTH_ZONE, input);

        Proof(output.proof_id.into())
    }

    pub fn create_proof(resource_def_id: ResourceDefId) -> Proof {
        let input = CreateAuthZoneProofInput { resource_def_id };
        let output: CreateAuthZoneProofOutput = call_engine(CREATE_AUTH_ZONE_PROOF, input);

        Proof(output.proof_id.into())
    }

    pub fn create_proof_by_amount(amount: Decimal, resource_def_id: ResourceDefId) -> Proof {
        let input = CreateAuthZoneProofByAmountInput {
            resource_def_id,
            amount,
        };
        let output: CreateAuthZoneProofByAmountOutput =
            call_engine(CREATE_AUTH_ZONE_PROOF_BY_AMOUNT, input);

        Proof(output.proof_id.into())
    }

    pub fn create_proof_by_ids(
        ids: &BTreeSet<NonFungibleId>,
        resource_def_id: ResourceDefId,
    ) -> Proof {
        let input = CreateAuthZoneProofByIdsInput {
            resource_def_id,
            ids: ids.clone(),
        };
        let output: CreateAuthZoneProofByIdsOutput =
            call_engine(CREATE_AUTH_ZONE_PROOF_BY_IDS, input);

        Proof(output.proof_id.into())
    }
}
