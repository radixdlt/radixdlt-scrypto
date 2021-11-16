use scrypto::prelude::*;

blueprint! {
    struct Account {
        key: Address,
        vaults: LazyMap<Address, Vault>,
    }

    impl Account {
        pub fn new(key: Address) -> Component {
            Account {
                key,
                vaults: LazyMap::new(),
            }
            .instantiate()
        }

        pub fn with_bucket(key: Address, bucket: Bucket) -> Component {
            let account = Self::new(key);
            account.call::<()>("deposit", vec![scrypto_encode(&bucket)]);
            account
        }

        /// Deposit a batch of buckets into this account
        pub fn deposit_batch(&mut self, buckets: Vec<Bucket>) {
            for bucket in buckets {
                self.deposit(bucket);
            }
        }

        /// Deposits resource into this account.
        pub fn deposit(&mut self, bucket: Bucket) {
            let address = bucket.resource_address();
            match self.vaults.get(&address) {
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
        pub fn withdraw(&mut self, amount: Decimal, resource_def: Address) -> Bucket {
            if !Context::transaction_signers().contains(&self.key) {
                panic!("Not authorized! Make sure you sign transaction with the correct keys.",)
            }

            let vault = self.vaults.get(&resource_def);
            match vault {
                Some(vault) => vault.take(amount),
                None => {
                    panic!("Insufficient balance");
                }
            }
        }
    }
}
