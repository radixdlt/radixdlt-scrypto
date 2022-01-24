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
            let vaults = LazyMap::new();
            vaults.insert(bucket.resource_address(), Vault::with_bucket(bucket));

            Account { key, vaults }.instantiate()
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
        pub fn withdraw(&mut self, amount: Decimal, resource_address: Address, account_auth: BucketRef) -> Bucket {
            account_auth.check_nft_id(ECDSA_TOKEN, |id| id == &self.key.to_u128());

            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(vault) => vault.take(amount),
                None => {
                    panic!("Insufficient balance");
                }
            }
        }

        /// Withdraws resource from this account.
        pub fn withdraw_with_auth(
            &mut self,
            amount: Decimal,
            resource_address: Address,
            auth: BucketRef,
            account_auth: BucketRef
        ) -> Bucket {
            account_auth.check_nft_id(ECDSA_TOKEN, |id| id == &self.key.to_u128());

            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(vault) => vault.take_with_auth(amount, auth),
                None => {
                    panic!("Insufficient balance");
                }
            }
        }

        /// Withdraws NFTs from this account.
        pub fn withdraw_nfts(&mut self, ids: BTreeSet<u128>, resource_address: Address, account_auth: BucketRef) -> Bucket {
            account_auth.check_nft_id(ECDSA_TOKEN, |id| id == &self.key.to_u128());

            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(vault) => {
                    let bucket = Bucket::new(resource_address);
                    for id in ids {
                        bucket.put(vault.take_nft(id));
                    }
                    bucket
                }
                None => {
                    panic!("Insufficient balance");
                }
            }
        }

        /// Withdraws NFTs from this account.
        pub fn withdraw_nfts_with_auth(
            &mut self,
            ids: BTreeSet<u128>,
            resource_address: Address,
            auth: BucketRef,
            account_auth: BucketRef
        ) -> Bucket {
            account_auth.check_nft_id(ECDSA_TOKEN, |id| id == &self.key.to_u128());

            let vault = self.vaults.get(&resource_address);
            let bucket = match vault {
                Some(vault) => {
                    let bucket = Bucket::new(resource_address);
                    for id in ids {
                        bucket.put(vault.take_nft_with_auth(id, auth.clone()));
                    }
                    bucket
                }
                None => {
                    panic!("Insufficient balance")
                }
            };

            auth.drop();
            bucket
        }
    }
}
