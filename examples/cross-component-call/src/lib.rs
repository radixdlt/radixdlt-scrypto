use scrypto::prelude::*;

import! {
r#"
{
    "package": "01a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876",
    "name": "GumballMachine",
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
                    "name": "scrypto::Bucket"
                }
            ],
            "output": {
                "type": "Custom",
                "name": "scrypto::Bucket"
            }
        }
    ]
}
"#
}

blueprint! {
    struct Vendor {
        machine: GumballMachine
    }

    impl Vendor {
        pub fn new() -> Address {
            Self {
                machine: GumballMachine::new().into()
            }
            .instantiate()
        }

        pub fn get_gumball(&self, payment: Bucket) -> Bucket {
            self.machine.get_gumball(payment)
        }
    }
}
