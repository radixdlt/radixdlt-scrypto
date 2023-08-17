use scrypto::prelude::*;

// Faucet - TestNet only
#[blueprint]
mod faucet {
    struct Faucet {
        vault: Vault,
        transactions: KeyValueStore<Hash, Epoch>,
    }

    impl Faucet {
        pub fn new(
            address_reservation: GlobalAddressReservation,
            bucket: Bucket,
        ) -> Global<Faucet> {
            Self {
                vault: Vault::with_bucket(bucket),
                transactions: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .with_address(address_reservation)
            .metadata(metadata! {
                init {
                    "name" => "Test Faucet".to_owned(), locked;
                    "description" => "A simple faucet for distributing tokens for testing purposes.".to_owned(), locked;
                }
            })
            .globalize()
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
            self.vault.as_fungible().lock_fee(amount);
        }
    }
}
