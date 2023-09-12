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

#[blueprint]
mod address_reservation_test {
    struct AddressReservationTest {
        own: ScryptoValue,
    }

    impl AddressReservationTest {
        pub fn drop_address_reservation(_reservation: GlobalAddressReservation) {
            // No longer works as object dropping API has been removed for WASM
        }

        pub fn put_address_reservation_into_component_state(reservation: GlobalAddressReservation) {
            Self {
                own: scrypto_decode(&scrypto_encode(&reservation).unwrap()).unwrap(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }

        pub fn put_address_reservation_into_kv_store(reservation: GlobalAddressReservation) {
            let kv_store = KeyValueStore::<u32, ScryptoValue>::new();
            kv_store.insert(
                1u32,
                scrypto_decode(&scrypto_encode(&reservation).unwrap()).unwrap(),
            );

            Self {
                own: scrypto_decode(&scrypto_encode(&kv_store).unwrap()).unwrap(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }
    }
}
