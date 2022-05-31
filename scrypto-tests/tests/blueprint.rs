#![cfg_attr(not(feature = "std"), no_std)]

use sbor::Type;
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
        pub fn new() -> ComponentAddress {
            Self {
                state: 0
            }
            .instantiate()
            .globalize()
        }

        pub fn get_state(&self) -> u32 {
            self.state
        }

        pub fn set_state(&mut self, new_state: u32) {
            self.state = new_state;
        }

        pub fn custom_types() -> (Decimal, PackageAddress, LazyMap<String, String>, Hash, Bucket, Proof, Vault) {
            todo!()
        }
    }
}

fn assert_json_eq<T: Serialize>(actual: T, expected: Value) {
    assert_eq!(to_value(&actual).unwrap(), expected);
}

#[test]
fn test_simple_abi() {
    let ptr = Simple_abi(core::ptr::null_mut::<u8>(), core::ptr::null_mut::<u8>());
    let abi: (Type, Vec<abi::Function>, Vec<abi::Method>) =
        scrypto_consume(ptr, |slice| scrypto_decode(slice).unwrap());

    assert_json_eq(
        abi,
        json!([
            {
                "fields":{
                    "named":[
                        [
                            "state",
                            { "type":"U32" }
                        ]
                    ],
                    "type":"Named"
                },
                "name":"Simple",
                "type":"Struct"
            },
            [
                {
                    "name": "new",
                    "input": {
                        "type": "Struct",
                        "name": "Simple_new_Input",
                        "fields": {
                            "type": "Named",
                            "named": []
                        }
                    },
                    "output": {
                        "type": "Custom",
                        "name": "ComponentAddress",
                        "generics": []
                    }
                },
                {
                    "name": "custom_types",
                    "input": {
                        "type": "Struct",
                        "name": "Simple_custom_types_Input",
                        "fields": {
                            "type": "Named",
                            "named": []
                        }
                    },
                    "output": {
                        "type": "Tuple",
                        "elements": [
                            {
                                "type": "Custom",
                                "name": "Decimal",
                                "generics": []
                            },
                            {
                                "type": "Custom",
                                "name": "PackageAddress",
                                "generics": []
                            },
                            {
                                "type": "Custom",
                                "name": "LazyMap",
                                "generics": [
                                    {
                                        "type": "String"
                                    },
                                    {
                                        "type": "String"
                                    }
                                ]
                            },
                            {
                                "type": "Custom",
                                "name":  "Hash",
                                "generics": []
                            },
                            {
                                "type": "Custom",
                                "name": "Bucket",
                                "generics": []
                            },
                            {
                                "type": "Custom",
                                "name": "Proof",
                                "generics": []
                            },
                            {
                                "type": "Custom",
                                "name": "Vault",
                                "generics": []
                            }
                        ]
                    }
                }
            ],
            [
                {
                    "name": "get_state",
                    "mutability": "Immutable",
                    "input": {
                        "type": "Struct",
                        "name": "Simple_get_state_Input",
                        "fields": {
                            "type": "Named",
                            "named": []
                        }
                    },
                    "output": {
                        "type": "U32"
                    }
                },
                {
                    "name": "set_state",
                    "mutability": "Mutable",
                    "input": {
                        "type": "Struct",
                        "name": "Simple_set_state_Input",
                        "fields": {
                            "type": "Named",
                            "named": [
                                [
                                    "arg0",
                                    {
                                        "type": "U32"
                                    }
                                ]
                            ]
                        }
                    },
                    "output": {
                        "type": "Unit"
                    }
                }
            ]
        ]),
    );
}
