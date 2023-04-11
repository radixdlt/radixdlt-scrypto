use radix_engine::{transaction::TransactionReceipt, types::*};
use radix_engine_interface::types::Level;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

fn log_message<S: AsRef<str>>(message: S, panic_log: bool) -> TransactionReceipt {
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/logger");

    let method_name = match panic_log {
        true => "panic_log",
        false => "no_panic_log",
    };

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "Logger",
            method_name,
            manifest_args!(message.as_ref().to_owned()),
        )
        .build();
    let receipt = test_runner.execute_manifest_ignoring_fee(manifest, vec![]);

    receipt
}

#[test]
fn log_messages_from_transaction_with_no_panic_shows_up_in_receipts() {
    // Arrange
    let message = "Hello World";
    let panic = false;

    // Act
    let receipt = log_message(message, panic);

    // Assert
    {
        receipt.expect_commit_success();

        let logs = receipt.expect_commit(true).application_logs.clone();
        let expected_logs = vec![(Level::Info, message.to_owned())];

        assert_eq!(expected_logs, logs)
    }
}

#[test]
fn log_messages_from_transaction_with_panic_shows_up_in_receipts() {
    // Arrange
    let message = "Hey Hey World";
    let panic = true;

    // Act
    let receipt = log_message(message, panic);

    // Assert
    {
        let logs = receipt.expect_commit(false).application_logs.clone();
        let expected_logs = vec![
            (Level::Info, message.to_owned()),
            (
                Level::Error,
                "Panicked at 'I'm panicking!', logger/src/lib.rs:16:13".to_owned(),
            ),
        ];

        assert_eq!(expected_logs, logs)
    }
}
