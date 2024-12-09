use scrypto::prelude::*;

#[blueprint]
mod hello {
    struct Hello {
        // Define what resources and data will be managed by Hello components
        sample_vault: FungibleVault,
    }

    impl Hello {
        // Implement the functions and methods which will manage those resources and data

        // This is a function, and can be called directly on the blueprint once deployed
        pub fn instantiate_hello() -> Global<Hello> {
            // Create a new token called "HelloToken," with a fixed supply of 1000, and put that supply into a bucket
            let my_bucket: FungibleBucket = ResourceBuilder::new_fungible(OwnerRole::None)
                .divisibility(DIVISIBILITY_MAXIMUM)
                .metadata(metadata! {
                    init {
                        "name" => "HelloToken", locked;
                        "symbol" => "HT", locked;
                    }
                })
                .mint_initial_supply(1000)
                .into();

            // Instantiate a Hello component, populating its vault with our supply of 1000 HelloToken
            Self {
                sample_vault: FungibleVault::with_bucket(my_bucket),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::None)
            .globalize()
        }

        // This is a method, because it needs a reference to self.  Methods can only be called on components
        pub fn free_token(&mut self) -> FungibleBucket {
            info!(
                "My balance is: {} HelloToken. Now giving away a token!",
                self.sample_vault.amount()
            );
            // If the semi-colon is omitted on the last line, the last value seen is automatically returned
            // In this case, a bucket containing 1 HelloToken is returned
            self.sample_vault.take(1)
        }
    }
}
