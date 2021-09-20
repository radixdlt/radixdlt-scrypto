#![cfg_attr(not(feature = "std"), no_std)]

use scrypto::import;
use scrypto::rust::str::FromStr;
use scrypto::rust::string::String;
use scrypto::rust::string::ToString;
use scrypto::rust::vec::Vec;
use scrypto::types::*;

// base directory: `scrypto-derive`
import! {
r#"
{
    "package": "056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7",
    "blueprint": "Sample",
    "functions": [
        {
            "name": "stateless_func",
            "inputs": [],
            "output": {
                "type": "U32"
            }
        },
        {
            "name": "test_custom_types",
            "inputs": [
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
                },
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
                },
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
            ],
            "output": {
                "type": "Custom",
                "name": "scrypto::BucketRef"
            }
        }
    ],
    "methods": [
        {
            "name": "calculate_volume",
            "mutability": "Immutable",
            "inputs": [
                {
                    "type": "Struct",
                    "name": "Floor",
                    "fields": {
                        "type": "Named",
                        "named": [
                            [
                                "x",
                                {
                                    "type": "U32"
                                }
                            ],
                            [
                                "y",
                                {
                                    "type": "U32"
                                }
                            ]
                        ]
                    }
                },
                {
                    "type": "Tuple",
                    "elements": [
                        {
                            "type": "U8"
                        },
                        {
                            "type": "U16"
                        }
                    ]
                },
                {
                    "type": "Vec",
                    "element": {
                        "type": "String"
                    }
                },
                {
                    "type": "U32"
                },
                {
                    "type": "Enum",
                    "name": "Hello",
                    "variants": [
                        {
                            "name": "A",
                            "fields": {
                                "type": "Named",
                                "named": [
                                    [
                                        "x",
                                        {
                                            "type": "U32"
                                        }
                                    ]
                                ]
                            }
                        },
                        {
                            "name": "B",
                            "fields": {
                                "type": "Unnamed",
                                "unnamed": [
                                    {
                                        "type": "U32"
                                    }
                                ]
                            }
                        },
                        {
                            "name": "C",
                            "fields": {
                                "type": "Unit"
                            }
                        }
                    ]
                },
                {
                    "type": "Array",
                    "element": {
                        "type": "String"
                    },
                    "length": 2
                }
            ],
            "output": {
                "type": "U32"
            }
        }
    ]
}
"#
}

#[test]
#[should_panic] // asserts it compiles
fn test_import_from_abi() {
    let instance = Sample::instance(Address::from_str("").unwrap());

    let arg1 = Floor { x: 5, y: 12 };
    let arg2 = (1u8, 2u16);
    let arg3 = Vec::<String>::new();
    let arg4 = 5;
    let arg5 = Hello::A { x: 1 };
    let arg6 = ["a".to_string(), "b".to_string()];

    instance.calculate_volume(arg1, arg2, arg3, arg4, arg5, arg6);
}
