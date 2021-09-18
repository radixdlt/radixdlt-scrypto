use scrypto::prelude::*;

blueprint! {
    struct Account {
        vaults: Storage,
    }

    impl Account {
        pub fn new() -> Address {
            Account {
                vaults: Storage::new(),
            }
            .instantiate()
        }

        /// Publishes a package from this account.
        pub fn publish_package(&self, code: Vec<u8>) -> Address {
            let package = Package::new(&code);
            package.into()
        }

        /// Creates a resource with mutable supply.
        pub fn new_resource_mutable(
            &self,
            metadata: HashMap<String, String>,
            minter: Address,
        ) -> Address {
            let resource = Resource::new_mutable(metadata, minter);
            resource.into()
        }

        /// Creates a resource with fixed supply, which will be deposited into this account.
        pub fn new_resource_fixed(
            &mut self,
            metadata: HashMap<String, String>,
            supply: Amount,
        ) -> Address {
            let bucket = Resource::new_fixed(metadata, supply);
            let address = bucket.resource();
            self.deposit(bucket);
            address
        }

        /// Mints resources and deposits them into this account.
        pub fn mint_resource(&mut self, amount: Amount, resource: Address)  {
            let bucket = Resource::from(resource).mint(amount);
            self.deposit(bucket);
        }

        /// Deposit a batch of buckets into this account
        pub fn deposit_batch(&mut self, buckets: Vec<Bucket>) {
            for bucket in buckets {
                self.deposit(bucket);
            }
        }

        /// Deposits resources into this account.
        pub fn deposit(&mut self, bucket: Bucket) {
            let resource = bucket.resource();
            match self.vaults.get::<Address, Vault>(&resource) {
                Some(v) => {
                    v.put(bucket);
                }
                None => {
                    let v = Vault::wrap(bucket);
                    self.vaults.insert(resource, v);
                }
            }
        }

        /// Withdraws resources from this account.
        pub fn withdraw(&mut self, amount: Amount, resource: Address) -> Bucket {
            let vault = self.vaults.get::<Address, Vault>(&resource).unwrap();
            vault.take(amount)
        }
    }
}
