use scrypto::prelude::*;

// Change the "change_me" text with the present list package address.
// Example:
// {
//   "package": "013fa22e238526e9c82376d2b4679a845364243bf970e5f783d13f"
//   "name": "PresentList"
//   ...
import! {
    r#"
    {
        "package": "01bfe0f41f7e8ff54cc942b4dbfee48349563517a765579b8e4ae8",
        "name": "PresentList",
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
            "name": "start_new_list",
            "mutability": "Mutable",
            "inputs": [],
            "output": {
              "type": "Custom",
              "name": "scrypto::resource::Bucket",
              "generics": []
            }
          },
          {
            "name": "add",
            "mutability": "Mutable",
            "inputs": [
              {
                "type": "String"
              },
              {
                "type": "Custom",
                "name": "scrypto::resource::BucketRef",
                "generics": []
              }
            ],
            "output": {
              "type": "Unit"
            }
          },
          {
            "name": "remove",
            "mutability": "Mutable",
            "inputs": [
              {
                "type": "String"
              },
              {
                "type": "Custom",
                "name": "scrypto::resource::BucketRef",
                "generics": []
              }
            ],
            "output": {
              "type": "Unit"
            }
          },
          {
            "name": "display_list",
            "mutability": "Immutable",
            "inputs": [
              {
                "type": "Custom",
                "name": "scrypto::resource::BucketRef",
                "generics": []
              }
            ],
            "output": {
              "type": "Unit"
            }
          },
          {
            "name": "get_lists",
            "mutability": "Immutable",
            "inputs": [],
            "output": {
              "type": "HashMap",
              "key": {
                "type": "Custom",
                "name": "scrypto::types::Address",
                "generics": []
              },
              "value": {
                "type": "Vec",
                "element": {
                  "type": "String"
                }
              }
            }
          }
        ]
      }
    "#
    }

blueprint! {
    struct Santa {
        present_list: PresentList,
        presents: HashMap<Address, Vec<Vault>>
    }

    impl Santa {
        pub fn new(present_list_component: Address) -> Component {
            // Convert the address into a PresentList component
            let present_list: PresentList = present_list_component.into();

            Self {
                present_list: present_list,
                presents: HashMap::new()
            }.instantiate()
        }

        // Create tokens for every present in the list
        // and associate them with the recipient's badge
        pub fn prepare_gifts(&mut self) {
            let lists: HashMap<Address, Vec<String>> = self.present_list.get_lists();
            for (badge_address, gifts) in lists {
                // Retrieve the list of vaults for that particular badge's address.
                // If not present, create entry with empty vec
                let vaults = self.presents.entry(badge_address).or_insert(Vec::new());

                // Create the tokens that will act as gifts
                for gift in gifts {
                    let resource = ResourceBuilder::new_fungible(DIVISIBILITY_NONE)
                                    .metadata("name", format!("{}", gift))
                                    .initial_supply_fungible(1);
                    vaults.push(Vault::with_bucket(resource));
                }
            }
        }

        // Allow people to withdraw their gifts.
        // They use the same badge as the one for their present list
        pub fn withdraw_gifts(&self, badge: BucketRef) -> Vec<Bucket> {
            assert!(badge.amount() > Decimal::zero());

            let mut buckets: Vec<Bucket> = Vec::new();
            match self.presents.get(&badge.resource_address()) {
                Some(gifts) => {
                    for gift in gifts {
                        buckets.push(gift.take_all())
                    }
                },
                None => {
                    info!("Badge is invalid !");
                    std::process::abort();
                }
            };

            badge.drop();
            buckets
        }
    }
}
