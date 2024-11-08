use paste::paste;
use radix_common::prelude::*;
use radix_engine::vm::wasm::WasmModule;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

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
                let code = wat2wasm(&include_local_wasm_str!("sign_extensions.wat")
                        .replace("${base}", stringify!($type))
                        .replace("${instruction}", $instruction)
                        .replace("${initial}", &input.to_string())
                        .replace("${value_kind}", &value_kind)
                        .replace("${slice_len}", &slice_len));

                assert!(WasmModule::init(&code).unwrap().contains_sign_ext_ops());

                let mut ledger = LedgerSimulatorBuilder::new().build();
                let package_address = ledger.publish_package(
                    (code, single_function_package_definition("Test", "f")),
                    BTreeMap::new(),
                    OwnerRole::None,
                );
                let manifest = ManifestBuilder::new()
                    .lock_fee_from_faucet()
                    .call_function(package_address, "Test", "f", manifest_args!())
                    .build();
                let receipt = ledger.execute_manifest(manifest, vec![]);

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
fn test_wasm_non_mvp_mutable_globals_import() {
    // Arrange
    let code = wat2wasm(&include_local_wasm_str!("mutable_globals_import.wat"));

    // Act
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .publish_package_advanced(
            None,
            code,
            single_function_package_definition("Test", "f"),
            BTreeMap::new(),
            OwnerRole::None,
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_failure_containing_error("InvalidImport");
}

#[test]
fn test_wasm_non_mvp_mutable_globals_export() {
    // Arrange
    let code = wat2wasm(&include_local_wasm_str!("mutable_globals_export.wat"));

    // Act
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package(
        (code, single_function_package_definition("Test", "f")),
        BTreeMap::new(),
        OwnerRole::None,
    );
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(package_address, "Test", "f", manifest_args!())
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    // Assert
    assert!(receipt.is_commit_success());
}
