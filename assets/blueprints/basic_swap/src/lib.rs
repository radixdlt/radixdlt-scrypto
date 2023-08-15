use scrypto::prelude::*;

#[blueprint]
mod basic_swap {
    struct BasicSwap {
        xrd_vault: Vault,
        my_vault: Vault,
    }

    impl BasicSwap {
        pub fn new() -> Global<BasicSwap> {
            let bucket =
                ResourceBuilder::new_fungible(OwnerRole::None).mint_initial_supply(1000000000);

            Self {
                xrd_vault: Vault::new(XRD),
                my_vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        pub fn swap(&mut self, xrd: Bucket) -> Bucket {
            self.xrd_vault.put(xrd);
            self.my_vault.take(1)
        }
    }
}
