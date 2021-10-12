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
                "name": "scrypto::types::Address",
                "generics": []
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
                    "name": "scrypto::resource::Bucket",
                    "generics": []
                }
            ],
            "output": {
                "type": "Custom",
                "name": "scrypto::resource::Bucket",
                "generics": []
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
        pub fn new() -> Component {
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
