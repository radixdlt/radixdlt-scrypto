use scrypto::prelude::*;

#[blueprint]
mod local_recursion_bomb {
    struct LocalRecursionBomb {
        vault: Vault,
    }

    impl LocalRecursionBomb {
        pub fn recurse(&mut self) -> Bucket {
            let amount_to_take = self.vault.amount() - 1;
            let bucket = self.vault.take(amount_to_take);
            let mut returned_bucket = Self::recursion_bomb(bucket);
            returned_bucket.put(self.vault.take(1));
            returned_bucket
        }

        pub fn recursion_bomb(bucket: Bucket) -> Bucket {
            if bucket.amount().is_zero() {
                return bucket;
            }

            let local_component = Self {
                vault: Vault::with_bucket(bucket),
            }
            .instantiate();

            let rtn_bucket = local_component.recurse();
            local_component.globalize();
            rtn_bucket
        }
    }
}

#[blueprint]
mod local_recursion_bomb2 {
    struct LocalRecursionBomb2 {
        vaults: KeyValueStore<u32, Vault>,
    }

    impl LocalRecursionBomb2 {
        pub fn recurse(&mut self) -> Bucket {
            let mut vault = self.vaults.get_mut(&0u32).unwrap();
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
