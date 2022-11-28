use radix_engine_interface::api::wasm_input::*;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;

blueprint! {
    struct ClockTest;

    impl ClockTest {
        pub fn get_current_time_in_minutes() -> u64 {
            Runtime::current_time_in_minutes()
        }

        pub fn set_current_time(clock: SystemAddress, current_time_millis: u64) {
            let input = RadixEngineInput::InvokeNativeFn(NativeFnInvocation::Method(
                NativeMethodInvocation::Clock(ClockMethodInvocation::SetCurrentTime(
                    ClockSetCurrentTimeInvocation {
                        receiver: clock,
                        current_time_millis,
                    },
                )),
            ));
            call_engine(input)
        }
    }
}
