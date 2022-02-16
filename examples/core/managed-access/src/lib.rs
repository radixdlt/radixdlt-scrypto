use scrypto::prelude::*;

import! {
r#"
{
    "package": "01ca59a8d6ea4f7efa1765cef702d14e47570c079aedd44992dd09",
    "name": "FlatAdmin",
    "functions": [
        {
            "name": "instantiate_flat_admin",
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
        },
        {
            "name": "get_admin_badge_address",
            "mutability": "Immutable",
            "inputs": [],
            "output": {
                "type": "Custom",
                "name": "scrypto::types::Address",
                "generics": []
            }
        }
    ]
}
"#
}

blueprint! {
    struct ManagedAccess {
        admin_badge: ResourceDef,
        flat_admin_controller: Address,
        protected_vault: Vault,
    }

    impl ManagedAccess {
        pub fn instantiate_managed_access() -> (Component, Bucket) {
            let (flat_admin_component, admin_badge) =
                FlatAdmin::instantiate_flat_admin("My Managed Access Badge".into());

            let component = Self {
                admin_badge: admin_badge.resource_def(),
                flat_admin_controller: flat_admin_component.address(),
                protected_vault: Vault::new(RADIX_TOKEN),
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

        pub fn get_admin_badge_address(&self) -> Address {
            self.admin_badge.address()
        }

        pub fn get_flat_admin_controller_address(&self) -> Address {
            self.flat_admin_controller
        }
    }
}
