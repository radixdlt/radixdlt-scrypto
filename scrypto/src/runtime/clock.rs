use radix_engine_interface::api::ClientObjectApi;
use radix_engine_interface::blueprints::clock::{
    ClockCompareCurrentTimeInput, ClockGetCurrentTimeInput, TimePrecision,
    CLOCK_COMPARE_CURRENT_TIME_IDENT, CLOCK_GET_CURRENT_TIME_IDENT,
};
use radix_engine_interface::constants::CLOCK;
use radix_engine_interface::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_engine_interface::time::*;
use sbor::rust::fmt::Debug;
use scrypto::engine::scrypto_env::ScryptoEnv;

/// The system clock
#[derive(Debug)]
pub struct Clock {}

impl Clock {
    /// Returns the current timestamp (in seconds), rounded down to minutes
    pub fn current_time_rounded_to_minutes() -> Instant {
        Self::current_time(TimePrecision::Minute)
    }

    /// Returns the current timestamp (in seconds), rounded down to the specified precision
    pub fn current_time(precision: TimePrecision) -> Instant {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                CLOCK.as_node_id(),
                CLOCK_GET_CURRENT_TIME_IDENT,
                scrypto_encode(&ClockGetCurrentTimeInput { precision }).unwrap(),
            )
            .unwrap();
        scrypto_decode(&rtn).unwrap()
    }

    /// Returns true if current time, rounded down to a given precision,
    /// is strictly before the specified instant (also rounded down to a given precision), false otherwise.
    pub fn current_time_is_strictly_before(instant: Instant, precision: TimePrecision) -> bool {
        Self::current_time_comparison(instant, precision, TimeComparisonOperator::Lt)
    }

    /// Returns true if current time, rounded down to a given precision,
    /// is before or equal to the specified instant (also rounded down to a given precision), false otherwise.
    pub fn current_time_is_at_or_before(instant: Instant, precision: TimePrecision) -> bool {
        Self::current_time_comparison(instant, precision, TimeComparisonOperator::Lte)
    }

    /// Returns true if current time, rounded down to a given precision,
    /// is strictly after the specified instant (also rounded down to a given precision), false otherwise.
    pub fn current_time_is_strictly_after(instant: Instant, precision: TimePrecision) -> bool {
        Self::current_time_comparison(instant, precision, TimeComparisonOperator::Gt)
    }

    /// Returns true if current time, rounded down to a given precision,
    /// is after or equal to the specified instant (also rounded down to a given precision), false otherwise.
    pub fn current_time_is_at_or_after(instant: Instant, precision: TimePrecision) -> bool {
        Self::current_time_comparison(instant, precision, TimeComparisonOperator::Gte)
    }

    /// Returns true if current time, rounded down to a given precision,
    /// matches the given comparison operator against
    /// the specified instant (also rounded down to a given precision), false otherwise.
    pub fn current_time_comparison(
        instant: Instant,
        precision: TimePrecision,
        operator: TimeComparisonOperator,
    ) -> bool {
        let mut env = ScryptoEnv;
        let rtn = env
            .call_method(
                CLOCK.as_node_id(),
                CLOCK_COMPARE_CURRENT_TIME_IDENT,
                scrypto_encode(&ClockCompareCurrentTimeInput {
                    instant,
                    precision,
                    operator,
                })
                .unwrap(),
            )
            .unwrap();

        scrypto_decode(&rtn).unwrap()
    }
}
