use paste::paste;
use radix_engine::types::*;
#[cfg(not(feature = "wasmer"))]
use radix_engine::vm::wasm::run_module_with_mutable_global;
use radix_engine::vm::wasm::WasmModule;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;
use wabt::{wat2wasm_with_features, ErrorKind, Features};

// Verify WASM sign-extensions, which were enabled by default to the wasm32 target
// since rust 1.70.0
// see: https://github.com/rust-lang/rust/issues/109807
macro_rules! assert_sign_extensions {
    ($type:expr, $instruction:expr, $input:expr, $output:expr) => {
        paste! {
            #[test]
            fn [<test_wasm_non_mvp_sign_extensions_ $type _ $instruction>]() {
                // Arrange
                let value_kind = BasicValueKind::[<$type:upper>].as_u8().to_string();
                let slice_len = (
                    1 +                           // prefix byte
                    1 +                           // value kind byte
                    std::mem::size_of::<$type>()  // value bytes
                ).to_string();
                let input = $input as $type;

                // Act
                let code = wat2wasm(&include_str!("wasm/sign_extensions.wat")
                        .replace("${base}", stringify!($type))
                        .replace("${instruction}", $instruction)
                        .replace("${initial}", &input.to_string())
                        .replace("${value_kind}", &value_kind)
                        .replace("${slice_len}", &slice_len));

                let mut test_runner = TestRunner::builder().build();
                let package_address = test_runner.publish_package(
                    code,
                    single_function_package_definition("Test", "f"),
                    BTreeMap::new(),
                    OwnerRole::None,
                );
                let manifest = ManifestBuilder::new()
                    .lock_fee(test_runner.faucet_component(), 10.into())
                    .call_function(package_address, "Test", "f", manifest_args!())
                    .build();
                let receipt = test_runner.execute_manifest(manifest, vec![]);

                // Assert
                let outcome: $type = receipt.expect_commit(true).output(1);
                assert_eq!(outcome, $output as $type);
            }
        }
    };
}

assert_sign_extensions!(i32, "extend8_s", 0x44332211, 0x11);
assert_sign_extensions!(i32, "extend16_s", 0x44332211, 0x2211);
assert_sign_extensions!(i64, "extend8_s", 0x665544332211, 0x11);
assert_sign_extensions!(i64, "extend16_s", 0x665544332211, 0x2211);
assert_sign_extensions!(i64, "extend32_s", 0x665544332211, 0x44332211);

#[test]
fn test_wasm_non_mvp_expect_sign_ext_from_rust_code() {
    // Arrange
    let (code, _) = Compile::compile("./tests/blueprints/wasm_non_mvp");

    assert!(WasmModule::init(&code).unwrap().contains_sign_ext_ops())
}

// Below tests verify WASM "mutable-global" feature, which allows importing/exporting mutable globals.
// more details:
// - https://github.com/WebAssembly/mutable-global/blob/master/proposals/mutable-global/Overview.md

// NOTE!
//  We test only WASM code, because Rust currently does not use the WASM "global" construct for globals
//  (it places them into the linear memory instead).
//  more details:
//  - https://github.com/rust-lang/rust/issues/60825
//  - https://github.com/rust-lang/rust/issues/65987
#[test]
fn test_wasm_non_mvp_mutable_globals_build_with_feature_disabled() {
    let mut features = Features::new();
    features.disable_mutable_globals();

    assert!(
        match wat2wasm_with_features(include_str!("./wasm/mutable_globals.wat"), features) {
            Err(err) => {
                match err.kind() {
                    ErrorKind::Validate(msg) => {
                        println!("err = {:?}", msg);
                        msg.contains("mutable globals cannot be imported")
                    }
                    _ => false,
                }
            }
            Ok(_) => false,
        }
    )
}

#[cfg(not(feature = "wasmer"))]
#[test]
fn test_wasm_non_mvp_mutable_globals_execute_code() {
    // wat2wasm has "mutable-globals" enabled by default
    let code = wat2wasm(include_str!("./wasm/mutable_globals.wat"));

    let val = run_module_with_mutable_global(
        &code,
        "increase_global_value",
        "global_mutable_value",
        100,
        1000,
    );
    assert_eq!(val, 1100);

    let val = run_module_with_mutable_global(
        &code,
        "increase_global_value",
        "global_mutable_value",
        val,
        10000,
    );
    assert_eq!(val, 11100);
}
