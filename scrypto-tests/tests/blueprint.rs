#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::abi;
use scrypto::buffer::*;
use scrypto::prelude::*;
use serde::Serialize;
use serde_json::{json, to_value, Value};

blueprint! {
    struct Simple {
        state: u32,
    }

    impl Simple {
        pub fn new() -> Address {
            Self {
                state: 0
            }.instantiate()
        }

        pub fn get_state(&self) -> u32 {
            self.state
        }

        pub fn set_state(&mut self, new_state: u32) {
            self.state = new_state;
        }

        pub fn custom_types_1() -> (Amount, Address, H256, BID, RID, SID, VID) {
            todo!()
        }

        pub fn custom_types_2() -> (Package, Blueprint, Component, Storage) {
            todo!()
        }

        pub fn custom_types_3() -> (Bucket, BucketRef, Vault) {
            todo!()
        }
    }
}

fn assert_json_eq<T: Serialize>(actual: T, expected: Value) {
    assert_eq!(to_value(&actual).unwrap(), expected);
}

#[test]
fn test_simple_abi() {
    let ptr = Simple_abi();
    let abi: (Vec<abi::Function>, Vec<abi::Method>) =
        unsafe { scrypto_consume(ptr, |slice| scrypto_decode(slice).unwrap()) };

    assert_json_eq(
        abi,
        json!([
            [
                {
                    "name": "new",
                    "inputs": [],
                    "output": {
                        "type": "Custom",
                        "name": "scrypto::Address"
                    }
                },
                {
                    "name": "custom_types_1",
                    "inputs": [],
                    "output": {
                        "type": "Tuple",
                        "elements": [
                            {
                                "type": "Custom",
                                "name": "scrypto::Amount"
                            },
                            {
                                "type": "Custom",
                                "name": "scrypto::Address"
                            },
                            {
                                "type": "Custom",
                                "name": "scrypto::H256"
                            },
                            {
                                "type": "Custom",
                                "name": "scrypto::BID"
                            },
                            {
                                "type": "Custom",
                                "name": "scrypto::RID"
                            },
                            {
                                "type": "Custom",
                                "name": "scrypto::SID"
                            },
                            {
                                "type": "Custom",
                                "name": "scrypto::VID"
                            }
                        ]
                    }
                },
                {
                    "name": "custom_types_2",
                    "inputs": [],
                    "output": {
                        "type": "Tuple",
                        "elements": [
                            {
                                "type": "Custom",
                                "name": "scrypto::Package"
                            },
                            {
                                "type": "Custom",
                                "name": "scrypto::Blueprint"
                            },
                            {
                                "type": "Custom",
                                "name": "scrypto::Component"
                            },
                            {
                                "type": "Custom",
                                "name": "scrypto::Storage"
                            }
                        ]
                    }
                },
                {
                    "name": "custom_types_3",
                    "inputs": [],
                    "output": {
                        "type": "Tuple",
                        "elements": [
                            {
                                "type": "Custom",
                                "name": "scrypto::Bucket"
                            },
                            {
                                "type": "Custom",
                                "name": "scrypto::BucketRef"
                            },
                            {
                                "type": "Custom",
                                "name": "scrypto::Vault"
                            }
                        ]
                    }
                }
            ],
            [
                {
                    "name": "get_state",
                    "mutability": "Immutable",
                    "inputs": [],
                    "output": {
                        "type": "U32"
                    }
                },
                {
                    "name": "set_state",
                    "mutability": "Mutable",
                    "inputs": [
                        {
                            "type": "U32"
                        }
                    ],
                    "output": {
                        "type": "Unit"
                    }
                }
            ]
        ]),
    );
}
