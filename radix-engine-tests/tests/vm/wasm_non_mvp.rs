use paste::paste;
use radix_common::prelude::*;
use radix_engine::vm::{
    wasm::{PrepareError, WasmModule},
    ScryptoVmVersion,
};
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

                assert!(WasmModule::init(&code, ScryptoVmVersion::latest()).unwrap().contains_sign_ext_ops());

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

macro_rules! get_ledger {
    ($version:expr) => {
        LedgerSimulatorBuilder::new()
            .with_custom_protocol(|builder: radix_engine::updates::ProtocolBuilder| {
                builder.from_bootstrap_to($version)
            })
            .build()
    };
}

macro_rules! manifest_execute_test_function {
    ($ledger:expr, $package_address:expr) => {{
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function($package_address, "Test", "f", manifest_args!())
            .build();
        $ledger.execute_manifest(manifest, vec![])
    }};
}

#[test]
fn test_wasm_non_mvp_multi_value_function_return_multiple_values_cuttlefish_failure() {
    // Arrange
    let code = wat2wasm(
        &include_local_wasm_str!("multi_value_function_return_multiple_values.wat")
            .replace("${a}", "10")
            .replace("${b}", "20"),
    );

    // Act
    let mut ledger = get_ledger!(ProtocolVersion::Cuttlefish);
    let receipt =
        ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

    // Assert
    receipt.expect_specific_failure(|error| match error {
        RuntimeError::ApplicationError(ApplicationError::PackageError(
            PackageError::InvalidWasm(PrepareError::ValidationError(message)),
        )) => message.contains(
            "func type returns multiple values but the multi-value feature is not enabled",
        ),
        _ => false,
    });
}

#[test]
fn test_wasm_non_mvp_multi_value_function_return_multiple_values_dugong_success() {
    // Arrange
    let code = wat2wasm(
        &include_local_wasm_str!("multi_value_function_return_multiple_values.wat")
            .replace("${a}", "10")
            .replace("${b}", "20"),
    );

    // Act
    let mut ledger = get_ledger!(ProtocolVersion::Dugong);
    let receipt =
        ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

    // Assert
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Act
    let receipt = manifest_execute_test_function!(ledger, package_address);

    // Assert
    let outcome: i32 = receipt.expect_commit(true).output(1);
    assert_eq!(outcome, 30);
}

#[test]
fn test_wasm_non_mvp_multi_value_if_return_multiple_values_cuttlefish_failure() {
    // Arrange
    let code = wat2wasm(
        &include_local_wasm_str!("multi_value_if_return_multiple_values.wat")
            .replace("${a}", "10")
            .replace("${b}", "20"),
    );

    // Act
    let mut ledger = get_ledger!(ProtocolVersion::Cuttlefish);
    let receipt =
        ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

    // Assert
    receipt.expect_specific_failure(|error| match error {
        RuntimeError::ApplicationError(ApplicationError::PackageError(
            PackageError::InvalidWasm(PrepareError::ValidationError(message)),
        )) => message.contains(
            "func type returns multiple values but the multi-value feature is not enabled",
        ),
        _ => false,
    });
}

#[test]
fn test_wasm_non_mvp_multi_value_if_return_multiple_values_dugong_success() {
    // Arrange
    let code = wat2wasm(
        &include_local_wasm_str!("multi_value_if_return_multiple_values.wat")
            .replace("${a}", "10")
            .replace("${b}", "20"),
    );

    // Act
    let mut ledger = get_ledger!(ProtocolVersion::Dugong);
    let receipt =
        ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

    // Assert
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Act
    let receipt = manifest_execute_test_function!(ledger, package_address);

    // Assert
    let outcome: i32 = receipt.expect_commit(true).output(1);
    assert_eq!(outcome, 30);
}

#[test]
fn test_wasm_non_mvp_multi_value_loop_return_multiple_values_cuttlefish_failure() {
    // Arrange
    let code = wat2wasm(
        &include_local_wasm_str!("multi_value_loop_or_block_return_multiple_values.wat")
            .replace("${loop_or_block}", "loop")
            .replace("${a}", "10")
            .replace("${b}", "20"),
    );

    // Act
    let mut ledger = get_ledger!(ProtocolVersion::Cuttlefish);
    let receipt =
        ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

    // Assert
    receipt.expect_specific_failure(|error| match error {
        RuntimeError::ApplicationError(ApplicationError::PackageError(
            PackageError::InvalidWasm(PrepareError::ValidationError(message)),
        )) => message.contains(
            "func type returns multiple values but the multi-value feature is not enabled",
        ),
        _ => false,
    });
}

