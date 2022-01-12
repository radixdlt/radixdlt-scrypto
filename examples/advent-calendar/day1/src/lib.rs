use scrypto::prelude::*;

/*
 * PresentFactory.
 * Component allowing users to create new tokens representing presents.
 */ 
blueprint! {
    struct PresentFactory {
        // A vault can only hold one type of token.
        // I use a Hashmap mapping present names to vaults containing the presents.
        presents: HashMap<String, Vault>
    }

    impl PresentFactory {
        pub fn new() -> Component {
            Self {
                // Initiate the HashMap as empty
                presents: HashMap::new()
            }
            .instantiate()
        }

        /*
         * Allow caller to create a new present with specified name and quantity
         */
        pub fn create_present(&mut self, name: String, quantity: u64) {
            assert!(!self.presents.contains_key(&name), "Present already exist !");

            // Create the present token
            let bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", &name)
                .initial_supply_fungible(quantity);

            // Store inside the present list
            self.presents.insert(name, Vault::with_bucket(bucket));
        }

        /*
         * Used to display the list of presents
         */
        pub fn list_presents(&self) {
            info!("{} presents", self.presents.len());
            info!("==========");

            for (name, vault) in &self.presents {
                info!("{} {}", vault.amount(), name);
            }
        }
    }
}
