use crate::api::types::AccessRule;
use crate::*;

/// An enum of the roles in the Access Controller component
#[derive(
    Debug,
    Clone,
    Copy,
    PartialOrd,
    PartialEq,
    Ord,
    Eq,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    Hash,
)]
pub enum Role {
    Primary,
    Recovery,
    Confirmation,
}

/// A struct with the set of rule associated with each role - used when creating a new access
/// controller for the initial rules and also used during recovery for proposing a rule set.
#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct RuleSet {
    pub primary_role: AccessRule,
    pub recovery_role: AccessRule,
    pub confirmation_role: AccessRule,
}
