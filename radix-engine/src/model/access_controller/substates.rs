use crate::types::*;
use radix_engine_interface::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq, ScryptoCategorize, ScryptoEncode, ScryptoDecode)]
pub struct AccessControllerSubstate {
    /// A vault where the asset controlled by the access controller lives.
    pub controlled_asset: VaultId,

    /// Maps the role proposing the rule set changes to their proposed rule set and a timestamp of
    /// when the recovery was initiated. Since [`Proposer`] is used as the key here, we can have a
    /// maximum of two entries in this [`HashMap`] at any given time.
    pub ongoing_recoveries: Option<HashMap<Proposer, (RuleSet, Instant)>>,

    /// The amount of time (in hours) that it takes for timed recovery to be done. Maximum is 65,535
    /// hours which is 7.48 years.
    pub timed_recovery_delay_in_hours: u16,

    /// A boolean of whether the primary role is locked or not.
    pub is_primary_role_locked: bool,
}
