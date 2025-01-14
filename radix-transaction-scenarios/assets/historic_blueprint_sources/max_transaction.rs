use scrypto::prelude::*;

#[derive(ScryptoSbor, ScryptoEvent)]
pub struct A {
    a: Vec<u8>,
}

#[blueprint]
#[events(A)]
mod max_transaction {
    struct MaxTransaction {
        kv_store: KeyValueStore<u32, Vec<u8>>,
    }

    impl MaxTransaction {
        pub fn new() {
            Self {
                kv_store: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }

        pub fn max_events(n: u32) {
            let name = "A";
            let mut buf = Vec::with_capacity(MAX_EVENT_SIZE);
            let mut enc = ScryptoEncoder::new(&mut buf, 100);
            enc.write_discriminator(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
                .unwrap();
            enc.write_value_kind(ValueKind::Tuple).unwrap();
            enc.write_size(1).unwrap();
            enc.write_value_kind(ValueKind::Array).unwrap();
            enc.write_value_kind(ValueKind::U8).unwrap();
            enc.write_size(
                MAX_EVENT_SIZE
                - 5 /* the above */
                - 3, /* the size */
            )
            .unwrap();
            unsafe { buf.set_len(MAX_EVENT_SIZE) };

            for _ in 0..n {
                unsafe {
                    wasm_api::actor::actor_emit_event(
                        name.as_ptr(),
                        name.len(),
                        buf.as_ptr(),
                        buf.len(),
                        0,
                    )
                }
            }
        }

        pub fn max_state_updates(&mut self, n: u32) {
            let max_size = MAX_SUBSTATE_VALUE_SIZE - 11;

            let mut buf = Vec::with_capacity(max_size);
            let mut enc = ScryptoEncoder::new(&mut buf, 100);
            enc.write_discriminator(SCRYPTO_SBOR_V1_PAYLOAD_PREFIX)
                .unwrap();
            enc.write_value_kind(ValueKind::Array).unwrap();
            enc.write_value_kind(ValueKind::U8).unwrap();
            enc.write_size(
                max_size
                - 3 /* the above */
                - 3, /* the size */
            )
            .unwrap();
            unsafe { buf.set_len(max_size) };

            for i in 0..n {
                let handle = ScryptoVmV1Api::kv_store_open_entry(
                    self.kv_store.id.as_node_id(),
                    &scrypto_encode(&i).unwrap(),
                    LockFlags::MUTABLE,
                );
                unsafe { wasm_api::kv_entry::kv_entry_write(handle, buf.as_ptr(), buf.len()) };
                ScryptoVmV1Api::kv_entry_close(handle);
            }
        }
    }
}
