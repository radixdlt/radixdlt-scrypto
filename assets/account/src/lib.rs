use scrypto::prelude::*;

blueprint! {
    struct Account {
        vaults: LazyMap<ResourceAddress, Vault>,
    }

    impl Account {
        fn internal_new(withdraw_rule: MethodAuth, bucket: Option<Bucket>) -> ComponentAddress {
            let vaults = LazyMap::new();
            if let Some(b) = bucket {
                vaults.insert(b.resource_address(), Vault::with_bucket(b));
            }

            let access_rules = AccessRules::new()
                .method("deposit", auth!(allow_all))
                .method("deposit_batch", auth!(allow_all))
                .default(withdraw_rule);

            Self { vaults }.instantiate().add_access_check(access_rules).globalize()
        }

        pub fn new(withdraw_rule: MethodAuth) -> ComponentAddress {
            Self::internal_new(withdraw_rule, Option::None)
        }

        pub fn new_with_resource(withdraw_rule: MethodAuth, bucket: Bucket) -> ComponentAddress {
            Self::internal_new(withdraw_rule, Option::Some(bucket))
        }

        /// Deposits resource into this account.
        pub fn deposit(&mut self, bucket: Bucket) {
            let resource_address = bucket.resource_address();
            match self.vaults.get(&resource_address) {
                Some(mut v) => {
                    v.put(bucket);
                }
                None => {
                    let v = Vault::with_bucket(bucket);
                    self.vaults.insert(resource_address, v);
                }
            }
        }

        /// Deposit a batch of buckets into this account
        pub fn deposit_batch(&mut self, buckets: Vec<Bucket>) {
            for bucket in buckets {
                self.deposit(bucket);
            }
        }

        /// Withdraws resource from this account.
        pub fn withdraw(&mut self, resource_address: ResourceAddress) -> Bucket {
            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(mut vault) => vault.take_all(),
                None => {
                    panic!("No such resource in account");
                }
            }
        }

        /// Withdraws resource from this account, by amount.
        pub fn withdraw_by_amount(
            &mut self,
            amount: Decimal,
            resource_address: ResourceAddress,
        ) -> Bucket {
            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(mut vault) => vault.take(amount),
                None => {
                    panic!("No such resource in account");
                }
            }
        }

        /// Withdraws resource from this account, by non-fungible ids.
        pub fn withdraw_by_ids(
            &mut self,
            ids: BTreeSet<NonFungibleId>,
            resource_address: ResourceAddress,
        ) -> Bucket {
            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(mut vault) => vault.take_non_fungibles(&ids),
                None => {
                    panic!("No such resource in account");
                }
            }
        }

        /// Create proof of resource.
        pub fn create_proof(&self, resource_address: ResourceAddress) -> Proof {
            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(vault) => vault.create_proof(),
                None => {
                    panic!("No such resource in account");
                }
            }
        }

        /// Create proof of resource.
        ///
        /// A runtime error is raised if the amount is zero or there isn't enough
        /// balance to cover the amount.
        pub fn create_proof_by_amount(
            &self,
            amount: Decimal,
            resource_address: ResourceAddress,
        ) -> Proof {
            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(vault) => vault.create_proof_by_amount(amount),
                None => {
                    panic!("No such resource in account");
                }
            }
        }

        /// Create proof of resource.
        ///
        /// A runtime error is raised if the non-fungible ID set is empty or not
        /// available in this account.
        pub fn create_proof_by_ids(
            &self,
            ids: BTreeSet<NonFungibleId>,
            resource_address: ResourceAddress,
        ) -> Proof {
            let vault = self.vaults.get(&resource_address);
            match vault {
                Some(vault) => vault.create_proof_by_ids(&ids),
                None => {
                    panic!("No such resource in account");
                }
            }
        }
    }
}
