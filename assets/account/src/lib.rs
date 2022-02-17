use scrypto::prelude::*;

blueprint! {
    struct Account {
        public_key: EcdsaPublicKey,
        vaults: LazyMap<ResourceDefId, Vault>,
    }

    impl Account {
        pub fn new(public_key: EcdsaPublicKey) -> ComponentId {
            Account {
                public_key,
                vaults: LazyMap::new(),
            }
            .instantiate()
        }

        pub fn with_bucket(public_key: EcdsaPublicKey, bucket: Bucket) -> ComponentId {
            let vaults = LazyMap::new();
            vaults.insert(bucket.resource_def_id(), Vault::with_bucket(bucket));

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
            let resource_def_id = bucket.resource_def_id();
            match self.vaults.get(&resource_def_id) {
                Some(mut v) => {
                    v.put(bucket);
                }
                None => {
                    let v = Vault::with_bucket(bucket);
                    self.vaults.insert(resource_def_id, v);
                }
            }
        }

        fn non_fungible_key(&self) -> NonFungibleKey {
            NonFungibleKey::new(self.public_key.to_vec())
        }

        /// Withdraws resource from this account.
        pub fn withdraw(
            &mut self,
            amount: Decimal,
            resource_def_id: ResourceDefId,
            account_auth: Proof,
        ) -> Bucket {
            account_auth.check_non_fungible_key(ECDSA_TOKEN, |key| key == &self.non_fungible_key());

            let vault = self.vaults.get(&resource_def_id);
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
            resource_def_id: ResourceDefId,
            auth: Proof,
            account_auth: Proof,
        ) -> Bucket {
            account_auth.check_non_fungible_key(ECDSA_TOKEN, |key| key == &self.non_fungible_key());

            let vault = self.vaults.get(&resource_def_id);
            match vault {
                Some(mut vault) => vault.take_with_auth(amount, auth),
                None => {
                    panic!("Insufficient balance");
                }
            }
        }

        /// Withdraws non-fungibles from this account.
        pub fn withdraw_non_fungibles(
            &mut self,
            keys: BTreeSet<NonFungibleKey>,
            resource_def_id: ResourceDefId,
            account_auth: Proof,
        ) -> Bucket {
            account_auth.check_non_fungible_key(ECDSA_TOKEN, |key| key == &self.non_fungible_key());

            let vault = self.vaults.get(&resource_def_id);
            match vault {
                Some(vault) => {
                    let mut bucket = Bucket::new(resource_def_id);
                    for key in keys {
                        bucket.put(vault.take_non_fungible(&key));
                    }
                    bucket
                }
                None => {
                    panic!("Insufficient balance");
                }
            }
        }

        /// Withdraws non-fungibles from this account.
        pub fn withdraw_non_fungibles_with_auth(
            &mut self,
            keys: BTreeSet<NonFungibleKey>,
            resource_def_id: ResourceDefId,
            auth: Proof,
            account_auth: Proof,
        ) -> Bucket {
            account_auth.check_non_fungible_key(ECDSA_TOKEN, |key| key == &self.non_fungible_key());

            let vault = self.vaults.get(&resource_def_id);
            match vault {
                Some(vault) => {
                    let mut bucket = Bucket::new(resource_def_id);
                    for key in keys {
                        bucket.put(vault.take_non_fungible_with_auth(&key, auth.clone()));
                    }
                    bucket
                }
                None => {
                    panic!("Insufficient balance")
                }
            }
        }
    }
}
