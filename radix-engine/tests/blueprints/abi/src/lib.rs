use scrypto::abi::{BlueprintAbi, Fields, Fn, Type};
use scrypto::prelude::*;

blueprint! {
    struct AbiComponent {}

    impl AbiComponent {
        pub fn create_component() -> ComponentAddress {
            let component = Self {}.instantiate();
            component.globalize()
        }

        pub fn create_invalid_abi_component() -> ComponentAddress {
            let mut component = Self {}.instantiate();
            component.add_access_check(
                AccessRules::new()
                    .method("no_method", rule!(require("something")), rule!(deny_all))
                    .default(rule!(allow_all), AccessRule::DenyAll),
            );
            component.globalize()
        }
    }
}

#[no_mangle]
pub extern "C" fn AbiComponent2_main(_input: *mut u8) -> *mut u8 {
    ::scrypto::buffer::scrypto_encode_to_buffer(&()).unwrap()
}

#[no_mangle]
pub extern "C" fn AbiComponent2_abi(_input: *mut u8) -> *mut u8 {
    let structure = Type::Struct {
        name: "AbiComponent2".to_string(),
        fields: Fields::Unit,
    };
    let abi = BlueprintAbi {
        structure,
        fns: vec![
            Fn {
                ident: "invalid_output".to_string(),
                mutability: Option::None,
                input: Type::Tuple {
                    element_types: vec![],
                },
                output: Type::U8,
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "unit".to_string(),
                mutability: Option::None,
                input: Type::Tuple {
                    element_types: vec![],
                },
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "bool".to_string(),
                mutability: Option::None,
                input: Type::Bool,
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "i8".to_string(),
                mutability: Option::None,
                input: Type::I8,
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "i16".to_string(),
                mutability: Option::None,
                input: Type::I16,
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "i32".to_string(),
                mutability: Option::None,
                input: Type::I32,
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "i64".to_string(),
                mutability: Option::None,
                input: Type::I64,
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "i128".to_string(),
                mutability: Option::None,
                input: Type::I128,
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "u8".to_string(),
                mutability: Option::None,
                input: Type::U8,
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "u16".to_string(),
                mutability: Option::None,
                input: Type::U16,
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "u32".to_string(),
                mutability: Option::None,
                input: Type::U32,
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "u64".to_string(),
                mutability: Option::None,
                input: Type::U64,
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "u128".to_string(),
                mutability: Option::None,
                input: Type::U128,
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "result".to_string(),
                mutability: Option::None,
                input: Type::Result {
                    okay_type: Box::new(Type::Tuple {
                        element_types: vec![],
                    }),
                    err_type: Box::new(Type::Tuple {
                        element_types: vec![],
                    }),
                },
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "tree_map".to_string(),
                mutability: Option::None,
                input: Type::TreeMap {
                    key_type: Box::new(Type::Tuple {
                        element_types: vec![],
                    }),
                    value_type: Box::new(Type::Tuple {
                        element_types: vec![],
                    }),
                },
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
            Fn {
                ident: "hash_set".to_string(),
                mutability: Option::None,
                input: Type::HashSet {
                    element_type: Box::new(Type::Tuple {
                        element_types: vec![],
                    }),
                },
                output: Type::Tuple {
                    element_types: vec![],
                },
                export_name: "AbiComponent2_main".to_string(),
            },
        ],
    };

    ::scrypto::buffer::scrypto_encode_to_buffer(&abi).unwrap()
}
