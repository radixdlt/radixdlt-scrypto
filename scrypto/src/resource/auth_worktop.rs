use crate::engine::{api::*, call_engine};
use crate::resource::*;

/// Represents the auth worktop, which is used by system for checking
/// if this component is allowed to
///
/// 1. Call methods on another component;
/// 2. Access resource system.
///
/// TODO: rename
pub struct AuthWorktop {}

impl AuthWorktop {
    /// Pushes a proof to the auth worktop.
    pub fn push(proof: Proof) {
        let input = PushOntoAuthWorktopInput { proof_id: proof.0 };
        let _: PushOntoAuthWorktopOutput = call_engine(PUSH_ONTO_AUTH_WORKTOP, input);
    }

    /// Pops the most recently added proof from the auth worktop.
    pub fn pop() -> Proof {
        let input = PopFromAuthWorktopInput {};
        let output: PopFromAuthWorktopOutput = call_engine(POP_FROM_AUTH_WORKTOP, input);

        Proof(output.proof_id.into())
    }
}
