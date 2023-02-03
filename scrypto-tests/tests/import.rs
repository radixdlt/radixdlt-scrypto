#![cfg_attr(not(feature = "std"), no_std)]

use sbor::rust::string::String;
use sbor::rust::string::ToString;
use sbor::rust::vec::Vec;
use scrypto::component::*;
use scrypto::prelude::*;
use scrypto::{blueprint, import};

// base directory: `scrypto-derive`
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
                "ident": "stateless_func",
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
                },
                "export_name": "Simple_stateless_func_main"
            },
            {
                "ident": "calculate_volume",
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
                                    "element_types": [
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
                                    "element_type": {
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
                                    "element_type": {
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
                },
                "export_name": "Simple_calculate_volume_main"
            }
        ]
    }
}
"#
}

#[blueprint]
mod use_import {
    struct UseImport {
        simple: SimpleGlobalComponentRef,
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
fn test_import_from_abi() {
    let _ = SimpleGlobalComponentRef::from(ComponentAddress::Normal([0; 26]));
}
