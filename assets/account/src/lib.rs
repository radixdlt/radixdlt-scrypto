use scrypto::prelude::*;

blueprint! {
    struct Account {
        vaults: LazyMap<ResourceDefId, Vault>,
    }

    impl Account {
        pub fn new(auth_rule: ProofRule) -> ComponentId {
            Account {
                vaults: LazyMap::new(),
            }
            .instantiate_with_auth(HashMap::from([("withdraw".to_string(), auth_rule)]))
        }

        pub fn with_bucket(auth_rule: ProofRule, bucket: Bucket) -> ComponentId {
            let vaults = LazyMap::new();
            vaults.insert(bucket.resource_def_id(), Vault::with_bucket(bucket));
            Account { vaults }
                .instantiate_with_auth(HashMap::from([("withdraw".to_string(), auth_rule)]))
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

        /// Withdraws resource from this account.
        pub fn withdraw(&mut self, amount: Decimal, resource_def_id: ResourceDefId) -> Bucket {
            let vault = self.vaults.get(&resource_def_id);
            match vault {
                Some(mut vault) => vault.take(amount),
                None => {
                    panic!("Insufficient balance");
                }
            }
        }

        /// Withdraws non-fungibles from this account.
        pub fn withdraw_non_fungibles(
            &mut self,
            ids: BTreeSet<NonFungibleId>,
            resource_def_id: ResourceDefId,
        ) -> Bucket {
            let vault = self.vaults.get(&resource_def_id);
            match vault {
                Some(vault) => {
                    let mut bucket = Bucket::new(resource_def_id);
                    for id in ids {
                        bucket.put(vault.take_non_fungible(&id));
                    }
                    bucket
                }
                None => {
                    panic!("Insufficient balance");
                }
            }
        }
    }
}
