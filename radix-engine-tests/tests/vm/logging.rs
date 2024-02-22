use radix_engine::{
    errors::{ApplicationError, RuntimeError},
    transaction::TransactionReceipt,
};
use radix_engine_interface::prelude::*;
use radix_engine_interface::types::Level;
use radix_engine_tests::common::*;
use scrypto_test::prelude::*;

fn call<S: AsRef<str>>(function_name: &str, message: S) -> TransactionReceipt {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.publish_package_simple(PackageLoader::get("logger"));

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Logger",
            function_name,
            manifest_args!(message.as_ref().to_owned()),
        )
        .build();
    let receipt = ledger.execute_manifest(manifest, vec![]);

    receipt
}

#[test]
fn test_emit_log() {
    // Arrange
    let function_name = "emit_log";
    let message = "Hello";

    // Act
    let receipt = call(function_name, message);

    // Assert
    {
        receipt.expect_commit_success();

        let logs = receipt.expect_commit(true).application_logs.clone();
        let expected_logs = vec![(Level::Info, message.to_owned())];

        assert_eq!(expected_logs, logs)
    }
}

#[test]
fn test_rust_panic() {
    // Arrange
    let function_name = "rust_panic";
    let message = "Hey";

    // Act
    let receipt = call(function_name, message);

    // Assert
    {
        let logs = receipt.expect_commit(false).application_logs.clone();
        assert!(logs.is_empty());

        receipt.expect_specific_failure(|e| match e {
            RuntimeError::ApplicationError(ApplicationError::PanicMessage(e)) => {
                e.eq("Hey @ logger/src/lib.rs:15:13")
            }
            _ => false,
        })
    }
}

#[test]
fn test_scrypto_panic() {
    // Arrange
    let function_name = "scrypto_panic";
    let message = "Hi";

    // Act
    let receipt = call(function_name, message);

    // Assert
    {
        let logs = receipt.expect_commit(false).application_logs.clone();
        assert!(logs.is_empty());

        receipt.expect_specific_failure(|e| match e {
            RuntimeError::ApplicationError(ApplicationError::PanicMessage(e)) => e.eq(message),
            _ => false,
        })
    }
}

#[test]
fn test_assert_length_5() {
    // Arrange
    let function_name = "assert_length_5";
    let message = "!5";

    // Act
    let receipt = call(function_name, message);

    // Assert
    {
        let logs = receipt.expect_commit(false).application_logs.clone();
        assert!(logs.is_empty());
        receipt.expect_specific_failure(|e| match e {
            RuntimeError::ApplicationError(ApplicationError::PanicMessage(e)) => {
                e.contains("logger/src/lib.rs:23:13")
            }
            _ => false,
        })
    }
}
