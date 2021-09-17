use scrypto::prelude::*;

blueprint! {
    struct Account {
        buckets: Storage,
    }

    impl Account {
        pub fn new() -> Address {
            Account {
                buckets: Storage::new(),
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
            supply: U256,
        ) -> Address {
            let bucket = Resource::new_fixed(metadata, supply);
            let address = bucket.resource();
            self.deposit(bucket);
            address
        }

        /// Mints resources and deposits them into this account.
        pub fn mint_resource(&mut self, amount: U256, resource: Address)  {
            let bucket = Resource::from(resource).mint(amount);
            self.deposit(bucket);
        }

        /// Deposit buckets of resources into this account
        pub fn deposit_all(&mut self, buckets: Vec<BID>) {
            for bucket in buckets {
                self.deposit(bucket.into());
            }
        }

        /// Deposits resources into this account.
        pub fn deposit(&mut self, bucket: Bucket) {
            let resource = bucket.resource();
            match self.buckets.get::<Address, Bucket>(&resource) {
                Some(b) => {
                    b.put(bucket);
                }
                None => {
                    let b = Bucket::new(resource);
                    b.put(bucket);
                    self.buckets.insert(resource, b);
                }
            }
        }

        /// Withdraws resources from this account.
        pub fn withdraw(&mut self, amount: U256, resource: Address) -> Bucket {
            let bucket = self.buckets.get::<Address, Bucket>(&resource).unwrap();
            bucket.take(amount)
        }
    }
}
