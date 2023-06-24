use scrypto::api::key_value_entry_api::ClientKeyValueEntryApi;
use scrypto::api::key_value_store_api::ClientKeyValueStoreApi;
use scrypto::prelude::scrypto_env::ScryptoEnv;
use scrypto::prelude::*;

#[blueprint]
mod transaction_limits {
    struct TransactionLimitTest {
        kv_store: KeyValueStore<u32, u32>,
    }

    impl TransactionLimitTest {
        pub fn write_kv_stores(n: u32) -> Global<TransactionLimitTest> {
            let kv_store = KeyValueStore::new();
            for i in 0..n {
                kv_store.insert(i, i);
            }

            TransactionLimitTest { kv_store }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn read_kv_stores(n: u32) -> Global<TransactionLimitTest> {
            let kv_store = KeyValueStore::new();
            kv_store.insert(0, 0);
            for _i in 0..n {
                kv_store.get(&0);
            }

            TransactionLimitTest { kv_store }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }

        pub fn recursive_with_memory(n: u32, m: usize) {
            if n > 1 {
                let _v: Vec<u8> = Vec::with_capacity(m);
                let _: () = Runtime::call_function(
                    Runtime::package_address(),
                    "TransactionLimitTest",
                    "recursive_with_memory",
                    scrypto_args!(n - 1, m),
                );
            }
        }
    }
}

#[blueprint]
mod transaction_limits_substate {
    struct TransactionLimitSubstateTest {
        kv_store: KeyValueStore<u32, Vec<u8>>,
    }

    impl TransactionLimitSubstateTest {
        pub fn write_large_value(raw_array_size: usize) -> Global<TransactionLimitSubstateTest> {
            let mut vec = Vec::<u8>::with_capacity(raw_array_size);
            unsafe {
                vec.set_len(raw_array_size);
            }

            // Create a KVStore
            let kv_store = KeyValueStore::<u32, Vec<u8>>::new();

            // Insert into store
            let key_payload = scrypto_encode(&1u32).unwrap();
            let value_payload = scrypto_encode(&vec).unwrap();
            let handle = ScryptoEnv
                .key_value_store_lock_entry(
                    kv_store.id.as_node_id(),
                    &key_payload,
                    LockFlags::MUTABLE,
                )
                .unwrap();
            ScryptoEnv
                .key_value_entry_set(handle, value_payload)
                .unwrap();
            ScryptoEnv.key_value_entry_release(handle).unwrap();

            // Put the kv store into a component
            TransactionLimitSubstateTest { kv_store }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }
    }
}

#[blueprint]
mod invoke_limits {
    struct InvokeLimitsTest {}

    impl InvokeLimitsTest {
        pub fn call(raw_array_size: usize) {
            let mut vec = Vec::<u8>::with_capacity(raw_array_size);
            unsafe {
                vec.set_len(raw_array_size);
            }

            Runtime::call_function(
                Runtime::package_address(),
                "InvokeLimitsTest",
                "callee",
                scrypto_args!(vec),
            )
        }

        pub fn callee(_: Vec<u8>) {}
    }
}

#[blueprint]
mod sbor_overflow {
    struct SborOverflow {
        kv_store: KeyValueStore<u32, Vec<u8>>,
    }

    impl SborOverflow {
        pub fn write_large_value() -> Global<SborOverflow> {
            // Construct large SBOR payload
            let mut vec = Vec::<u8>::with_capacity(1 * 1024 * 1024);
            unsafe {
                vec.set_len(1 * 1024 * 1024);
            }
            (&mut vec[0..7]).copy_from_slice(&[92, 32, 7, 249, 193, 215, 47]);

            // Create a KVStore
            let kv_store = KeyValueStore::<u32, Vec<u8>>::new();

            // Insert into store
            let key_payload = scrypto_encode(&1u32).unwrap();
            let value_payload = vec;
            let handle = ScryptoEnv
                .key_value_store_lock_entry(
                    kv_store.id.as_node_id(),
                    &key_payload,
                    LockFlags::MUTABLE,
                )
                .unwrap();
            ScryptoEnv
                .key_value_entry_set(handle, value_payload)
                .unwrap();
            ScryptoEnv.key_value_entry_release(handle).unwrap();

            // Put the kv store into a component
            SborOverflow { kv_store }
                .instantiate()
                .prepare_to_globalize(OwnerRole::None)
                .globalize()
        }
    }
}
