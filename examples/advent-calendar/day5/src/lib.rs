use scrypto::prelude::*;

blueprint! {
    struct PresentDeliverySchedule {
        // Represent the places Santa must go to, 
        // associated with the epoch at which he has to be there
        // to stay on schedule
        places: HashMap<String, u64>,
        places_done: Vec<String>,
    }

    impl PresentDeliverySchedule {
        pub fn new() -> Component {
            // Configure the places Santa must go to
            let mut places = HashMap::new();
            // Example: Santa must reach Africa before or on epoch 2
            // to stay on schedule
            places.insert("Africa".to_string(), 2);
            places.insert("Asia".to_string(), 5);
            places.insert("Europe".to_string(), 9);
            places.insert("North-America".to_string(), 12);
            places.insert("South-America".to_string(), 14);
            places.insert("Antarctica".to_string(), 16);
            places.insert("Australia".to_string(), 20);

            Self {
                places: places,
                places_done: Vec::new()
            }
            .instantiate()
        }

        /*
         * Displays the remaining places Santa must go
         * to and if he is currently late.
         */
        pub fn display_schedule(&self) {
            let mut late_places: Vec<String> = Vec::new();
            let mut places_to_go: Vec<String> = Vec::new();
            for (continent, epoch_limit) in self.places.iter(){
                if !self.places_done.contains(continent) {
                    places_to_go.push(continent.clone());
                    if Context::current_epoch() > *epoch_limit {
                        late_places.push(continent.clone());
                    }
                }
            }

            info!("Current epoch: {}", Context::current_epoch());

            if late_places.len() > 0 {
                info!("Uwu! You are not on schedule !")
            } else{
                info!("Good job ! You are on schedule.")
            }

            info!("Places left to visit:");
            for continent in places_to_go.iter() {
                info!("{} before epoch {}", continent, self.places.get(continent).unwrap());
            }
        }

        /*
         * Add a continent to the list of
         * visited places
         */
        pub fn add_done(&mut self, continent: String) {
            // Make sure the continent is not yet visited
            assert!(!self.places_done.contains(&continent), "Already visited this continent !");
            // Make sure the continent is valid
            assert!(self.places.contains_key(&continent), "Continent does not exist on this planet !");

            self.places_done.push(continent);
            info!("{}/{} done !", self.places_done.len(), self.places.keys().len());
        }
    }
}
