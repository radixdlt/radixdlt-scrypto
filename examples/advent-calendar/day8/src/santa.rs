use scrypto::prelude::*;
use crate::house::House;

blueprint! {
    struct Santa {
        // List of House components
        houses: Vec<House>,
        // Badges required to access the houses
        keys: Vec<Vault>,
        // Gift vault used to put gifts under the houses trees
        gifts: Vault
    }

    impl Santa {
        pub fn new() -> Component {
            // Create the tokens that will represent the gifts
            let gifts = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                            .metadata("name", "Gift")
                            .initial_supply_fungible(8000);

            // Instantiate the 10 house components
            let mut houses: Vec<House> = Vec::new();
            let mut keys: Vec<Vault> = Vec::new();

            for _ in 0..10 {
                let (component, key) = House::new(gifts.resource_def());
                houses.push(component.into());
                keys.push(Vault::with_bucket(key));
            }

            Self {
                houses: houses,
                keys: keys,
                gifts: Vault::with_bucket(gifts)
            }
            .instantiate()
        }

        // Take the milk and cookies from the house at the specified index.
        // Then put a gift under the house's Christmas tree
        pub fn go_into_house(&self, house_index: usize) -> (Bucket, Bucket) {

            let (cookies, milk) = match self.houses.get(house_index) {
                Some(house) => {
                    // Get the key and use the authorize method.
                    // The authorize method takes a bucket from the key vault,
                    // gives access to its BucketRef and then put the bucket back into the vault.
                    let (cookies, milk) = self.keys.get(house_index).unwrap().authorize(|key_bucket_ref| {
                        // Take the cookies and milk
                        house.get_milk_and_cookie(key_bucket_ref)
                    });

                    // At the moment, we have to do multiple authorize because the key_bucket_ref is moved
                    self.keys.get(house_index).unwrap().authorize(|key_bucket_ref| {
                        // Put gift under the tree
                        house.give_gift(self.gifts.take(1), key_bucket_ref);
                    });

                    (cookies, milk)
                },
                None => {
                    // House not found with the provided house_index
                    info!("Invalid house index !");
                    std::process::abort();
                }
            };

            // Return the cookies and milk to the caller
            (cookies, milk)
        }
    }
}
