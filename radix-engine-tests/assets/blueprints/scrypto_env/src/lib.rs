use scrypto::prelude::*;

#[blueprint]
mod scrypto_env_test {
    struct ScryptoEnvTest {}

    impl ScryptoEnvTest {
        pub fn create_node_with_invalid_blueprint() {
            ScryptoVmV1Api::object_new(
                "invalid_blueprint",
                indexmap![0u8 => FieldValue::new(&ScryptoEnvTest {})],
            );
        }

        pub fn create_and_open_mut_substate_twice(heap: bool) {
            let obj = Self {}.instantiate();
            if heap {
                obj.open_mut_substate_twice();
                obj.prepare_to_globalize(OwnerRole::None).globalize();
            } else {
                let globalized = obj.prepare_to_globalize(OwnerRole::None).globalize();
                globalized.open_mut_substate_twice();
            }
        }

        pub fn open_mut_substate_twice(&mut self) {
            ScryptoVmV1Api::actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::MUTABLE);

            ScryptoVmV1Api::actor_open_field(ACTOR_STATE_SELF, 0u8, LockFlags::MUTABLE);
        }

        pub fn bech32_encode_address(address: ComponentAddress) -> String {
            Runtime::bech32_encode_address(address)
        }
    }
}

#[blueprint]
mod local_auth_zone {
    struct LocalAuthZoneTest {}

    impl LocalAuthZoneTest {
        pub fn pop_empty_auth_zone() -> Option<Proof> {
            LocalAuthZone::pop()
        }

        pub fn create_signature_proof() {
            let _ = LocalAuthZone::create_proof_of_all(SECP256K1_SIGNATURE_RESOURCE);
        }
    }
}

#[blueprint]
mod max_sbor_depth {
    use sbor::basic_well_known_types::ANY_TYPE;
    use sbor::*;

    struct MaxSborDepthTest {
        kv_store: Own,
    }

    impl MaxSborDepthTest {
        pub fn write_kv_store_entry_with_depth(buffer: Vec<u8>) {
            // Create KeyValueStore<Any, Any>
            let kv_store = ScryptoVmV1Api::kv_store_new(KeyValueStoreDataSchema::Local {
                additional_schema: Schema::empty().into(),
                key_type: LocalTypeId::from(ANY_TYPE),
                value_type: LocalTypeId::from(ANY_TYPE),
                allow_ownership: false,
            });

            // Open entry
            let handle = ScryptoVmV1Api::kv_store_open_entry(
                &kv_store,
                &scrypto_encode("key").unwrap(),
                LockFlags::MUTABLE,
            );

            // Write entry
            ScryptoVmV1Api::kv_entry_write(handle, buffer);
            ScryptoVmV1Api::kv_entry_close(handle);

            // Clean up
            Self {
                kv_store: Own(kv_store),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }
    }
}
