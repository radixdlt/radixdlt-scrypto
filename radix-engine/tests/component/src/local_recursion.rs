use scrypto::prelude::*;

blueprint! {
    struct LocalRecursionBomb {
        vault: Vault,
    }

    impl LocalRecursionBomb {
        pub fn recurse(&mut self) -> ComponentAddress {
            let amount_to_take = self.vault.amount() - 1;
            let bucket = self.vault.take(amount_to_take);
            Self::recursion_bomb(bucket)
        }

        pub fn recursion_bomb(bucket: Bucket) -> ComponentAddress {
            let local_component = Self {
                vault: Vault::with_bucket(bucket)
            }.instantiate();
            let _: ComponentAddress = local_component.call("recurse", vec![]);
            local_component.globalize()
        }
    }
}
