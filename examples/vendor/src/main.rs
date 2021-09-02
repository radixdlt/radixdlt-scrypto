#![no_main]

use scrypto::prelude::*;

import! {
r#"
{
    "package": "05a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876",
    "blueprint": "GumballMachine",
    "functions": [
        {
            "name": "new",
            "inputs": [],
            "output": {
                "type": "Custom",
                "name": "scrypto::Address"
            }
        }
    ],
    "methods": [
        {
            "name": "get_gumball",
            "mutability": "Mutable",
            "inputs": [
                {
                    "type": "Custom",
                    "name": "scrypto::Tokens"
                }
            ],
            "output": {
                "type": "Custom",
                "name": "scrypto::Tokens"
            }
        }
    ]
}
"#
}

blueprint! {
    struct Vendor {
        machine: Address
    }

    impl Vendor {
        pub fn new() -> Address {
            let component = Self {
                machine: GumballMachine::new()
            }.instantiate();

            info!("New vendor: {}", component);
            component
        }

        pub fn get_gumball(&self, payment: Tokens) -> Tokens {
            let m = GumballMachine::at(self.machine);
            m.get_gumball(payment)
        }
    }
}
