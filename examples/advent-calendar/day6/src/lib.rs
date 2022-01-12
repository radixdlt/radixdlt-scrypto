use scrypto::prelude::*;

blueprint! {
    struct PresentList {
        // Used to store the presents in the list for every account
        lists: HashMap<Address, Vec<String>>,
    }

    impl PresentList {
        pub fn new() -> Component {
            // Store all required information info the component's state
            Self {
                lists: HashMap::new()
            }
            .instantiate()
        }
        
        // Allow the user to start a new christmas list.
        // It generates a badge that will allow users to add and remove
        // presents associated with it.
        pub fn start_new_list(&mut self) -> Bucket {
            // Mint a new christmas list badge

            // Edit 2021-12-15: Since Alexandria, this should be done with NFTs
            let list_bucket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                .metadata("name", format!("Christmas List ID #{}", self.lists.len() + 1))
                .initial_supply_fungible(1);

            // Store an empty list for the badge's address in the lists map
            self.lists.insert(list_bucket.resource_address(), vec![]);

            // Return the list badge to the caller
            list_bucket
        }

        // Add a new present to the list
        pub fn add(&mut self, present_name: String, list_badge: BucketRef) {
            let list_address = self.get_list_id(list_badge);
            let list = self.lists.get(&list_address).unwrap();

            // Make sure that the present is not already inside the user's list
            assert!(!list.contains(&present_name), "Present already on the list !");

            let mut presents = list.clone();
            presents.push(present_name);

            // Update the list with the newly added present
            self.lists.insert(list_address, presents);
            info!("Present added to your list !");
        }

        // Remove a present in the list
        pub fn remove(&mut self, present_name: String, list_badge: BucketRef) {
            let list_address = self.get_list_id(list_badge);
            let list = self.lists.get(&list_address).unwrap();

            // Make sure that the present is not already inside the user's list
            assert!(list.contains(&present_name), "Present not on the list !");

            let mut presents = list.clone();
        
            // Find the index of the present to remove
            let index = presents.iter().position(|x| *x == present_name).unwrap();
            presents.remove(index);

            // Update the list with the present removed
            self.lists.insert(list_address, presents);
        }

        // Display the presents stored in the list
        // associated with the list badge
        pub fn display_list(&self, list_badge: BucketRef) {
            let list_address = self.get_list_id(list_badge);
            let list = self.lists.get(&list_address).unwrap();

            info!("==== Christmas list content");
            for item in list.iter() {
                info!("{}", item);
            }
        }

        // Private method to get the badge's address while doing
        // some assertions on the passed badge instead of 
        // only taking the badge.resource_address()
        fn get_list_id(&self, badge: BucketRef) -> Address {
            let address = badge.resource_address();

            // Make sure that the provided badge quantity is greater than 0
            assert!(badge.amount() > Decimal::zero(), "You have to pass the list badge!");
            assert!(self.lists.contains_key(&address));

            // Drop the badge since we don't need it anymore or else we will get an error
            badge.drop();

            // Return the list
            address
        }
    }
}