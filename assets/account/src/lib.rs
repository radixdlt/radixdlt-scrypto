use scrypto::prelude::*;

blueprint! {
    struct Account {
        vaults: LazyMap,
    }

    impl Account {
        pub fn new() -> Address {
            Account {
                vaults: LazyMap::new(),
            }
            .instantiate()
        }

        pub fn with_bucket(bucket: Bucket) -> Address {
            let mut account = Account {
                vaults: LazyMap::new()
            };
            account.deposit(bucket);
            account.instantiate()
        }

        /// Deposit a batch of buckets into this account
        pub fn deposit_batch(&mut self, buckets: Vec<Bucket>) {
            for bucket in buckets {
                self.deposit(bucket);
            }
        }

        /// Deposits resource into this account.
        pub fn deposit(&mut self, bucket: Bucket) {
            let address = bucket.resource_def().address();
            match self.vaults.get::<Address, Vault>(&address) {
                Some(v) => {
                    v.put(bucket);
                }
                None => {
                    let v = Vault::with_bucket(bucket);
                    self.vaults.insert(address, v);
                }
            }
        }

        /// Withdraws resource from this account.
        pub fn withdraw(&mut self, amount: Amount, address: Address) -> Bucket {
            let vault = self
                .vaults
                .get::<Address, Vault>(&address)
                .unwrap();
            vault.take(amount)
        }
    }
}
