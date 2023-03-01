use radix_engine_interface::api::LockFlags;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;
use scrypto::radix_engine_interface::api::ClientSubstateApi;

#[derive(ScryptoEncode)]
struct CustomEvent {
    number: u64,
}

#[blueprint]
mod event_store_visibility {
    struct EventStoreVisibility;

    impl EventStoreVisibility {
        pub fn emit_event(number: u64) {
            Runtime::emit_event(CustomEvent { number });
        }
    }
}
