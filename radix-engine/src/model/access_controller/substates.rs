use crate::types::*;
use radix_engine_interface::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerSubstate {
    /// A vault of the badge that the controller protects and controls.
    controlled_asset: VaultId,

    /// The currently active set of role rules. Used for asserting that the caller indeed fulfills
    /// role X.
    active_rule_set: RuleSet,

    /// Maps the role proposing the rule set changes to their proposed rule set a timestamp of when
    /// the recovery was initiated. Since [`Role`] is used as the key here, we can have a maximum of
    /// three entries in this [`HashMap`] at any given time.
    ongoing_recoveries: Option<HashMap<Role, (RuleSet, Instant)>>,

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
