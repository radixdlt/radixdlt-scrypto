use radix_engine_interface::wasm::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

blueprint! {
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

        pub fn get_current_time_rounded_to_minutes() -> u64 {
            Clock::current_time_rounded_to_minutes().seconds_since_unix_epoch
        }

        pub fn set_current_time(clock: SystemAddress, current_time_ms: u64) {
            let input = RadixEngineInput::Invoke(SerializedInvocation::Native(
                NativeFnInvocation::Method(NativeMethodInvocation::Clock(
                    ClockMethodInvocation::SetCurrentTime(ClockSetCurrentTimeInvocation {
                        receiver: clock,
                        current_time_ms,
                    }),
                )),
            ));
            call_engine(input)
        }
    }
}
