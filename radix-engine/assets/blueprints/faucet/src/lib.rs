use scrypto::prelude::*;

// Faucet - TestNet only
#[blueprint]
#[types(Hash, Epoch)]
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
                transactions: KeyValueStore::new_with_registered_type(),
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

        /// Gives away tokens
        pub fn free(&mut self) -> Bucket {
            let transaction_hash = Runtime::transaction_hash();
            let epoch = Runtime::current_epoch();
            assert!(self.transactions.get(&transaction_hash).is_none());
            self.transactions.insert(transaction_hash, epoch);

            let amount: Decimal = 10000.into();

            if self.vault.amount() < amount {
                panic!("The faucet doesn't have funds on this environment. You will need to source XRD another way.")
            }

            self.vault.take(amount)
        }

        /// Locks fee
        pub fn lock_fee(&mut self, amount: Decimal) {
            if self.vault.amount() < amount {
                panic!("The faucet doesn't have funds on this environment. Consider locking fee from an account instead.")
            }

            self.vault.as_fungible().lock_fee(amount);
        }
    }
}