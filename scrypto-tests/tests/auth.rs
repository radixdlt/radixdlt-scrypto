#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::abi;
use scrypto::buffer::*;
use scrypto::prelude::*;
use serde::Serialize;
use serde_json::{json, to_value, Value};

blueprint! {
    struct SimpleAuth {
        admin: ResourceDefRef,
        user: ResourceDefRef,
        reserves: Vault,
    }

    impl SimpleAuth {
        pub fn new(admin: ResourceDefRef, user: ResourceDefRef) -> ComponentRef {
            Self {
                admin,
                user,
                reserves: Vault::new(RADIX_TOKEN),
            }
            .instantiate()
        }

        #[auth(admin)]
        pub fn pump(&mut self, xrd: Bucket) {
            self.reserves.put(xrd);
        }

        #[auth(admin, user)]
        pub fn airdrop(&mut self) -> Bucket {
            self.reserves.take(1)
        }

        #[auth(admin, user)]
        pub fn airdrop_mut(&mut self) -> Bucket {
            return self.mut_take(); // tests both return and &mut self with auth
        }

        fn mut_take(&mut self) -> Bucket {
            self.reserves.take(1)
        }
    }
}

fn assert_json_eq<T: Serialize>(actual: T, expected: Value) {
    assert_eq!(to_value(&actual).unwrap(), expected);
}

#[test]
fn test_simple_auth() {
    let ptr = SimpleAuth_abi();
    let abi: (Vec<abi::Function>, Vec<abi::Method>) =
        unsafe { scrypto_consume(ptr, |slice| scrypto_decode(slice).unwrap()) };

    assert_json_eq(
        abi,
        json!([
            [
                {
                    "name": "new",
                    "inputs": [
                        {
                            "type": "Custom",
                            "name": "scrypto::resource::ResourceDefRef",
                            "generics": []
                        },
                        {
                            "type": "Custom",
                            "name": "scrypto::resource::ResourceDefRef",
                            "generics": []
                        }
                    ],
                    "output": {
                        "type": "Custom",
                        "name": "scrypto::core::ComponentRef",
                        "generics": []
                    }
                }
            ],
            [
                {
                    "name": "pump",
                    "mutability": "Mutable",
                    "inputs": [
                        {
                            "type": "Custom",
                            "name": "scrypto::resource::Bucket",
                            "generics": []
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
                    "name": "airdrop",
                    "mutability": "Mutable",
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
                    "name": "airdrop_mut",
                    "mutability": "Mutable",
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
                }
            ]
        ]),
    );
}
