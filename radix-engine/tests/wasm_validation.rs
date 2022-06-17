#[rustfmt::skip]
pub mod test_runner;

use radix_engine::wasm::{InvalidMemory, PrepareError, WasmValidator};
use test_runner::wat2wasm;

#[test]
fn test_large_data() {
    let code = wat2wasm(&include_str!("wasm/large_data.wat"));
    let result = WasmValidator::default().validate(&code);

    assert_eq!(Err(PrepareError::NotInstantiatable), result);
}

#[test]
fn test_large_memory() {
    let code = wat2wasm(&include_str!("wasm/large_memory.wat"));
    let result = WasmValidator::default().validate(&code);

    assert_eq!(
        Err(PrepareError::InvalidMemory(
            InvalidMemory::InitialMemorySizeLimitExceeded
        )),
        result
    );
}
