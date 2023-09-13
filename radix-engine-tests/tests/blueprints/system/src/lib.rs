use scrypto::prelude::*;

#[blueprint]
mod recursive_test {
    struct HandleMismatchTest {}

    impl HandleMismatchTest {
        pub fn new() -> Global<HandleMismatchTest> {
            Self {}
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn treat_field_handle_as_kv_store_handle(&self) {
            let lock_handle =
                ScryptoVmV1Api::actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::read_only());

            ScryptoVmV1Api::kv_entry_remove(lock_handle);
        }
    }
}
