use scrypto::prelude::*;

import! {
r#"
{
    "package": "01bda8686d6c2fa45dce04fac71a09b54efbc8028c23aac74bc00e",
    "name": "Airdrop",
    "functions": [
        {
            "name": "instantiate_airdrop",
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
            "name": "free_token",
            "mutability": "Immutable",
            "inputs": [],
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
    struct Proxy1 {
        airdrop: Airdrop,
    }

    impl Proxy1 {
        pub fn instantiate_proxy() -> Component {
            Self {
                // The instantiate_airdrop() function returns a generic Component. We use `.into()` to convert it into an `Airdrop`.
                airdrop: Airdrop::instantiate_airdrop().into(),
            }
            .instantiate()
        }

        pub fn free_token(&self) -> Bucket {
            // Calling a method on a component using `.method_name()`.
            self.airdrop.free_token()
        }
    }
}
