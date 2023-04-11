use radix_engine::errors::{ModuleError, RuntimeError};
use radix_engine::system::kernel_modules::costing::CostingError;
use radix_engine::types::*;
use radix_engine_constants::DEFAULT_MAX_CALL_DEPTH;
use scrypto_unit::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_max_call_depth_success() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/recursion");

    // Act
    // ============================
    // Stack layout:
    // * 0: Executor
    // * 1: Transaction Executor
    // * 2-15: Caller::call x 14
    // ============================
    let num_calls = u32::try_from(DEFAULT_MAX_CALL_DEPTH).unwrap() - 1u32;
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "Caller",
            "recursive",
            manifest_args!(num_calls),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_commit_success();
}

#[test]
fn test_max_call_depth_failure() {
    // Arrange
    let mut test_runner = TestRunner::builder().build();
    let package_address = test_runner.compile_and_publish("./tests/blueprints/recursion");

    // Act
    let manifest = ManifestBuilder::new()
        .lock_fee(test_runner.faucet_component(), 10.into())
        .call_function(
            package_address,
            "Caller",
            "recursive",
            manifest_args!(16u32),
        )
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_specific_failure(|e| {
        matches!(
            e,
            RuntimeError::ModuleError(ModuleError::CostingError(
                CostingError::MaxCallDepthLimitReached
            ))
        )
    });
}
