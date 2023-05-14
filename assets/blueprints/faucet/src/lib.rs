use scrypto::prelude::*;

// Faucet - TestNet only
#[blueprint]
mod faucet {
    struct Faucet {
        vault: Vault,
        transactions: KeyValueStore<Hash, u64>,
    }

    impl Faucet {
        pub fn new(
            preallocated_address_bytes: [u8; 30],
            bucket: Bucket,
        ) -> Global<FaucetComponent> {
            let access_rules = AccessRules::new(MethodAuthorities::new(), AuthorityRules::new());

            Self {
                vault: Vault::with_bucket(bucket),
                transactions: KeyValueStore::new(),
            }
            .instantiate()
            .globalize_at_address_with_modules(
                ComponentAddress::new_or_panic(preallocated_address_bytes),
                access_rules,
                Royalty::new(RoyaltyConfig::default()),
            )
        }

        /// Gives away tokens.
        pub fn free(&mut self) -> Bucket {
            let transaction_hash = Runtime::transaction_hash();
            let epoch = Runtime::current_epoch();
            assert!(self.transactions.get(&transaction_hash).is_none());
            self.transactions.insert(transaction_hash, epoch);
            self.vault.take(10000)
        }

        /// Locks fees.
        pub fn lock_fee(&mut self, amount: Decimal) {
            // There is MAX_COST_UNIT_LIMIT and COST_UNIT_PRICE which limit how much fee can be spent
            // per transaction, thus no further limitation is applied.
            self.vault.lock_fee(amount);
        }
    }
}
