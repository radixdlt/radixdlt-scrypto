use radix_engine_interface::api::LockFlags;
use scrypto::engine::scrypto_env::*;
use scrypto::prelude::*;
use scrypto::radix_engine_interface::api::ClientSubstateApi;

#[derive(ScryptoEncode, LegacyDescribe)]
struct CustomEvent {
    number: u64,
}

#[blueprint]
mod event_store_visibility {
    struct EventStoreVisibility;

    impl EventStoreVisibility {
        pub fn lock_event_store(lock_flags: u32) {
            let mut env = ScryptoEnv;
            env.sys_lock_substate(
                RENodeId::EventStore,
                SubstateOffset::EventStore(EventStoreOffset::EventStore),
                LockFlags::from_bits(lock_flags).unwrap(),
            )
            .unwrap();
        }

        pub fn emit_event(number: u64) {
            Runtime::emit_event(CustomEvent { number });
        }
    }
}
