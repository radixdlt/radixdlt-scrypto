use crate::engine::{api::*, call_engine};
use crate::resource::*;

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
}
