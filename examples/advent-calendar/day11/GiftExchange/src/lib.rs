use scrypto::prelude::*;

// Import the price oracle blueprint
// Change the "change_me" text with the price oracle package address.
// Example:
// {
//   "package": "013fa22e238526e9c82376d2b4679a845364243bf970e5f783d13f"
//   "name": "PriceOracle"
//   ...
import! {
    r#"
    {
      "package": "change_me",
      "name": "PriceOracle",
      "functions": [
        {
          "name": "new",
          "inputs": [],
          "output": {
            "type": "Custom",
            "name": "scrypto::core::Component",
            "generics": []
          }
        }
      ],
      "methods": [
        {
          "name": "get_price",
          "mutability": "Immutable",
          "inputs": [
            {
              "type": "Custom",
              "name": "scrypto::types::Address",
              "generics": []
            },
            {
              "type": "Custom",
              "name": "scrypto::types::Address",
              "generics": []
            }
          ],
          "output": {
            "type": "Option",
            "value": {
              "type": "Custom",
              "name": "scrypto::types::Decimal",
              "generics": []
            }
          }
        },
        {
          "name": "get_usd_address",
          "mutability": "Immutable",
          "inputs": [],
          "output": {
            "type": "Custom",
            "name": "scrypto::types::Address",
            "generics": []
          }
        },
        {
          "name": "update_price",
          "mutability": "Immutable",
          "inputs": [
            {
              "type": "Custom",
              "name": "scrypto::types::Address",
              "generics": []
            },
            {
              "type": "Custom",
              "name": "scrypto::types::Address",
              "generics": []
            },
            {
              "type": "Custom",
              "name": "scrypto::types::Decimal",
              "generics": []
            }
          ],
          "output": {
            "type": "Unit"
          }
        }
      ]
    }
    "#
    }

blueprint! {
    struct GiftExchange {
      // Will store the price oracle component
      price_oracle: PriceOracle,
      // Keep track of the participants
      participants: Vec<Address>,
      // Keep track of who should give to who
      who_to_who: HashMap<Address, Address>,
      // Indicates if the component decided who is going to give to who
      decided: bool,
      // Used to protect methods on this blueprint
      organizer_def: ResourceDef,
    }

    impl GiftExchange {
        pub fn new(price_oracle_address: Address) -> (Component, Bucket) {
            // Create the organizer badge.
            // Used to protect the `add_participant` and `prepare_exchange` methods
            let organizer_badge = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                    .metadata("name", "Organizer Badge")
                                    .initial_supply_fungible(1);

            let component = Self {
                price_oracle: price_oracle_address.into(),
                participants: Vec::new(),
                who_to_who: HashMap::new(),
                decided: false,
                organizer_def: organizer_badge.resource_def(),
            }
            .instantiate();

            // Return the instantiated component and organizer's badge
            (component, organizer_badge)
        }

        // As organizer, add a participant to the gift exchange
        #[auth(organizer_def)]
        pub fn add_participant(&mut self, address: Address) {
            assert!(!self.decided, "Component already decided who would give presents to who !");

            // Create the participant's badge, used
            // as identification in `send_gift` method
            let participant_badge =  ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                        .metadata("name", "Participant Badge")
                                        .metadata("account", format!("{}", address))
                                        .initial_supply_fungible(1);

            self.participants.push(participant_badge.resource_address());

            // Send the badge to the participant
            Account::from(address).deposit(participant_badge);
        }

        // Organizer can call this method after adding the participants
        // to decide who should give to who.
        #[auth(organizer_def)]
        pub fn prepare_exchange(&mut self) {
            assert!(self.participants.len() >= 2, "Add at least two participants first !");
            assert!(self.participants.len() % 2 == 0, "Need to have even number of participants !");
            assert!(!self.decided, "Component already decided who would give presents to who !");

            let amount_to_slice = self.participants.len() / 2;

            for i in 0..amount_to_slice {
                let from = self.participants.get(i).unwrap();
                let to = self.participants.get(i + amount_to_slice).unwrap();
                
                info!("{} is giving to {}", from, to);
                info!("{} is giving to {}", to, from);

                self.who_to_who.insert(*from, *to);
                self.who_to_who.insert(*to, *from);
            }

            // Set to true so that no one can call `make_exchange` and `add_participant` anymore
            self.decided = true;
        }

        // Allow participants to send their gift.
        // They only have to provide their badge. The destination is
        // fetched from the `who_to_who` map.
        pub fn send_gift(&self, gift: Bucket, your_badge: BucketRef) {
            assert!(your_badge.amount() > Decimal::zero(), "Missing badge");
            assert!(self.decided, "You have to call `make_exchange` first to decide who should give to who.");
            assert!(self.who_to_who.contains_key(&your_badge.resource_address()), "Captain. What should we do? He's not on the list");

            let to_resource = ResourceDef::from(*self.who_to_who.get(&your_badge.resource_address()).unwrap());
            your_badge.drop();

            // Make sure the provided gift price is less than 20$
            match self.price_oracle.get_price(gift.resource_address(), self.price_oracle.get_usd_address()) {
                Some(price) => {
                    if price > 20.into() {
                        info!("Gift is too expensive for the exchange ! Consider creating a YankeeSwap component instead");
                        std::process::abort();
                    }
                },
                None => {
                    info!("Price of {} unknown", gift.resource_def().metadata().get("name").unwrap());
                    std::process::abort();
                }
            };

            // Fetch the address from the metadata
            let to_address: Address = Address::from_str(to_resource.metadata().get("account").unwrap()).unwrap();

            // Deposit the gift into the recipient's account
            Account::from(to_address).deposit(gift);
        }
    }
}
