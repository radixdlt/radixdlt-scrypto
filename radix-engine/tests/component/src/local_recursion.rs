use scrypto::prelude::*;

blueprint! {
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

            let rtn: Bucket = local_component.call("recurse", vec![]);
            local_component.globalize();
            rtn
        }
    }
}
