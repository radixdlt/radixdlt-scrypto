use scrypto::prelude::*;

blueprint! {
    struct YankeeSwap {
        // Used to generate "random" numbers
        random_seed: i32,
        nonce: i32,

        // Keep track of the different gifs
        gifts: Vec<Vault>,
        // Keep track of the participant badges
        participants: Vec<Address>,
        // Maps the index of the gift to the participant's badge
        gift_participant: HashMap<u32, Option<Address>>,
        // Definition of the admin badge, allowing us to secure some methods
        admin_badge_def: ResourceDef,

        // Store the current state of the game
        started: bool,
        ended: bool,
        current_gift_index: usize,
    }

    impl YankeeSwap {
        pub fn new() -> (Component, Bucket) {
            // Create the admin badge
            let admin_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                .metadata("name", "Admin")
                                .initial_supply_fungible(1);

            let component = Self {
                random_seed: Self::generate_seed(),
                nonce: i32::MAX / 2,
                gifts: Vec::new(),
                participants: Vec::new(),
                gift_participant: HashMap::new(),
                admin_badge_def: admin_badge.resource_def(),
                started: false,
                ended: false,
                current_gift_index: 0
            }
            .instantiate();

            // Return the component and admin_badge to the caller
            (component, admin_badge)
        }

        // Display the current state of the game and
        // the available gifts.
        pub fn current_gift(&self) {
            info!("--- Yankee Swap ---");
            let current_participant = self.participants.get(self.current_gift_index).unwrap();
            info!("{} picked a {}. Will they decide to keep or swap it ?", 
                            current_participant, 
                            self.gifts.get(self.current_gift_index).unwrap().resource_def().metadata().get("name").unwrap()
                );
            
            info!("All gifts: ");
            for (i, participant) in self.gift_participant.iter() {
                if participant.is_some() && *i < self.current_gift_index as u32 {
                    info!("{} - {}", i, self.gifts.get(*i as usize).unwrap().resource_def().metadata().get("name").unwrap());
                }
            }
        }

        // Allow anybody to enter the game.
        // They must send a bucket containing the gift to contribute.
        pub fn enter_swap(&mut self, gift: Bucket) -> Bucket {
            // Make sure the game is not already started
            assert!(!self.started, "Game already started !");

            // Create a new badge that will allow us to
            // identify the users
            let ticket = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                            .metadata("name", "YankeeSwap Ticket")
                            .initial_supply_fungible(1);

            
            self.gift_participant.insert(self.gifts.len() as u32, None);

            // Store the gift in a new vault because vaults can only 
            // store one kind of token.
            self.gifts.push(Vault::with_bucket(gift));

            // Add the badge to the list of participants
            self.participants.push(ticket.resource_address());

            // Return the badge to the caller
            ticket
        }

        // Allow the current player to swap their gift with another one
        // at specified index.
        pub fn swap(&mut self, with_index: u32, participant_badge: BucketRef) {
            assert!(participant_badge.amount() > Decimal::zero(), "Missing badge");
            assert!(*self.participants.get(self.current_gift_index).unwrap() == participant_badge.resource_address(), "It's not your turn !");

            // Swap the owner of the gifts
            self.gift_participant.insert(self.current_gift_index as u32, *self.gift_participant.get(&with_index).unwrap());
            self.gift_participant.insert(with_index, Some(participant_badge.resource_address()));
            participant_badge.drop();

            self.next_turn();
        }

        // Allow the current player to keep their gift
        pub fn keep(&mut self, participant_badge: BucketRef) {
            assert!(participant_badge.amount() > Decimal::zero(), "Missing badge");
            assert!(*self.participants.get(self.current_gift_index).unwrap() == participant_badge.resource_address(), "It's not your turn !");
            participant_badge.drop();

            self.next_turn();
        }
        
        // After the game is ended, participants can call
        // this method to withdraw their gift.
        pub fn withdraw(&self, participant_badge: BucketRef) -> Bucket {
            assert!(self.ended, "Game not completed !");

            for (i, badge) in self.gift_participant.iter() {
                if badge.unwrap() == participant_badge.resource_address() {
                    participant_badge.drop();
                    return self.gifts.get(*i as usize).unwrap().take_all();
                }
            }

            info!("You are not participating in this game !");
            std::process::abort();
        }

        // Call this method with the admin badge to
        // start the game.
        #[auth(admin_badge_def)]
        pub fn start(&mut self) {
            assert!(!self.started, "Already started !");

            // Reorder the list of participants "randomly"
            let mut participants = self.participants.clone();
            let mut ordered_list: Vec<Address> = Vec::new();

            for _ in 0..self.participants.len() {
                let random_index = self.random_number(0, participants.len() as i32);
                ordered_list.push(*participants.get(random_index).unwrap());
                info!("{}", *participants.get(random_index).unwrap());
                participants.remove(random_index);
            }

            self.participants = ordered_list;
            self.started = true;

            self.gift_participant.insert(0, Some(*self.participants.get(0).unwrap()));
            self.gift_participant.insert(1, Some(*self.participants.get(1).unwrap()));
            self.current_gift_index += 1;
        }

        // Move to the next turn or end the game.
        fn next_turn(&mut self) {
            if self.current_gift_index < self.participants.len() - 1 {
                info!("Next person's turn !");
                self.current_gift_index += 1;
                self.gift_participant.insert(self.current_gift_index as u32, Some(*self.participants.get(self.current_gift_index).unwrap()));
            } else {
                info!("Swap ended !");
                for (i, badge) in self.gift_participant.iter() {
                    info!("{} has {}", badge.unwrap(), self.gifts.get(*i as usize).unwrap().resource_def().metadata().get("name").unwrap());
                }
                self.ended = true;
            }
        }

        // Generate the seed for random number generation
        fn generate_seed() -> i32 {
            let mut seed: i32 = 1;
            for byte in Context::transaction_signers()[0].to_vec().iter() {
                if (seed * *byte as i32) != 0 {
                    seed *= *byte as i32;
                }
            }
            seed
        }

        // Generate a random number
        // WARNING: DON'T USE THIS IN PRODUCTION !
        pub fn random_number(&mut self, min: i32, max: i32) -> usize {
            let mut random_number: Decimal = (self.random_seed * (self.nonce as i32)).into();
            random_number = (random_number.abs() / i32::MAX) * (max - min) + min;

            self.nonce += 1;
            random_number.to_string().split(".").collect::<Vec<&str>>().get(0).unwrap().parse().unwrap()
        }
    }
}
