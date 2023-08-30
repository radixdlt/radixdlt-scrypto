use radix_engine::types::*;
use radix_engine_interface::blueprints::resource::FromPublicKey;
use scrypto_unit::*;
use transaction::prelude::*;

#[test]
fn test_query_transaction_runtime_info() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, _) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/transaction_runtime");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "TransactionRuntimeTest",
            "query",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_generate_ruid() {
    // Arrange
    let mut test_runner = TestRunnerBuilder::new().build();
    let (public_key, _, _) = test_runner.new_allocated_account();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/transaction_runtime");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "TransactionRuntimeTest",
            "generate_ruid",
            manifest_args!(),
        )
        .call_function(
            package_address,
            "TransactionRuntimeTest",
            "generate_ruid",
            manifest_args!(),
        )
        .build();
    let receipt = test_runner.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );

    // Assert
    let ruid1: [u8; 32] = receipt.expect_commit_success().output(1);
    let ruid2: [u8; 32] = receipt.expect_commit_success().output(2);
    assert_ne!(ruid1, ruid2);
}
