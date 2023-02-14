use radix_engine_interface::api::wasm::*;
use scrypto::abi::{BlueprintAbi, Fields, Fn, Type};
use scrypto::prelude::*;

static LARGE: u32 = u32::MAX / 2;
static MAX: u32 = u32::MAX;
static ZERO: u32 = 0;

#[no_mangle]
pub extern "C" fn LargeReturnSize_f(_args: u64) -> Slice {
    Slice(LARGE as u64)
}

#[no_mangle]
pub extern "C" fn MaxReturnSize_f(_args: u64) -> Slice {
    Slice(MAX as u64)
}

#[no_mangle]
pub extern "C" fn ZeroReturnSize_f(_args: u64) -> Slice {
    Slice(ZERO as u64)
}

#[no_mangle]
pub extern "C" fn LargeReturnSize_abi() -> Slice {
    let structure = Type::Struct {
        name: "LargeReturnSize".to_string(),
        fields: Fields::Unit,
    };
    let abi = BlueprintAbi {
        structure,
        fns: vec![Fn {
            ident: "f".to_string(),
            mutability: Option::None,
            input: Type::Struct {
                name: "Any".to_string(),
                fields: Fields::Named { named: vec![] },
            },
            output: Type::Tuple {
                element_types: vec![],
            },
            export_name: "LargeReturnSize_f".to_string(),
        }],
    };
    ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto_encode(&abi).unwrap())
}

#[no_mangle]
pub extern "C" fn MaxReturnSize_abi() -> Slice {
    let structure = Type::Struct {
        name: "MaxReturnSize".to_string(),
        fields: Fields::Unit,
    };
    let abi = BlueprintAbi {
        structure,
        fns: vec![Fn {
            ident: "f".to_string(),
            mutability: Option::None,
            input: Type::Struct {
                name: "Any".to_string(),
                fields: Fields::Named { named: vec![] },
            },
            output: Type::Tuple {
                element_types: vec![],
            },
            export_name: "MaxReturnSize_f".to_string(),
        }],
    };

    ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto_encode(&abi).unwrap())
}

#[no_mangle]
pub extern "C" fn ZeroReturnSize_abi() -> Slice {
    let structure = Type::Struct {
        name: "ZeroReturnSize".to_string(),
        fields: Fields::Unit,
    };
    let abi = BlueprintAbi {
        structure,
        fns: vec![Fn {
            ident: "f".to_string(),
            mutability: Option::None,
            input: Type::Struct {
                name: "Any".to_string(),
                fields: Fields::Named { named: vec![] },
            },
            output: Type::Tuple {
                element_types: vec![],
            },
            export_name: "ZeroReturnSize_f".to_string(),
        }],
    };

    ::scrypto::engine::wasm_api::forget_vec(::scrypto::data::scrypto_encode(&abi).unwrap())
}
