#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::borrow::ToOwned;
use sbor::rust::str::FromStr;
use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::component::*;
use scrypto::{blueprint, import};

// base directory: `scrypto-derive`
import! {
r#"
{
    "package_address": "056967d3d49213394892980af59be76e9b3e7cc4cb78237460d0c7",
    "blueprint_name": "Simple",
    "abi": {
        "value": {
            "type": "Custom",
            "name": "Simple",
            "generics": []
        },
        "functions": [
            {
                "name": "stateless_func",
                "input": {
                    "type": "Struct",
                    "name": "",
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
                "name": "test_custom_types",
                "input": {
                    "type": "Struct",
                    "name": "",
                    "fields": {
                        "type": "Named",
                        "named": [
                            [
                                "arg0",
                                {
                                    "type": "Custom",
                                    "name": "Decimal",
                                    "generics": []
                                }
                            ],
                            [
                                "arg1",
                                {
                                    "type": "Custom",
                                    "name": "PackageAddress",
                                    "generics": []
                                }
                            ],
                            [
                                "arg2",
                                {
                                    "type": "Custom",
                                    "name": "ComponentAddress",
                                    "generics": []
                                }
                            ],
                            [
                                "arg3",
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
                                }
                            ],
                            [
                                "arg4",
                                {
                                    "type": "Custom",
                                    "name": "Bucket",
                                    "generics": []
                                }
                            ],
                            [
                                "arg5",
                                {
                                    "type": "Custom",
                                    "name": "Proof",
                                    "generics": []
                                }
                            ],
                            [
                                "arg6",
                                {
                                    "type": "Custom",
                                    "name": "Vault",
                                    "generics": []
                                }
                            ],
                            [
                                "arg7",
                                {
                                    "type": "Custom",
                                    "name": "ResourceAddress",
                                    "generics": []
                                }
                            ]
                        ]
                    }
                },
                "output": {
                    "type": "Custom",
                    "name": "Bucket",
                    "generics": []
                }
            }
        ],
        "methods": [
            {
                "name": "calculate_volume",
                "mutability": "Immutable",
                "input": {
                    "type": "Struct",
                    "name": "",
                    "fields": {
                        "type": "Named",
                        "named": [
                            [
                                "arg0",
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
                                }
                            ],
                            [
                                "arg1",
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
                                }
                            ],
                            [
                                "arg2",
                                {
                                    "type": "Vec",
                                    "element": {
                                        "type": "String"
                                    }
                                }
                            ],
                            [
                                "arg3",
                                {
                                    "type": "U32"
                                }
                            ],
                            [
                                "arg4",
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
                                }
                            ],
                            [
                                "arg5",
                                {
                                    "type": "Array",
                                    "element": {
                                        "type": "String"
                                    },
                                    "length": 2
                                }
                            ]
                        ]
                    }
                },
                "output": {
                    "type": "U32"
                }
            }
        ]
    }
}
"#
}

blueprint! {
    struct UseImport {
        simple: Simple
    }

    impl UseImport {
        pub fn new(address: ComponentAddress) -> ComponentAddress {
            Self {
                simple: address.into(),
            }
            .instantiate()
            .globalize()
        }
    }
}

#[test]
#[should_panic] // asserts it compiles
fn test_import_from_abi() {
    let instance = Simple::from(ComponentAddress::from_str("").unwrap());

    let arg1 = Floor { x: 5, y: 12 };
    let arg2 = (1u8, 2u16);
    let arg3 = Vec::<String>::new();
    let arg4 = 5;
    let arg5 = Hello::A { x: 1 };
    let arg6 = ["a".to_owned(), "b".to_owned()];

    instance.calculate_volume(arg1, arg2, arg3, arg4, arg5, arg6);
}
