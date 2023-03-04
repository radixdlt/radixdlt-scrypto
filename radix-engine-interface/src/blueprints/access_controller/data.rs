use crate::blueprints::resource::AccessRule;
use crate::*;

/// An enum of the roles in the Access Controller component
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, ScryptoSbor, Hash)]
pub enum Role {
    Primary,
    Recovery,
    Confirmation,
}

/// The set of roles allowed to propose recoveries. Only Primary and Recovery roles can initiate,
/// or propose recoveries, Confirmation can't initiate nor propose.
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq, ScryptoSbor, Hash)]
pub enum Proposer {
    Primary,
    Recovery,
}

impl From<Proposer> for Role {
    fn from(value: Proposer) -> Self {
        match value {
            Proposer::Primary => Role::Primary,
            Proposer::Recovery => Role::Recovery,
        }
    }
}

/// A struct with the set of rule associated with each role - used when creating a new access
/// controller for the initial rules and also used during recovery for proposing a rule set.
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct RuleSet {
    pub primary_role: AccessRule,
    pub recovery_role: AccessRule,
    pub confirmation_role: AccessRule,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor)]
pub struct RecoveryProposal {
    /// The set of rules being proposed for the different roles.
    pub rule_set: RuleSet,

    /// The proposed delay of timed recoveries.
    pub timed_recovery_delay_in_minutes: Option<u32>,
}
