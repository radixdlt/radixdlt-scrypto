#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use scrypto::prelude::*;
use scrypto::to_struct;
use transaction::builder::ManifestBuilder;

#[test]
fn create_non_fungible_mutable() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package = test_runner.publish_package("non_fungible");

    // Act
    let manifest = ManifestBuilder::new()
        .call_function(
            package,
            "NonFungibleTest",
            "create_non_fungible_mutable",
            to_struct!(),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.result.expect("It should work");
}

#[test]
fn can_burn_non_fungible() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package = test_runner.publish_package("non_fungible");
    let manifest = ManifestBuilder::new()
        .call_function(
            package,
            "NonFungibleTest",
            "create_burnable_non_fungible",
            to_struct!(),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![]);
    receipt.result.expect("Should be okay.");
    let resource_address = receipt.new_resource_addresses[0];
    let non_fungible_address =
        NonFungibleAddress::new(resource_address, NonFungibleId::from_u32(0));
    let mut ids = BTreeSet::new();
    ids.insert(NonFungibleId::from_u32(0));

    // Act
    let manifest = ManifestBuilder::new()
        .withdraw_from_account(resource_address, account)
        .burn_non_fungible(non_fungible_address.clone())
        .call_function(
            package,
            "NonFungibleTest",
            "verify_does_not_exist",
            to_struct!(non_fungible_address),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);

    // Assert
    receipt.result.expect("Should be okay.");
}

#[test]
fn test_non_fungible() {
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.publish_package("non_fungible");

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "NonFungibleTest",
            "create_non_fungible_fixed",
            to_struct!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "update_and_get_non_fungible",
            to_struct!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "non_fungible_exists",
            to_struct!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "take_and_put_bucket",
            to_struct!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "take_and_put_vault",
            to_struct!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "get_non_fungible_ids_bucket",
            to_struct!(),
        )
        .call_function(
            package_address,
            "NonFungibleTest",
            "get_non_fungible_ids_vault",
            to_struct!(),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    println!("{:?}", receipt);
    receipt.result.expect("It should work");
}

#[test]
fn test_singleton_non_fungible() {
    let mut test_runner = TestRunner::new(true);
    let (public_key, _, account) = test_runner.new_account();
    let package_address = test_runner.publish_package("non_fungible");

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "NonFungibleTest",
            "singleton_non_fungible",
            to_struct!(),
        )
        .call_method_with_all_resources(account, "deposit_batch")
        .build();
    let receipt = test_runner.execute_manifest(manifest, vec![public_key]);
    println!("{:?}", receipt);
    receipt.result.expect("It should work");
}
