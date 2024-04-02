use radix_common::constants::CONSENSUS_MANAGER;
use radix_common::data::scrypto::{scrypto_decode, scrypto_encode};
use radix_common::time::*;
use radix_engine_interface::blueprints::consensus_manager::{
    ConsensusManagerCompareCurrentTimeInputV2, ConsensusManagerGetCurrentTimeInputV2,
    TimePrecision, CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT,
    CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT,
};
use sbor::rust::fmt::Debug;
use scrypto::engine::scrypto_env::ScryptoVmV1Api;

/// The system clock
#[derive(Debug)]
pub struct Clock {}

impl Clock {
    /// Returns the current timestamp (in seconds)
    pub fn current_time_rounded_to_seconds() -> Instant {
        Self::current_time(TimePrecision::Second)
    }

    /// Returns the current timestamp (in seconds), rounded down to minutes
    pub fn current_time_rounded_to_minutes() -> Instant {
        Self::current_time(TimePrecision::Minute)
    }

    /// Returns the current timestamp (in seconds), rounded down to the specified precision
    pub fn current_time(precision: TimePrecision) -> Instant {
        let rtn = ScryptoVmV1Api::object_call(
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_GET_CURRENT_TIME_IDENT,
            scrypto_encode(&ConsensusManagerGetCurrentTimeInputV2 { precision }).unwrap(),
        );
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
        let rtn = ScryptoVmV1Api::object_call(
            CONSENSUS_MANAGER.as_node_id(),
            CONSENSUS_MANAGER_COMPARE_CURRENT_TIME_IDENT,
            scrypto_encode(&ConsensusManagerCompareCurrentTimeInputV2 {
                instant,
                precision,
                operator,
            })
            .unwrap(),
        );

        scrypto_decode(&rtn).unwrap()
    }
}
