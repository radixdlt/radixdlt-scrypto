use crate::types::*;
use radix_engine_interface::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerSubstate {
    /// A vault of the badge that the controller protects and controls.
    controlled_asset: Own,

    /// The currently active set of role rules. Used for asserting that the caller indeed fulfills
    /// role X.
    active_rule_set: RuleSet,

    // TODO: KVStore would be better here but we need the ability to just delete recoveries once one
    // of them is completed successfully. Migrate to KVStore once it has this ability.
    /// Maps the proposed rule set to the role that proposed at and when it was proposed.
    ongoing_recoveries: Option<HashMap<RuleSet, (Role, Instant)>>,

    /// The amount of time (in hours) that it takes for timed recovery to be done.
    timed_recovery_delay_in_hours: u16,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    PartialOrd,
    Eq,
    Ord,
    ScryptoCategorize,
    ScryptoEncode,
    ScryptoDecode,
    Hash,
)]
pub struct RuleSet {
    primary_role: AccessRule,
    recovery_role: AccessRule,
    confirmation_role: AccessRule,
}

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub enum Role {
    Primary,
    Recovery,
    Confirmation,
}
