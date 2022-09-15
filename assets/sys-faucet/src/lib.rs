use scrypto::prelude::*;

// Faucet - TestNet only
blueprint! {
    struct SysFaucet {
        vault: Vault,
        transactions: KeyValueStore<Hash, u64>,
        admin_badge_address: NonFungibleAddress,
    }

    impl SysFaucet {
        pub fn new(initial: Bucket, admin_badge_address: NonFungibleAddress) -> ComponentAddress {
            assert!(initial.resource_address() == RADIX_TOKEN);

            let mut component = SysFaucet {
                vault: Vault::with_bucket(initial),
                transactions: KeyValueStore::new(),
                admin_badge_address,
            }
            .instantiate();
            component.add_access_check(
                AccessRules::new()
                    .method("take", rule!(require("admin_badge_address")))
                    .default(rule!(allow_all)),
            );
            component.globalize()
        }

        /// Gives away XRD tokens.
        pub fn free_xrd(&mut self) -> Bucket {
            let transaction_hash = Runtime::transaction_hash();
            let epoch = Runtime::current_epoch();
            assert!(self.transactions.get(&transaction_hash).is_none());
            self.transactions.insert(transaction_hash, epoch);
            self.vault.take(1000)
        }

        /// Locks fees.
        pub fn lock_fee(&mut self, amount: Decimal) {
            // There is a MAX_COST_UNIT_LIMIT which limits how much fee can be spent per transaction,
            // thus no further constraints are applied.
            self.vault.lock_fee(amount);
        }

        pub fn take(&mut self, amount: Decimal) -> Bucket {
            self.vault.take(amount)
        }

        pub fn put(&mut self, bucket: Bucket) {
            self.vault.put(bucket);
        }
    }
}
