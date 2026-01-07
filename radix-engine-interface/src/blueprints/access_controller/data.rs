use crate::blueprints::resource::AccessRule;
use crate::internal_prelude::*;

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
#[cfg_attr(feature = "fuzzing", derive(::arbitrary::Arbitrary))]
#[derive(Debug, Clone, Eq, PartialEq, ScryptoSbor, ManifestSbor)]
pub struct RuleSet {
    pub primary_role: AccessRule,
    pub recovery_role: AccessRule,
    pub confirmation_role: AccessRule,
}

#[cfg_attr(
    feature = "fuzzing",
    derive(::arbitrary::Arbitrary, ::serde::Serialize, ::serde::Deserialize)
)]
#[derive(Debug, Clone, Eq, PartialEq, ManifestSbor, ScryptoDescribe)]
pub struct ManifestRuleSet {
    pub primary_role: ManifestAccessRule,
    pub recovery_role: ManifestAccessRule,
    pub confirmation_role: ManifestAccessRule,
}

impl From<RuleSet> for ManifestRuleSet {
    fn from(value: RuleSet) -> Self {
        Self {
            primary_role: value.primary_role.into(),
            recovery_role: value.recovery_role.into(),
            confirmation_role: value.confirmation_role.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoSbor, ManifestSbor)]
pub struct RecoveryProposal {
    /// The set of rules being proposed for the different roles.
    pub rule_set: RuleSet,

    /// The proposed delay of timed recoveries.
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, ManifestSbor, ScryptoDescribe)]
pub struct ManifestRecoveryProposal {
    /// The set of rules being proposed for the different roles.
    pub rule_set: ManifestRuleSet,

    /// The proposed delay of timed recoveries.
    pub timed_recovery_delay_in_minutes: Option<u32>,
}

impl From<RecoveryProposal> for ManifestRecoveryProposal {
    fn from(value: RecoveryProposal) -> Self {
        Self {
            rule_set: value.rule_set.into(),
            timed_recovery_delay_in_minutes: value.timed_recovery_delay_in_minutes,
        }
    }
}
