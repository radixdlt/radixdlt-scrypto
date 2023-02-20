use crate::time::constants::*;
use sbor::*;

/// Represents a Unix timestamp, capturing the seconds since the unix epoch.
///
/// See also the [`UtcDateTime`](super::UtcDateTime) type which supports conversion to/from `Instant`.
#[derive(Sbor, Copy, Clone, Debug, Eq, PartialEq)]
pub struct Instant {
    pub seconds_since_unix_epoch: i64,
}

impl Instant {
    pub fn new(seconds_since_unix_epoch: i64) -> Instant {
        Instant {
            seconds_since_unix_epoch,
        }
    }

    pub fn compare(&self, other: Instant, operator: TimeComparisonOperator) -> bool {
        let self_seconds = self.seconds_since_unix_epoch;
        let other_seconds = other.seconds_since_unix_epoch;
        match operator {
            TimeComparisonOperator::Eq => self_seconds == other_seconds,
            TimeComparisonOperator::Lt => self_seconds < other_seconds,
            TimeComparisonOperator::Lte => self_seconds <= other_seconds,
            TimeComparisonOperator::Gt => self_seconds > other_seconds,
            TimeComparisonOperator::Gte => self_seconds >= other_seconds,
        }
    }

    pub fn add_days(&self, days_to_add: i64) -> Option<Instant> {
        days_to_add
            .checked_mul(SECONDS_IN_A_DAY)
            .and_then(|to_add| self.seconds_since_unix_epoch.checked_add(to_add))
            .map(Instant::new)
    }

    pub fn add_hours(&self, hours_to_add: i64) -> Option<Instant> {
        hours_to_add
            .checked_mul(SECONDS_IN_AN_HOUR)
            .and_then(|to_add| self.seconds_since_unix_epoch.checked_add(to_add))
            .map(Instant::new)
    }

    pub fn add_minutes(&self, minutes_to_add: i64) -> Option<Instant> {
        minutes_to_add
            .checked_mul(SECONDS_IN_A_MINUTE)
            .and_then(|to_add| self.seconds_since_unix_epoch.checked_add(to_add))
            .map(Instant::new)
    }

    pub fn add_seconds(&self, seconds_to_add: i64) -> Option<Instant> {
        self.seconds_since_unix_epoch
            .checked_add(seconds_to_add)
            .map(Instant::new)
    }
}

#[derive(Sbor, Copy, Clone, Debug, Eq, PartialEq)]
pub enum TimeComparisonOperator {
    Eq,
    Lt,
    Lte,
    Gt,
    Gte,
}
