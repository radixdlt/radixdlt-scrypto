use radix_engine::vm::wasm::{InvalidMemory, PrepareError, WasmValidator};
use scrypto_unit::*;

#[test]
fn test_large_data() {
    let code = wat2wasm(&include_str!("wasm/large_data.wat"));
    let definition = single_function_package_definition("Test", "f");
    let result = WasmValidator::default().validate(&code, definition.blueprints.values().map(|s| &s.schema));

    assert!(matches!(
        result,
        Err(PrepareError::NotInstantiatable { .. })
    ));
}

#[test]
fn test_large_memory() {
    let code = wat2wasm(&include_str!("wasm/large_memory.wat"));
    let definition = single_function_package_definition("Test", "f");
    let result = WasmValidator::default().validate(&code, definition.blueprints.values().map(|s| &s.schema));

    assert_eq!(
        Err(PrepareError::InvalidMemory(
            InvalidMemory::InitialMemorySizeLimitExceeded
        )),
        result
    );
}
