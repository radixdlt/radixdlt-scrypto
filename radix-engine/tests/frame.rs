#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn test_max_call_depth_success() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("recursion");

    // Act
    // ============================
    // Stack layout:
    // * 0: Executor
    // * 1: Transaction Executor
    // * 2-15: Caller::call x 14
    // * 16: AuthZone::clear
    // ============================
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .pay_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Caller", "recursive", to_struct!(14u32))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_success();
}

#[test]
fn test_max_call_depth_failure() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let package_address = test_runner.extract_and_publish_package("recursion");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .pay_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(package_address, "Caller", "recursive", to_struct!(15u32))
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);

    // Assert
    receipt.expect_err(|e| matches!(e, RuntimeError::MaxCallDepthLimitReached));
}
