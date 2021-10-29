use scrypto::prelude::*;

import! {
    r#"
    {
        "package": "013fa22e238526e9c82376d2b4679a845364243bf970e5f783d13f",
        "name": "FlatAdmin",
        "functions": [
          {
            "name": "new",
            "inputs": [
              {
                "type": "String"
              }
            ],
            "output": {
              "type": "Tuple",
              "elements": [
                {
                  "type": "Custom",
                  "name": "scrypto::core::Component",
                  "generics": []
                },
                {
                  "type": "Custom",
                  "name": "scrypto::resource::Bucket",
                  "generics": []
                }
              ]
            }
          }
        ],
        "methods": [
          {
            "name": "create_additional_admin",
            "mutability": "Immutable",
            "inputs": [
              {
                "type": "Custom",
                "name": "scrypto::resource::BucketRef",
                "generics": []
              }
            ],
            "output": {
              "type": "Custom",
              "name": "scrypto::resource::Bucket",
              "generics": []
            }
          },
          {
            "name": "destroy_admin_badge",
            "mutability": "Immutable",
            "inputs": [
              {
                "type": "Custom",
                "name": "scrypto::resource::Bucket",
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
    struct ManagedAccess {
        admin_badge: Address,
        flat_admin_controller: Address,
        protected_vault: Vault
    }

    impl ManagedAccess {
        pub fn new() -> (Component, Bucket) {
            let (flat_admin_component, admin_badge) = FlatAdmin::new("My Managed Access Badge".into());                        
            
            let component = Self {
                admin_badge: admin_badge.resource_def().address(),
                flat_admin_controller: flat_admin_component.address(),
                protected_vault: Vault::new(RADIX_TOKEN)
            }
            .instantiate();
            (component, admin_badge)
        }

        #[auth(admin_badge)]
        pub fn withdraw_all(&mut self) -> Bucket {
            self.protected_vault.take_all()
        }

        pub fn deposit(&mut self, to_deposit: Bucket) {
            self.protected_vault.put(to_deposit);
        }

        pub fn get_admin_badge_address(&mut self) -> Address {
            self.admin_badge
        }

        pub fn get_flat_admin_controller_address(&mut self) -> Address {
            self.flat_admin_controller
        }
    }
}
