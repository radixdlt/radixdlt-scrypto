#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::string::String;
use scrypto::prelude::*;
use scrypto::{blueprint, import};

import! {
     r#"
        {
            "package_address": "056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7",
            "blueprint_name": "Simple",
            "abi": {
                "structure": {
                    "type": "Struct",
                    "name": "Simple",
                    "fields": {
                        "type": "Named",
                        "named": []
                    }
                },
                "fns": [
                    {
                        "ident": "new",
                        "input": {
                            "type": "Struct",
                            "name": "",
                            "fields": {
                                "type": "Named",
                                "named": []
                            }
                        },
                        "output": {
                            "type": "ComponentAddress"
                        },
                        "export_name": "Simple_new_main"
                    },
                    {
                        "ident": "free_token",
                        "mutability": "Mutable",
                        "input": {
                            "type": "Struct",
                            "name": "",
                            "fields": {
                                "type": "Named",
                                "named": []
                            }
                        },
                        "output": {
                            "type": "Bucket"
                        },
                        "export_name": "Simple_free_token_main"
                    },
                    {
                        "ident": "hash",
                        "mutability": "Mutable",
                        "input": {
                            "type": "Struct",
                            "name": "",
                            "fields": {
                                "type": "Named",
                                "named": []
                            }
                        },
                        "output": {
                            "type": "Hash"
                        },
                        "export_name": "Simple_hash_main"
                    }
                ]
            }
        }
    "#
}

#[blueprint]
mod import {
    struct Import {}

    impl Import {}
}

#[test]
fn test_import_from_abi() {
    let _ = SimpleGlobalComponentRef::from(ComponentAddress::Normal([0; 26]));
    let _: SimpleGlobalComponentRef = ComponentAddress::Normal([0; 26]).into();
}
