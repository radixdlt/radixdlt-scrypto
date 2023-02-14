use scrypto::prelude::*;

#[blueprint]
mod account {
    struct Account {
        vaults: KeyValueStore<ResourceAddress, Vault>,
    }

    impl Account {
        pub fn create(withdraw_rule: AccessRule) -> AccountComponent {
            let mut account = Self {
                vaults: KeyValueStore::new(),
            }
            .instantiate();

            let access_rules = AccessRules::new()
                .method("balance", rule!(allow_all), LOCKED)
                .method("deposit", rule!(allow_all), LOCKED)
                .method("deposit_batch", rule!(allow_all), LOCKED)
                .default(withdraw_rule.clone(), MUTABLE(withdraw_rule));
            account.add_access_check(access_rules);

            account
        }

        fn internal_new(withdraw_rule: AccessRule, bucket: Option<Bucket>) -> ComponentAddress {
            let mut account = Self {
                vaults: KeyValueStore::new(),
            }
            .instantiate();

            if let Some(b) = bucket {
                // Test out the local component calls
                account.deposit(b);
            }

            let access_rules = AccessRules::new()
                .method("balance", rule!(allow_all), LOCKED)
                .method("deposit", rule!(allow_all), LOCKED)
                .method("deposit_batch", rule!(allow_all), LOCKED)
                .default(withdraw_rule.clone(), MUTABLE(withdraw_rule));
            account.add_access_check(access_rules);

            account.globalize()
        }

        pub fn new(withdraw_rule: AccessRule) -> ComponentAddress {
            Self::internal_new(withdraw_rule, Option::None)
        }

        pub fn new_with_resource(withdraw_rule: AccessRule, bucket: Bucket) -> ComponentAddress {
            Self::internal_new(withdraw_rule, Option::Some(bucket))
        }

        pub fn balance(&self, resource_address: ResourceAddress) -> Decimal {
            self.vaults
                .get(&resource_address)
                .map(|v| v.amount())
                .unwrap_or_default()
        }

        pub fn lock_fee(&mut self, amount: Decimal) {
            let vault = self.vaults.get_mut(&RADIX_TOKEN);
            match vault {
                Some(mut vault) => vault.lock_fee(amount),
                None => {
                    panic!("No XRD in account");
                }
            }
        }

        pub fn lock_contingent_fee(&mut self, amount: Decimal) {
            let vault = self.vaults.get_mut(&RADIX_TOKEN);
            match vault {
                Some(mut vault) => vault.lock_contingent_fee(amount),
                None => {
                    panic!("No XRD in account");
                }
            }
        }

        /// Deposits resource into this account.
        pub fn deposit(&mut self, bucket: Bucket) {
            let resource_address = bucket.resource_address();
            if self.vaults.get(&resource_address).is_none() {
                let v = Vault::with_bucket(bucket);
                self.vaults.insert(resource_address, v);
            } else {
                let mut v = self.vaults.get_mut(&resource_address).unwrap();
                v.put(bucket);
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
            let vault = self.vaults.get_mut(&resource_address);
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
            let vault = self.vaults.get_mut(&resource_address);
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
            ids: BTreeSet<NonFungibleLocalId>,
            resource_address: ResourceAddress,
        ) -> Bucket {
            let vault = self.vaults.get_mut(&resource_address);
            match vault {
                Some(mut vault) => vault.take_non_fungibles(&ids),
                None => {
                    panic!("No such resource in account");
                }
            }
        }

        pub fn lock_fee_and_withdraw(
            &mut self,
            amount_to_lock: Decimal,
            resource_address: ResourceAddress,
        ) -> Bucket {
            self.lock_fee(amount_to_lock);
            self.withdraw(resource_address)
        }

        pub fn lock_fee_and_withdraw_by_amount(
            &mut self,
            amount_to_lock: Decimal,
            amount: Decimal,
            resource_address: ResourceAddress,
        ) -> Bucket {
            self.lock_fee(amount_to_lock);
            self.withdraw_by_amount(amount, resource_address)
        }

        pub fn lock_fee_and_withdraw_by_ids(
            &mut self,
            amount_to_lock: Decimal,
            ids: BTreeSet<NonFungibleLocalId>,
            resource_address: ResourceAddress,
        ) -> Bucket {
            self.lock_fee(amount_to_lock);
            self.withdraw_by_ids(ids, resource_address)
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
            ids: BTreeSet<NonFungibleLocalId>,
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
