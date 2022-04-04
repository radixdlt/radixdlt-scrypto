use scrypto::prelude::*;

blueprint! {
    struct Account {
        vaults: LazyMap<ResourceDefId, Vault>,
    }

    impl Account {
        fn internal_new(withdraw_rule: MethodAuth, bucket: Option<Bucket>) -> ComponentId {
            let vaults = LazyMap::new();
            if let Some(b) = bucket {
                vaults.insert(b.resource_def_id(), Vault::with_bucket(b));
            }

            Self { vaults }
            .instantiate()
            .auth("withdraw", withdraw_rule.clone())
            .auth("withdraw_by_ids", withdraw_rule.clone())
            .auth("withdraw_by_amount", withdraw_rule.clone())
            .auth("create_proof_by_amount", withdraw_rule.clone())
            .auth("create_proof_by_ids", withdraw_rule.clone())
            .auth("deposit", auth!(allow_all))
            .auth("deposit_batch", auth!(allow_all))
            .globalize()
        }


        pub fn new(withdraw_rule: MethodAuth) -> ComponentId {
            Self::internal_new(withdraw_rule, Option::None)
        }

        pub fn new_with_resource(withdraw_rule: MethodAuth, bucket: Bucket) -> ComponentId {
            Self::internal_new(withdraw_rule, Option::Some(bucket))
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

        /// Deposit a batch of buckets into this account
        pub fn deposit_batch(&mut self, buckets: Vec<Bucket>) {
            for bucket in buckets {
                self.deposit(bucket);
            }
        }

        /// Withdraws resource from this account.
        pub fn withdraw(&mut self, resource_def_id: ResourceDefId) -> Bucket {
            let vault = self.vaults.get(&resource_def_id);
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
            resource_def_id: ResourceDefId,
        ) -> Bucket {
            let vault = self.vaults.get(&resource_def_id);
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
                    panic!("No such resource in account");
                }
            }
        }

        /// Create proof of resource.
        pub fn create_proof(&self, resource_def_id: ResourceDefId) -> Proof {
            let vault = self.vaults.get(&resource_def_id);
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
            resource_def_id: ResourceDefId,
        ) -> Proof {
            let vault = self.vaults.get(&resource_def_id);
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
            resource_def_id: ResourceDefId,
        ) -> Proof {
            let vault = self.vaults.get(&resource_def_id);
            match vault {
                Some(vault) => vault.create_proof_by_ids(&ids),
                None => {
                    panic!("No such resource in account");
                }
            }
        }
    }
}

package_init!(blueprint::Account::describe());
