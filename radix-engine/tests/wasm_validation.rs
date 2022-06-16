#[rustfmt::skip]
pub mod test_runner;

use radix_engine::wasm::{PrepareError, WasmValidator};
use test_runner::wat2wasm;

#[test]
fn test_large_data() {
    let code = wat2wasm(&include_str!("wasm/large_data.wat"));
    let result = WasmValidator::validate(&code);

    assert_eq!(Err(PrepareError::NotInstantiatable), result);
}
