use scrypto::prelude::*;

blueprint! {
    struct Account {
        public_key: EcdsaPublicKey,
        vaults: LazyMap<Address, Vault>,
    }

    impl Account {
        pub fn new(public_key: EcdsaPublicKey) -> Component {
            Account {
                public_key,
                vaults: LazyMap::new(),
            }
            .instantiate()
        }

        pub fn with_bucket(public_key: EcdsaPublicKey, bucket: Bucket) -> Component {
            let vaults = LazyMap::new();
            vaults.insert(bucket.resource_address(), Vault::with_bucket(bucket));

            Account { public_key, vaults }.instantiate()
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
                Some(mut v) => {
                    v.put(bucket);
                }
                None => {
                    let v = Vault::with_bucket(bucket);
                    self.vaults.insert(address, v);
                }
            }
        }

        fn nft_key(&self) -> NftKey {
            NftKey::new(self.public_key.to_vec())
        }

        /// Withdraws resource from this account.
        pub fn withdraw(
            &mut self,
            amount: Decimal,
            resource_address: Address,
            account_auth: BucketRef,
        ) -> Bucket {
            account_auth.check_nft_key(ECDSA_TOKEN, |key| key == &self.nft_key());

            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(mut vault) => vault.take(amount),
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
            account_auth: BucketRef,
        ) -> Bucket {
            account_auth.check_nft_key(ECDSA_TOKEN, |key| key == &self.nft_key());

            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(mut vault) => vault.take_with_auth(amount, auth),
                None => {
                    panic!("Insufficient balance");
                }
            }
        }

        /// Withdraws NFTs from this account.
        pub fn withdraw_nfts(
            &mut self,
            keys: BTreeSet<NftKey>,
            resource_address: Address,
            account_auth: BucketRef,
        ) -> Bucket {
            account_auth.check_nft_key(ECDSA_TOKEN, |key| key == &self.nft_key());

            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(vault) => {
                    let mut bucket = Bucket::new(resource_address);
                    for key in keys {
                        bucket.put(vault.take_nft(&key));
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
            keys: BTreeSet<NftKey>,
            resource_address: Address,
            auth: BucketRef,
            account_auth: BucketRef,
        ) -> Bucket {
            account_auth.check_nft_key(ECDSA_TOKEN, |key| key == &self.nft_key());

            let vault = self.vaults.get(&resource_address);
            let bucket = match vault {
                Some(vault) => {
                    let mut bucket = Bucket::new(resource_address);
                    for key in keys {
                        bucket.put(vault.take_nft_with_auth(&key, auth.clone()));
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
