use scrypto::prelude::*;

// Faucet - TestNet only
blueprint! {
    struct SysFaucet {
        vault: Vault,
        transactions: KeyValueStore<Hash, u64>,
    }

    impl SysFaucet {
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
            // There is MAX_COST_UNIT_LIMIT and COST_UNIT_PRICE which limit how much fee can be spent
            // per transaction, thus no further limitation is applied.
            self.vault.lock_fee(amount);
        }
    }
}
