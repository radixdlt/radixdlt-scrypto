use scrypto::prelude::*;

blueprint! {
    struct SantaCookieEater {
        // Will be used to store the cookies
        // you give to this component
        cookie_vault: Vault
    }

    impl SantaCookieEater {
        pub fn new() -> (Component, Bucket) {
            // Create 1000 "Cookie" tokens
            let cookie_bucket = ResourceBuilder::new_fungible(DIVISIBILITY_MAXIMUM)
                .metadata("name", "Cookie")
                .metadata("symbol", "CKE")
                .initial_supply_fungible(1000);

            // Instantiate the component with an empty vault of "Cookie" tokens
            let component = Self {
                cookie_vault: Vault::new(cookie_bucket.resource_def())
            }
            .instantiate();

            // return the component and the 1000 Cookie tokens
            (component, cookie_bucket)
        }

        // Give tokens to the component
        pub fn give_food(&mut self, food: Bucket) {
            // Make sure the provided tokens are Cookies
            assert!(food.resource_def() == self.cookie_vault.resource_def(), "No ! I want Cookies !");

            // Insert the tokens in the component's vault
            self.cookie_vault.put(food);
            info!("Thank you ! Very tasty !");
        }
    }
}
