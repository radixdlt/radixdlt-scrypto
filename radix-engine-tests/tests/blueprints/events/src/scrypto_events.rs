use scrypto::prelude::*;

#[derive(ScryptoSbor)]
struct CustomEvent {
    number: u64,
}

#[blueprint]
#[events(CustomEvent)]
mod scrypto_events {
    struct ScryptoEvents;

    impl ScryptoEvents {
        pub fn emit_event(number: u64) {
            Runtime::emit_event(CustomEvent { number });
        }
    }
}
