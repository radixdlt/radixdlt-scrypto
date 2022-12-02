use sbor::*;

#[derive(Encode, Decode, TypeId, Copy, Clone, Debug)]
pub enum TimePrecision {
    Minute,
}

#[derive(Encode, Decode, TypeId, Copy, Clone, Debug)]
pub struct Instant {
    pub seconds_since_unix_epoch: u64,
}

impl Instant {
    pub fn new(seconds_since_unix_epoch: u64) -> Instant {
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

    pub fn add_days(&self, days_to_add: u64) -> Option<Instant> {
        self.seconds_since_unix_epoch
            .checked_add(days_to_add * 24 * 60 * 60)
            .map(Instant::new)
    }

    pub fn add_hours(&self, hours_to_add: u64) -> Option<Instant> {
        self.seconds_since_unix_epoch
            .checked_add(hours_to_add * 60 * 60)
            .map(Instant::new)
    }

    pub fn add_minutes(&self, minutes_to_add: u64) -> Option<Instant> {
        self.seconds_since_unix_epoch
            .checked_add(minutes_to_add * 60)
            .map(Instant::new)
    }

    pub fn add_seconds(&self, seconds_to_add: u64) -> Option<Instant> {
        self.seconds_since_unix_epoch
            .checked_add(seconds_to_add)
            .map(Instant::new)
    }
}

#[derive(Encode, Decode, TypeId, Copy, Clone, Debug)]
pub enum TimeComparisonOperator {
    Eq,
    Lt,
    Lte,
    Gt,
    Gte,
}
