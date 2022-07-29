#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::ledger::InMemorySubstateStore;
use radix_engine::model::ResourceManagerError;
use scrypto::core::Network;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn test_resource_manager() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("resource");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(
            package_address,
            "ResourceTest",
            "create_fungible",
            to_struct!(),
        )
        .call_function(package_address, "ResourceTest", "query", to_struct!())
        .call_function(package_address, "ResourceTest", "burn", to_struct!())
        .call_function(
            package_address,
            "ResourceTest",
            "update_resource_metadata",
            to_struct!(),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_success();
}

#[test]
fn mint_with_bad_granularity_should_fail() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("resource");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(
            package_address,
            "ResourceTest",
            "create_fungible_and_mint",
            to_struct!(0u8, dec!("0.1")),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_failure(|e| {
        if let RuntimeError::ResourceManagerError(ResourceManagerError::InvalidAmount(
            amount,
            granularity,
        )) = e
        {
            amount.eq(&Decimal::from("0.1")) && *granularity == 0
        } else {
            false
        }
    });
}

#[test]
fn mint_too_much_should_fail() {
    // Arrange
    let mut store = InMemorySubstateStore::with_bootstrap();
    let mut test_runner = TestRunner::new(true, &mut store);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.extract_and_publish_package("resource");

    // Act
    let manifest = ManifestBuilder::new(Network::LocalSimulator)
        .lock_fee(10.into(), SYSTEM_COMPONENT)
        .call_function(
            package_address,
            "ResourceTest",
            "create_fungible_and_mint",
            to_struct!(0u8, dec!(100_000_000_001i128)),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.expect_failure(|e| {
        matches!(
            e,
            RuntimeError::ResourceManagerError(ResourceManagerError::MaxMintAmountExceeded)
        )
    })
}
