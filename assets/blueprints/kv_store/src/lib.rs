use scrypto::prelude::*;

#[derive(ScryptoSbor)]
struct TestType {
    b: u32,
}

#[blueprint]
#[types(TestType)]
mod kv_store {
    struct KVStore {
        kv_store: Own,
    }

    impl KVStore {
        pub fn create_key_value_store_with_remote_type(
            package_address: PackageAddress,
            blueprint_name: String,
            type_name: String,
        ) {
            // Create
            let kv_store = ScryptoVmV1Api::kv_store_new(KeyValueStoreDataSchema::Remote {
                key_type: BlueprintTypeIdentifier {
                    package_address,
                    blueprint_name: blueprint_name.clone(),
                    type_name: type_name.clone(),
                },
                value_type: BlueprintTypeIdentifier {
                    package_address,
                    blueprint_name: blueprint_name.clone(),
                    type_name: type_name.clone(),
                },
                allow_ownership: false,
            });

            // Insert
            let handle = ScryptoVmV1Api::kv_store_open_entry(
                &kv_store,
                &scrypto_encode(&TestType { b: 1 }).unwrap(),
                LockFlags::MUTABLE,
            );
            ScryptoVmV1Api::kv_entry_write(handle, scrypto_encode(&TestType { b: 1 }).unwrap());
            ScryptoVmV1Api::kv_entry_close(handle);

            Self {
                kv_store: Own(kv_store),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize();
        }
    }
}
