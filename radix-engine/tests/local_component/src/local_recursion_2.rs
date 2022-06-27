use scrypto::prelude::*;

blueprint! {
    struct LocalRecursionBomb2 {
        vaults: KeyValueStore<u32, Vault>,
    }

    impl LocalRecursionBomb2 {
        pub fn recurse(&mut self) -> Bucket {
            let mut vault = self.vaults.get(&0u32).unwrap();
            let amount_to_take = vault.amount() - 1;
            let bucket = vault.take(amount_to_take);
            let mut returned_bucket = Self::recursion_bomb(bucket);
            returned_bucket.put(vault.take(1));
            returned_bucket
        }

        pub fn recursion_bomb(bucket: Bucket) -> Bucket {
            if bucket.amount().is_zero() {
                return bucket;
            }

            let vaults = KeyValueStore::new();
            vaults.insert(0u32, Vault::with_bucket(bucket));
            let local_component = Self { vaults }.instantiate();

            let rtn_bucket = local_component.recurse();
            local_component.globalize();
            rtn_bucket
        }
    }
}
