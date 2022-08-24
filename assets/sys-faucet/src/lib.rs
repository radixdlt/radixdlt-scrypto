use scrypto::prelude::*;

// Faucet - TestNet only
blueprint! {
    struct SysFaucet {
        xrd: Vault,
    }

    impl SysFaucet {
        /// Gives away XRD tokens.
        pub fn free_xrd(&mut self) -> Bucket {
            self.xrd.take(1_000_000)
        }

        /// Locks fees.
        pub fn lock_fee(&mut self, amount: Decimal) {
            self.xrd.lock_fee(amount);
        }
    }
}
