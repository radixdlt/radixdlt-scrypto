use paste::paste;
use radix_common::prelude::*;
use radix_engine::vm::{
    wasm::{PrepareError, WasmModule},
    ScryptoVmVersion,
};
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

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

// Verify WASM sign-extensions, which were enabled by default to the wasm32 target
// since rust 1.70.0
// see: https://github.com/rust-lang/rust/issues/109807
mod sign_extensions {
    use super::*;

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
                    let receipt = manifest_execute_test_function!(ledger, package_address);

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
}

mod mutable_globals {
    use super::*;

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
        let receipt = manifest_execute_test_function!(ledger, package_address);

        // Assert
        assert!(receipt.is_commit_success());
    }
}

// Verify WASM multi-value, which was enabled by default to the wasm32 target
// since rust 1.82.0 (which switched to LLVM 19)
// see: https://blog.rust-lang.org/2024/09/24/webassembly-targets-change-in-default-target-features.html
mod multi_value {
    use super::*;

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
    fn test_wasm_non_mvp_multi_value_return_multiple_values_cuttlefish_failure() {
        for section in ["block", "loop"] {
            // Arrange
            let code = wat2wasm(
                &include_local_wasm_str!("multi_value_loop_or_block_return_multiple_values.wat")
                    .replace("${loop_or_block}", section)
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
    }

    #[test]
    fn test_wasm_non_mvp_multi_value_return_multiple_values_dugong_success() {
        for section in ["block", "loop"] {
            // Arrange
            let code = wat2wasm(
                &include_local_wasm_str!("multi_value_loop_or_block_return_multiple_values.wat")
                    .replace("${loop_or_block}", section)
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
    }

    #[test]
    fn test_wasm_non_mvp_multi_value_params_cuttlefish_failure() {
        for section in ["block", "loop"] {
            // Arrange
            let code = wat2wasm(
                &include_local_wasm_str!("multi_value_loop_or_block_params.wat")
                    .replace("${loop_or_block}", section)
                    .replace("${a}", "10")
                    .replace("${b}", "20"),
            );

            // Act
            let mut ledger = get_ledger!(ProtocolVersion::Cuttlefish);

            let receipt =
                ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

            // Assert
            receipt.expect_specific_failure(|error| {
                match error {
                    RuntimeError::ApplicationError(ApplicationError::PackageError(
                        PackageError::InvalidWasm(PrepareError::ValidationError(message)),
                    )) => message.contains(
                        "blocks, loops, and ifs may only produce a resulttype when multi-value is not enabled",
                    ),
                    _ => false,
                }
            });
        }
    }

    #[test]
    fn test_wasm_non_mvp_multi_value_params_dugong_success() {
        for section in ["block", "loop"] {
            // Arrange
            let code = wat2wasm(
                &include_local_wasm_str!("multi_value_loop_or_block_params.wat")
                    .replace("${loop_or_block}", section)
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
    }
}

// Verify WASM reference-types, which was enabled by default to the wasm32 target
// since rust 1.82.0 (which switched to LLVM 19)
// see: https://blog.rust-lang.org/2024/09/24/webassembly-targets-change-in-default-target-features.html
mod reference_types {
    use super::*;

    #[test]
    fn test_wasm_non_mvp_reference_types_externref_cuttlefish_failure() {
        // Arrange
        let code = wat2wasm(&include_local_wasm_str!("reference_types_externref.wat"));

        // Act
        let mut ledger = get_ledger!(ProtocolVersion::Cuttlefish);

        let receipt =
            ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

        // Assert
        receipt.expect_specific_failure(|error| match error {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(PrepareError::ValidationError(message)),
            )) => message.contains("reference types support is not enabled"),
            _ => false,
        });
    }

    #[test]
    fn test_wasm_non_mvp_reference_types_externref_dugong_success() {
        // Arrange
        let code = wat2wasm(&include_local_wasm_str!("reference_types_externref.wat"));

        // Act
        let mut ledger = get_ledger!(ProtocolVersion::Dugong);

        let receipt =
            ledger.try_publish_package((code, single_function_package_definition("Test", "f")));
        // Assert
        let package_address = receipt.expect_commit(true).new_package_addresses()[0];

        // Act
        let receipt = manifest_execute_test_function!(ledger, package_address);

        // Assert
        receipt.expect_commit(true);
    }

    #[test]
    fn test_wasm_non_mvp_reference_types_tables_cuttlefish_failure() {
        // Arrange
        let code = wat2wasm(
            &include_local_wasm_str!("reference_types_tables.wat")
                .replace("${index}", "0")
                .replace("${a}", "20")
                .replace("${b}", "10"),
        );

        // Act
        let mut ledger = get_ledger!(ProtocolVersion::Cuttlefish);

        let receipt =
            ledger.try_publish_package((code, single_function_package_definition("Test", "f")));

        // Assert
        receipt.expect_specific_failure(|error| match error {
            RuntimeError::ApplicationError(ApplicationError::PackageError(
                PackageError::InvalidWasm(PrepareError::ValidationError(message)),
            )) => message.contains("reference types support is not enabled"),
            _ => false,
        });
    }

    #[test]
    fn test_wasm_non_mvp_reference_types_tables_dugong_success() {
        for (index, result) in [
            ("0", 30),  // Add
            ("1", 200), // Multiply
            ("2", 10),  // Subtract
        ] {
            // Arrange
            let code = wat2wasm(
                &include_local_wasm_str!("reference_types_tables.wat")
                    .replace("${index}", index)
                    .replace("${a}", "20")
                    .replace("${b}", "10"),
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
            assert_eq!(outcome, result);
        }
    }

    #[test]
    fn test_wasm_non_mvp_reference_types_ref_func_should_success() {
        // TODO WASM this test demonstrates that RefFunc (and some other instructions enabled in
        // reference-types) are not supported in 'wasm-instrument'
        // see https://github.com/radixdlt/wasm-instrument/blob/405166c526aa60fa2af4e4b1122b156dbcc1bb15/src/stack_limiter/max_height.rs#L455
        // It would be perfect to update the 'wasm-instrument' crate.
        // If not it shall be carefully investigated if it is safe to enable 'reference-types'
        // (all WASM-related tests are running fine when 'reference-types' are enabled,
        // as if the Rust compiler (LLVM) was not using those instructions)

        // Arrange
        let code = wat2wasm(
            &include_local_wasm_str!("reference_types_ref_func.wat").replace("${index}", "0"),
        );

        // Act
        let mut ledger = get_ledger!(ProtocolVersion::Dugong);

        let receipt =
            ledger.try_publish_package((code, single_function_package_definition("Test", "f")));
        // Assert
        // This test should success but it returns below error
        // thread 'vm::wasm_non_mvp::reference_types::test_wasm_non_mvp_reference_types_ref_func_failure' panicked at
        //   /home/ubuntu/.cargo/registry/src/index.crates.io-6f17d22bba15001f/radix-wasm-instrument-1.0.0/src/stack_limiter/max_height.rs:455:17:
        //   not yet implemented: some reference types proposal are not supported
        let _package_address = receipt.expect_commit(true).new_package_addresses()[0];
    }
}
