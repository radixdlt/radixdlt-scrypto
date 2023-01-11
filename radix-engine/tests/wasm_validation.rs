use radix_engine::types::Type;
use radix_engine::wasm::{InvalidMemory, PrepareError, WasmValidator};
use scrypto_unit::*;

#[test]
fn test_large_data() {
    let code = wat2wasm(&include_str!("wasm/large_data.wat"));
    let abi = generate_single_function_abi(
        "Test",
        "f",
        Type::Tuple {
            element_types: vec![],
        },
    );
    let result = WasmValidator::default().validate(&code, &abi);

    assert_eq!(Err(PrepareError::NotInstantiatable), result);
}

#[test]
fn test_large_memory() {
    let code = wat2wasm(&include_str!("wasm/large_memory.wat"));
    let abi = generate_single_function_abi(
        "Test",
        "f",
        Type::Tuple {
            element_types: vec![],
        },
    );
    let result = WasmValidator::default().validate(&code, &abi);

    assert_eq!(
        Err(PrepareError::InvalidMemory(
            InvalidMemory::InitialMemorySizeLimitExceeded
        )),
        result
    );
}
