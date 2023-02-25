use scrypto::prelude::*;

#[blueprint]
mod transaction_limits {
    struct TransactionLimitTest {
        kv_store: KeyValueStore<u32, u32>,
    }

    impl TransactionLimitTest {
        pub fn write_kv_stores(n: u32) -> ComponentAddress {
            let kv_store = KeyValueStore::new();
            for i in 0..n {
                kv_store.insert(i, i);
            }

            TransactionLimitTest { kv_store }.instantiate().globalize()
        }

        pub fn read_kv_stores(n: u32) -> ComponentAddress {
            let kv_store = KeyValueStore::new();
            kv_store.insert(0, 0);
            for _i in 0..n {
                kv_store.get(&0);
            }

            TransactionLimitTest { kv_store }.instantiate().globalize()
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
