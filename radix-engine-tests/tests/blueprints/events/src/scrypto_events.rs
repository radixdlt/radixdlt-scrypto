use scrypto::prelude::*;

#[derive(ScryptoSbor)]
struct RegisteredEvent {
    number: u64,
}

#[derive(ScryptoSbor)]
struct UnregisteredEvent {
    number: u64,
}

#[blueprint]
#[events(RegisteredEvent)]
mod scrypto_events {
    struct ScryptoEvents;

    impl ScryptoEvents {
        pub fn emit_registered_event(number: u64) {
            Runtime::emit_event(RegisteredEvent { number });
        }

        pub fn emit_unregistered_event(number: u64) {
            Runtime::emit_event(UnregisteredEvent { number });
        }
    }
}
