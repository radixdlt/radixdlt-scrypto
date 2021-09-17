use scrypto::prelude::*;

blueprint! {
    struct Account {
        resources: Storage,
    }

    impl Account {
        pub fn new() -> Address {
            Account {
                resources: Storage::new(),
            }
            .instantiate()
        }

        //===================
        // public methods //
        //===================

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
            let resource = Resource::new_mutable( metadata, minter);
            resource.into()
        }

        /// Creates a resource with fixed supply, which will be deposited into this account.
        pub fn new_resource_fixed(
            &mut self,
            metadata: HashMap<String, String>,
            supply: U256,
        ) -> Address {
            let bucket: BID = Resource::new_fixed(metadata, supply);
            let address = Bucket::resource(&bucket);
            self.deposit(bucket);
            address
        }

        /// Mint resources and deposit it into this account.
        pub fn mint_resource(&mut self, amount: U256, resource: Address)  {
            let bucket: BID = Resource::from(resource).mint(amount);
            self.deposit(bucket);
        }

        /// Deposit a collection of buckets into this account
        pub fn deposit_all(&mut self, buckets: Vec<BID>) {
            for bucket in buckets {
                self.deposit(bucket);
            }
        }

        /// Deposit tokens into this account
        pub fn deposit_tokens(&mut self, tokens: Tokens) {
            self.deposit(tokens.into());
        }

        /// Deposit badges into this account
        pub fn deposit_badges(&mut self, badges: Badges) {
            self.deposit(badges.into());
        }

        /// Withdraw tokens from this account
        pub fn withdraw_tokens(&mut self, amount: U256, resource: Address) -> Tokens {
          self.withdraw(amount, resource).into()
        }

        /// Withdraw badges from this account
        pub fn withdraw_badges(&mut self, amount: U256, resource: Address) -> Badges {
            self.withdraw(amount, resource).into()
        }

        //===================
        // private methods //
        //===================

        fn deposit(&mut self, bucket: BID) {
            let resource = bucket.resource();
            match self.resources.get::<Address, BID>(&resource) {
                Some(b) => {
                    b.put(bucket);
                }
                None => {
                    let b = BID::new(resource);
                    b.put(bucket);
                    self.resources.insert(resource, b);
                }
            }
        }

        fn withdraw(&mut self, amount: U256, resource: Address) -> BID {
            let bucket = self.resources.get::<Address, BID>(&resource).unwrap();
            bucket.take(amount)
        }
    }
}
