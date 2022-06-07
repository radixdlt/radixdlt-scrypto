#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use radix_engine::model::ResourceManagerError;
use scrypto::call_data;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

#[test]
fn test_resource_manager() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.publish_package("resource");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "ResourceTest",
            call_data!(create_fungible()),
        )
        .call_function(package_address, "ResourceTest", call_data!(query()))
        .call_function(package_address, "ResourceTest", call_data!(burn()))
        .call_function(
            package_address,
            "ResourceTest",
            call_data!(update_resource_metadata()),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    println!("{:?}", receipt);
    receipt.result.expect("It should work");
}

#[test]
fn mint_with_bad_granularity_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.publish_package("resource");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "ResourceTest",
            call_data![create_fungible_and_mint(0u8, dec!("0.1"))],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceManagerError(ResourceManagerError::InvalidAmount(
            Decimal::from("0.1"),
            0
        ))
    );
}

#[test]
fn mint_too_much_should_fail() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.publish_package("resource");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "ResourceTest",
            call_data![create_fungible_and_mint(0u8, dec!(100_000_000_001i128))],
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    let runtime_error = receipt.result.expect_err("Should be runtime error");
    assert_eq!(
        runtime_error,
        RuntimeError::ResourceManagerError(ResourceManagerError::MaxMintAmountExceeded)
    );
}
