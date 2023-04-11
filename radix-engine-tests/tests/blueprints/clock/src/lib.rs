use scrypto::api::*;
use scrypto::blueprints::clock::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

#[blueprint]
mod clock_test {
    struct ClockTest;

    impl ClockTest {
        pub fn test_clock_comparison_operators() {
            // Check against the current time
            let current_time = Clock::current_time_rounded_to_minutes();

            assert!(!Clock::current_time_is_strictly_before(
                current_time,
                TimePrecision::Minute
            ));
            assert!(Clock::current_time_is_at_or_before(
                current_time,
                TimePrecision::Minute
            ));
            assert!(!Clock::current_time_is_strictly_after(
                current_time,
                TimePrecision::Minute
            ));
            assert!(Clock::current_time_is_at_or_after(
                current_time,
                TimePrecision::Minute
            ));

            // Check against a future time (also after rounding)
            let time_in_the_future = Clock::current_time_rounded_to_minutes()
                .add_seconds(60)
                .unwrap();

            assert!(Clock::current_time_is_strictly_before(
                time_in_the_future,
                TimePrecision::Minute
            ));

            assert!(Clock::current_time_is_at_or_before(
                time_in_the_future,
                TimePrecision::Minute
            ));
            assert!(!Clock::current_time_is_strictly_after(
                time_in_the_future,
                TimePrecision::Minute
            ));
            assert!(!Clock::current_time_is_at_or_after(
                time_in_the_future,
                TimePrecision::Minute
            ));

            // Check against a future time, but the same after rounding to minutes
            let time_in_the_future = Clock::current_time_rounded_to_minutes()
                .add_seconds(59)
                .unwrap();

            assert!(!Clock::current_time_is_strictly_before(
                time_in_the_future,
                TimePrecision::Minute
            ));
            assert!(Clock::current_time_is_at_or_before(
                time_in_the_future,
                TimePrecision::Minute
            ));
            assert!(!Clock::current_time_is_strictly_after(
                time_in_the_future,
                TimePrecision::Minute
            ));
            assert!(Clock::current_time_is_at_or_after(
                time_in_the_future,
                TimePrecision::Minute
            ));

            // Check against a past time
            let time_in_the_past = Instant::new(
                Clock::current_time_rounded_to_minutes().seconds_since_unix_epoch - 60,
            );

            assert!(!Clock::current_time_is_strictly_before(
                time_in_the_past,
                TimePrecision::Minute
            ));
            assert!(!Clock::current_time_is_at_or_before(
                time_in_the_past,
                TimePrecision::Minute
            ));
            assert!(Clock::current_time_is_strictly_after(
                time_in_the_past,
                TimePrecision::Minute
            ));
            assert!(Clock::current_time_is_at_or_after(
                time_in_the_past,
                TimePrecision::Minute
            ));
        }

        pub fn test_date_time_conversions() {
            let now = Clock::current_time_rounded_to_minutes();
            let dt = UtcDateTime::try_from(now).unwrap();
            assert!(dt.to_instant() == now);

            let now_plus_2d = now.add_days(2).unwrap();
            let dt_plus_2d = dt.add_days(2).unwrap();
            let dt_instant_plus_2d = dt_plus_2d.to_instant();

            assert!(dt_instant_plus_2d == Instant::new(now.seconds_since_unix_epoch + 172800));
            assert!(now_plus_2d == Instant::new(now.seconds_since_unix_epoch + 172800));
        }

        pub fn get_current_time_rounded_to_minutes() -> i64 {
            Clock::current_time_rounded_to_minutes().seconds_since_unix_epoch
        }

        pub fn set_current_time(clock: ComponentAddress, current_time_ms: i64) {
            ScryptoEnv
                .call_method(
                    &clock.into(),
                    CLOCK_SET_CURRENT_TIME_IDENT,
                    scrypto_encode(&ClockSetCurrentTimeInput { current_time_ms }).unwrap(),
                )
                .unwrap();
        }
    }
}