#[test]
fn test_wasm_non_mvp_multi_value_block_return_multiple_values_dugong_success() {
    // Arrange
    let code = wat2wasm(
        &include_local_wasm_str!("multi_value_loop_or_block_return_multiple_values.wat")
            .replace("${loop_or_block}", "block")
            .replace("${a}", "10")
            .replace("${b}", "20"),
    );

    // Act
    let mut ledger = get_ledger!(ProtocolVersion::Dugong);
    let receipt =
        ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

    // Assert
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Act
    let receipt = manifest_execute_test_function!(ledger, package_address);

    // Assert
    let outcome: i32 = receipt.expect_commit(true).output(1);
    assert_eq!(outcome, 30);
}

#[test]
fn test_wasm_non_mvp_multi_value_loop_return_multiple_values_dugong_success() {
    // Arrange
    let code = wat2wasm(
        &include_local_wasm_str!("multi_value_loop_or_block_return_multiple_values.wat")
            .replace("${loop_or_block}", "loop")
            .replace("${a}", "10")
            .replace("${b}", "20"),
    );

    // Act
    let mut ledger = get_ledger!(ProtocolVersion::Dugong);
    let receipt =
        ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

    // Assert
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Act
    let receipt = manifest_execute_test_function!(ledger, package_address);

    // Assert
    let outcome: i32 = receipt.expect_commit(true).output(1);
    assert_eq!(outcome, 30);
}

#[test]
fn test_wasm_non_mvp_multi_value_block_params_cuttlefish_failure() {
    // Arrange
    let code = wat2wasm(
        &include_local_wasm_str!("multi_value_loop_or_block_params.wat")
            .replace("${loop_or_block}", "block")
            .replace("${a}", "10")
            .replace("${b}", "20"),
    );

    // Act
    let mut ledger = get_ledger!(ProtocolVersion::Cuttlefish);

    let receipt =
        ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

    // Assert
    receipt.expect_specific_failure(|error| match error {
        RuntimeError::ApplicationError(ApplicationError::PackageError(
            PackageError::InvalidWasm(PrepareError::ValidationError(message)),
        )) => message.contains(
            "blocks, loops, and ifs may only produce a resulttype when multi-value is not enabled",
        ),
        _ => false,
    });
}

#[test]
fn test_wasm_non_mvp_multi_value_loop_params_cuttlefish_failure() {
    // Arrange
    let code = wat2wasm(
        &include_local_wasm_str!("multi_value_loop_or_block_params.wat")
            .replace("${loop_or_block}", "loop")
            .replace("${a}", "10")
            .replace("${b}", "20"),
    );

    // Act
    let mut ledger = get_ledger!(ProtocolVersion::Cuttlefish);
    let receipt =
        ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

    // Assert
    receipt.expect_specific_failure(|error| match error {
        RuntimeError::ApplicationError(ApplicationError::PackageError(
            PackageError::InvalidWasm(PrepareError::ValidationError(message)),
        )) => message.contains(
            "blocks, loops, and ifs may only produce a resulttype when multi-value is not enabled",
        ),
        _ => false,
    });
}

#[test]
fn test_wasm_non_mvp_multi_value_block_params_dugong_success() {
    // Arrange
    let code = wat2wasm(
        &include_local_wasm_str!("multi_value_loop_or_block_params.wat")
            .replace("${loop_or_block}", "block")
            .replace("${a}", "10")
            .replace("${b}", "20"),
    );

    // Act
    let mut ledger = get_ledger!(ProtocolVersion::Dugong);
    let receipt =
        ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

    // Assert
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Act
    let receipt = manifest_execute_test_function!(ledger, package_address);

    // Assert
    let outcome: i32 = receipt.expect_commit(true).output(1);
    assert_eq!(outcome, 30);
}

#[test]
fn test_wasm_non_mvp_multi_value_loop_params_dugong_success() {
    // Arrange
    let code = wat2wasm(
        &include_local_wasm_str!("multi_value_loop_or_block_params.wat")
            .replace("${loop_or_block}", "loop")
            .replace("${a}", "10")
            .replace("${b}", "20"),
    );

    // Act
    let mut ledger = get_ledger!(ProtocolVersion::Dugong);

    let receipt =
        ledger.try_publish_package((code, single_function_package_definition("Test", "f")));
    // Assert
    let package_address = receipt.expect_commit(true).new_package_addresses()[0];

    // Act
    let receipt = manifest_execute_test_function!(ledger, package_address);

    // Assert
    let outcome: i32 = receipt.expect_commit(true).output(1);
    assert_eq!(outcome, 30);
}
