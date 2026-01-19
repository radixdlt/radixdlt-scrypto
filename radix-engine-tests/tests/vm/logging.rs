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

    ledger.execute_manifest(manifest, vec![])
}

fn call_log_macro<S: AsRef<str>>(
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>,
    package_address: PackageAddress,
    level: Level,
    message: S,
) -> TransactionReceipt {
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Logger",
            "mutate_input_if_log_level_enabled",
            manifest_args!(level, message.as_ref().to_owned()),
        )
        .build();

    ledger.execute_manifest(manifest, vec![])
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

#[test]
fn test_log_macros_enabled() {
    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    // PackageLoader compiles with all logs enabled (CompileProfile::FastWithTraceLogs)
    let package_address = ledger.publish_package_simple(PackageLoader::get("logger"));

    let input = "2";
    let output_log = "Mutated input = 3";

    for level in [
        Level::Error,
        Level::Warn,
        Level::Info,
        Level::Debug,
        Level::Trace,
    ] {
        // Act
        let receipt = call_log_macro(&mut ledger, package_address, level, input);

        // Assert
        {
            receipt.expect_commit_success();

            let logs = receipt.expect_commit(true).application_logs.clone();
            let output = receipt.expect_commit(true).output::<u8>(1);

            assert_eq!(output, 3);

            let expected_logs = vec![(level, output_log.to_owned())];
            assert_eq!(logs, expected_logs)
        }
    }
}

#[test]
fn test_log_macros_disabled() {
    use std::path::PathBuf;

    // Arrange
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let manifest_dir = PathBuf::from_str(env!("CARGO_MANIFEST_DIR")).unwrap();
    let package_dir = manifest_dir
        .join("assets")
        .join("blueprints")
        .join("logger");
    // Disable all logging macros
    let package = ledger.compile_with_option(package_dir, CompileProfile::FastWithNoLogs);

    let package_address = ledger.publish_package_simple(package);
    let input = "2";

    for level in [
        Level::Error,
        Level::Warn,
        Level::Info,
        Level::Debug,
        Level::Trace,
    ] {
        // Act
        let receipt = call_log_macro(&mut ledger, package_address, level, input);

        // Assert
        {
            receipt.expect_commit_success();

            let logs = receipt.expect_commit(true).application_logs.clone();
            let output = receipt.expect_commit(true).output::<u8>(1);

            assert_eq!(output, 2);

            let expected_logs: Vec<(Level, String)> = vec![];
            assert_eq!(logs, expected_logs)
        }
    }
}
