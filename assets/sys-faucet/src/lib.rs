use scrypto::prelude::*;

// Faucet - TestNet only
blueprint! {
    struct Faucet {
        vault: Vault,
    }

    impl Faucet {
        pub fn new(bucket: Bucket) -> ComponentAddress {
            Self {
                vault: Vault::with_bucket(bucket),
            }
            .instantiate()
            .globalize()
        }

        /// Gives away tokens.
        pub fn free(&mut self) -> Bucket {
            self.vault.take(1_000_000)
        }

        /// Locks fees.
        pub fn lock_fee(&mut self, amount: Decimal) {
            self.vault.lock_fee(amount);
        }
    }
}
