#[rustfmt::skip]
pub mod test_runner;

use crate::test_runner::TestRunner;
use radix_engine::engine::RuntimeError;
use scrypto::call_data;
use scrypto::prelude::*;
use transaction::builder::ManifestBuilder;

#[test]
fn cannot_make_cross_component_call_without_authorization() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (_, _, account) = test_runner.new_account();
    let auth = test_runner.create_non_fungible_resource(account.clone());
    let auth_id = NonFungibleId::from_u32(1);
    let auth_address = NonFungibleAddress::new(auth, auth_id);
    let authorization =
        AccessRules::new().method("get_component_state", rule!(require(auth_address.clone())));

    let package_address = test_runner.publish_package("component");
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "CrossComponent",
            call_data!(create_component_with_auth(authorization)),
        )
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);
    receipt.result.expect("Should be okay");
    let secured_component = receipt.new_component_addresses[0];

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "CrossComponent",
            call_data!(create_component()),
        )
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);
    receipt.result.expect("It should work");
    let my_component = receipt.new_component_addresses[0];

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            my_component,
            call_data!(cross_component_call(secured_component)),
        )
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    let error = receipt.result.expect_err("Should be error");
    assert_auth_error!(error);
}

#[test]
fn can_make_cross_component_call_with_authorization() {
    // Arrange
    let mut test_runner = TestRunner::new(true);
    let (key, sk, account) = test_runner.new_account();
    let auth = test_runner.create_non_fungible_resource(account.clone());
    let auth_id = NonFungibleId::from_u32(1);
    let auth_address = NonFungibleAddress::new(auth, auth_id.clone());
    let authorization =
        AccessRules::new().method("get_component_state", rule!(require(auth_address.clone())));

    let package_address = test_runner.publish_package("component");
    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "CrossComponent",
            call_data!(create_component_with_auth(authorization)),
        )
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);
    receipt.result.expect("Should be okay");
    let secured_component = receipt.new_component_addresses[0];

    let manifest = ManifestBuilder::new()
        .call_function(
            package_address,
            "CrossComponent",
            call_data!(create_component()),
        )
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);
    receipt.result.expect("Should be okay.");
    let my_component = receipt.new_component_addresses[0];

    let manifest = ManifestBuilder::new()
        .withdraw_from_account_by_ids(&BTreeSet::from([auth_id.clone()]), auth, account)
        .call_method_with_all_resources(my_component, "put_auth")
        .build();
    let signers = vec![key];
    let receipt = test_runner.execute_manifest(manifest, signers);
    receipt.result.expect("Should be okay.");

    // Act
    let manifest = ManifestBuilder::new()
        .call_method(
            my_component,
            call_data!(cross_component_call(secured_component)),
        )
        .build();
    let signers = vec![];
    let receipt = test_runner.execute_manifest(manifest, signers);

    // Assert
    receipt.result.expect("Should be okay");
}
