use crate::api::types::AccessRule;
use crate::*;

/// An enum of the roles in the Access Controller component
pub enum Role {
    Primary,
    Recovery,
    Confirmation,
}

/// A struct with the set of rule associated with each role - used when creating a new access
/// controller for the initial rules and also used during recovery for proposing a rule set.
#[derive(Debug, Clone, Eq, PartialEq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct RuleSet {
    pub primary: AccessRule,
    pub recovery: AccessRule,
    pub confirmation: AccessRule,
}
