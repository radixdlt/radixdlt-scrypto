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

        /// [Experimental] Publishes a package.
        pub fn publish_package(&self, code: Vec<u8>) -> Address {
            let package = Package::new(&code);
            package.into()
        }

        /// [Experimental] Creates a resource with mutable supply.
        pub fn new_resource_mutable(
            &self,
            metadata: HashMap<String, String>,
            minter: Address,
        ) -> Address {
            let resource_def = ResourceDef::new_mutable(metadata, minter);
            resource_def.address()
        }

        /// [Experimental] Creates a resource with fixed supply, which will be deposited into this account.
        pub fn new_resource_fixed(
            &mut self,
            metadata: HashMap<String, String>,
            supply: Amount,
        ) -> Address {
            let (resource_def, bucket) = ResourceDef::new_fixed(metadata, supply);
            self.deposit(bucket);
            resource_def.address()
        }

        /// [Experimental] Mints and deposits into this account.
        pub fn mint(&mut self, amount: Amount, resource_def: Address)  {
            let bucket = ResourceDef::from(resource_def).mint(amount);
            self.deposit(bucket);
        }

        /// Deposit a batch of buckets into this account
        pub fn deposit_batch(&mut self, buckets: Vec<Bucket>) {
            for bucket in buckets {
                self.deposit(bucket);
            }
        }

        /// Deposits a bucket of resource into this account.
        pub fn deposit(&mut self, bucket: Bucket) {
            let resource_def = bucket.resource_def().address();
            match self.vaults.get::<Address, Vault>(&resource_def) {
                Some(v) => {
                    v.put(bucket);
                }
                None => {
                    let v = Vault::with_bucket(bucket);
                    self.vaults.insert(resource_def, v);
                }
            }
        }

        /// Withdraws from this account.
        pub fn withdraw(&mut self, amount: Amount, resource_def: Address) -> Bucket {
            let vault = self.vaults.get::<Address, Vault>(&resource_def).unwrap();
            vault.take(amount)
        }
    }
}
